//! Durable workflow state and sequencing. Identity belongs to the saga record.

use super::ApplicationCommand;
use crate::platform::bus::{Command, KeyedCommand, stable_command_id};
use crate::saga::saga::Saga;
use crate::steps::graphql::acquire_storefront_token::{
    AcquireStorefrontToken, StorefrontTokenAcquired,
};
use crate::steps::graphql::extract_offer_details::{ExtractOfferDetails, OfferDetailsExtracted};
use crate::steps::graphql::offer_details::{FetchOfferDetails, OfferDetailsFetched, SourceType};
use crate::steps::graphql::product_catalog_ids::{
    FetchProductCatalogIdentifiers, ProductCatalogIdsRetrieved, initial,
};
use crate::steps::graphql::structuralize_offer_details::{
    OfferDetailsStructuralized, StructuralizeOfferDetails,
};
use crate::v2::prelude::InitialFillContexts;
use retailer_data_extractor::get_storefront_config;
use retailer_sauce::initial_fill::RequestedContext;
use retailer_sauce::prelude::StorefrontToken;
use serde::{Deserialize, Serialize};
use serde_json::to_vec;
use shared::retailer::RetailerCode;
use std::collections::BTreeMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct ResultRef {
    pub event: Uuid,
    pub object: Uuid,
}
/// A skip requires explicit evidence; queue errors never create one.
#[derive(Debug, Serialize, Deserialize)]
pub(super) struct Skip {
    pub reason: String,
    pub evidence: String,
}
#[derive(Debug, Default, Serialize, Deserialize)]
pub(super) struct Product {
    pub fetched: Option<ResultRef>,
    pub extracted: Option<ResultRef>,
    pub structuralized: Option<ResultRef>,
    pub failure: Option<String>,
    pub skipped: Option<Skip>,
}
#[derive(Debug, Serialize, Deserialize)]
pub(super) struct Catalog {
    pub context: RequestedContext,
    pub resolved: Option<RequestedContext>,
    pub event: Option<Uuid>,
    pub products: BTreeMap<String, Product>,
    pub failure: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ShopifyGraphQlInitialFillSaga {
    pub(super) catalogs: Vec<Catalog>,
    pub(super) acquire_token: bool,
    storefront_url: String,
    pub(super) initialization: Option<Uuid>,
    pub(super) token_event: Option<Uuid>,
    storefront_token: Option<StorefrontToken>,
    pub(super) token_failure: Option<String>,
}
impl Saga for ShopifyGraphQlInitialFillSaga {
    const TYPE: &'static str = "ShopifyGraphQlInitialFillSaga";
}
impl ShopifyGraphQlInitialFillSaga {
    pub(crate) fn snapshot(retailer: RetailerCode) -> Result<Self, String> {
        let config =
            get_storefront_config(retailer).ok_or("retailer has no storefront configuration")?;
        if !config.access.supports_graphql() {
            return Err("retailer does not support GraphQL initial fill".into());
        }
        Ok(Self {
            catalogs: config
                .initial_fill_contexts()?
                .into_iter()
                .map(|context| Catalog {
                    context,
                    resolved: None,
                    event: None,
                    products: BTreeMap::new(),
                    failure: None,
                })
                .collect(),
            acquire_token: config.attempts_token(),
            storefront_url: config.storefront_url().into(),
            initialization: None,
            token_event: None,
            storefront_token: None,
            token_failure: None,
        })
    }
    pub(crate) fn initial_commands(
        &mut self,
        namespace: Uuid,
        retailer: RetailerCode,
        event_id: Uuid,
    ) -> Vec<ApplicationCommand> {
        self.initialization.get_or_insert(event_id);
        if self.acquire_token {
            vec![ApplicationCommand::AcquireStorefrontToken(KeyedCommand {
                id: stable_command_id(&namespace, AcquireStorefrontToken::NAME.as_bytes()),
                command: AcquireStorefrontToken::new(retailer, self.storefront_url.clone()),
            })]
        } else {
            self.catalog_commands(namespace, retailer)
                .into_iter()
                .map(ApplicationCommand::FetchProductCatalogIdentifiers)
                .collect()
        }
    }
    pub(super) fn catalog_commands(
        &self,
        namespace: Uuid,
        retailer: RetailerCode,
    ) -> Vec<KeyedCommand<FetchProductCatalogIdentifiers>> {
        initial(
            namespace,
            retailer,
            self.storefront_token.clone(),
            &self
                .catalogs
                .iter()
                .map(|catalog| catalog.context.clone())
                .collect::<Vec<_>>(),
        )
    }
    pub(super) fn on_token(
        &mut self,
        namespace: Uuid,
        retailer: RetailerCode,
        cause: Uuid,
        event_id: Uuid,
        event: &StorefrontTokenAcquired,
    ) -> Result<Vec<ApplicationCommand>, String> {
        if !self.acquire_token
            || cause != stable_command_id(&namespace, AcquireStorefrontToken::NAME.as_bytes())
        {
            return Ok(vec![]);
        }
        if event.url != self.storefront_url {
            return Err("token result storefront mismatch".into());
        }
        if self.token_event.is_none() {
            if self.token_failure.is_some() {
                return Ok(vec![]);
            }
            self.token_event = Some(event_id);
            self.storefront_token = event.storefront_token.clone();
        }
        Ok(self
            .catalog_commands(namespace, retailer)
            .into_iter()
            .map(ApplicationCommand::FetchProductCatalogIdentifiers)
            .collect())
    }
    pub(super) fn detail_command(
        &self,
        namespace: Uuid,
        retailer: RetailerCode,
        catalog: usize,
        product: &str,
    ) -> KeyedCommand<FetchOfferDetails> {
        let entry = &self.catalogs[catalog];
        let context = entry.resolved.as_ref().unwrap_or(&entry.context);
        // Requested context keeps identities distinct even when results resolve alike.
        let key = to_vec(&(
            FetchOfferDetails::NAME,
            format!("{retailer:?}"),
            &entry.context.country,
            &entry.context.language,
            product,
        ))
        .expect("key serializes");
        KeyedCommand {
            id: stable_command_id(&namespace, &key),
            command: FetchOfferDetails {
                retailer_code: retailer,
                context: Some(context.clone()),
                storefront_token: self.storefront_token.clone(),
                source: SourceType::GraphQl {
                    product_id: product.into(),
                },
            },
        }
    }
    pub(super) fn on_catalog(
        &mut self,
        namespace: Uuid,
        retailer: RetailerCode,
        cause: Uuid,
        event_id: Uuid,
        event: &ProductCatalogIdsRetrieved,
    ) -> Result<Vec<ApplicationCommand>, String> {
        if self.acquire_token && self.token_event.is_none() {
            return Ok(vec![]);
        }
        let Some(index) = self
            .catalog_commands(namespace, retailer)
            .iter()
            .position(|command| command.id == cause)
        else {
            return Ok(vec![]);
        };
        let catalog = &mut self.catalogs[index];
        let context = event
            .context
            .as_ref()
            .ok_or("catalog result has no context")?;
        if context.language != catalog.context.language
            || (catalog.context.country.is_some() && context.country != catalog.context.country)
        {
            return Err("catalog result context mismatch".into());
        }
        if catalog.event.is_none() {
            if catalog.failure.is_some() {
                return Ok(vec![]);
            }
            catalog.event = Some(event_id);
            catalog.resolved = Some(context.clone());
            for product in &event.product_ids {
                catalog.products.entry(product.clone()).or_default();
            }
        }
        Ok(self.catalogs[index]
            .products
            .keys()
            .map(|product| {
                ApplicationCommand::FetchOfferDetails(
                    self.detail_command(namespace, retailer, index, product),
                )
            })
            .collect())
    }
    pub(super) fn locate_fetch(
        &self,
        namespace: Uuid,
        retailer: RetailerCode,
        cause: Uuid,
    ) -> Option<(usize, String)> {
        self.catalogs
            .iter()
            .enumerate()
            .find_map(|(index, catalog)| {
                catalog
                    .products
                    .keys()
                    .find(|product| {
                        self.detail_command(namespace, retailer, index, product).id == cause
                    })
                    .map(|product| (index, product.clone()))
            })
    }
    pub(super) fn locate_derived(&self, cause: Uuid, extracting: bool) -> Option<(usize, String)> {
        self.catalogs
            .iter()
            .enumerate()
            .find_map(|(index, catalog)| {
                catalog.products.iter().find_map(|(key, product)| {
                    let reference = if extracting {
                        &product.fetched
                    } else {
                        &product.extracted
                    };
                    let name = if extracting {
                        ExtractOfferDetails::NAME
                    } else {
                        StructuralizeOfferDetails::NAME
                    };
                    reference
                        .as_ref()
                        .filter(|reference| {
                            stable_command_id(&reference.object, name.as_bytes()) == cause
                        })
                        .map(|_| (index, key.clone()))
                })
            })
    }
    fn validate_product(
        &self,
        index: usize,
        product_id: &str,
        event: &OfferDetailsFetched,
    ) -> Result<(), String> {
        let context = self.catalogs[index]
            .resolved
            .as_ref()
            .ok_or("missing catalog context")?;
        let observed = event.context.as_ref().ok_or("missing product context")?;
        if context.language != observed.language
            || context
                .country
                .as_ref()
                .or(context.observed_country.as_ref())
                .is_some_and(|country| country != &observed.country)
            || context
                .currency
                .as_ref()
                .is_some_and(|currency| currency != &observed.currency)
        {
            return Err("product result context mismatch".into());
        }
        if event.source
            != (SourceType::GraphQl {
                product_id: product_id.into(),
            })
        {
            return Err("product result identity mismatch".into());
        }
        Ok(())
    }
    pub(super) fn accept_fetch(
        &mut self,
        index: usize,
        product_id: &str,
        event_id: Uuid,
        event: &OfferDetailsFetched,
    ) -> Result<Option<ResultRef>, String> {
        self.validate_product(index, product_id, event)?;
        let product = self.catalogs[index]
            .products
            .get_mut(product_id)
            .expect("located product");
        if product.fetched.is_none() && product.failure.is_none() && product.skipped.is_none() {
            product.fetched = Some(ResultRef {
                event: event_id,
                object: event.storage.object_storage_id,
            });
        }
        Ok(product.fetched.clone())
    }
    pub(super) fn accept_extracted(
        &mut self,
        index: usize,
        product_id: &str,
        event_id: Uuid,
        event: &OfferDetailsExtracted,
    ) -> Result<Option<ResultRef>, String> {
        self.validate_product(index, product_id, &event.fetched)?;
        let product = self.catalogs[index]
            .products
            .get_mut(product_id)
            .expect("located product");
        if event.fetched.source
            != (SourceType::GraphQl {
                product_id: product_id.into(),
            })
            || product.fetched.as_ref().map(|value| value.object)
                != Some(event.fetched.storage.object_storage_id)
        {
            return Err("extracted result source mismatch".into());
        }
        if product.extracted.is_none() && product.failure.is_none() && product.skipped.is_none() {
            product.extracted = Some(ResultRef {
                event: event_id,
                object: event.storage.object_storage_id,
            });
        }
        Ok(product.extracted.clone())
    }
    pub(super) fn accept_structuralized(
        &mut self,
        index: usize,
        product_id: &str,
        event_id: Uuid,
        event: &OfferDetailsStructuralized,
    ) -> Result<(), String> {
        self.validate_product(index, product_id, &event.extracted.fetched)?;
        let product = self.catalogs[index]
            .products
            .get_mut(product_id)
            .expect("located product");
        if event.extracted.fetched.source
            != (SourceType::GraphQl {
                product_id: product_id.into(),
            })
            || product.fetched.as_ref().map(|value| value.object)
                != Some(event.extracted.fetched.storage.object_storage_id)
            || product.extracted.as_ref().map(|value| value.object)
                != Some(event.extracted.storage.object_storage_id)
        {
            return Err("structuralized result source mismatch".into());
        }
        if product.structuralized.is_none()
            && product.failure.is_none()
            && product.skipped.is_none()
        {
            product.structuralized = Some(ResultRef {
                event: event_id,
                object: event.storage.object_storage_id,
            });
        }
        Ok(())
    }
    pub(super) fn status(&self) -> &'static str {
        if self.acquire_token && self.token_event.is_none() {
            return if self.token_failure.is_some() {
                "failed"
            } else {
                "in_progress"
            };
        }
        if self.catalogs.iter().any(|catalog| {
            (catalog.event.is_none() && catalog.failure.is_none())
                || catalog.products.values().any(|product| {
                    product.structuralized.is_none()
                        && product.failure.is_none()
                        && product.skipped.is_none()
                })
        }) {
            return "in_progress";
        }
        if self.catalogs.iter().any(|catalog| {
            catalog.failure.is_some()
                || catalog
                    .products
                    .values()
                    .any(|product| product.failure.is_some() || product.skipped.is_some())
        }) {
            "failed"
        } else {
            "succeeded"
        }
    }
}
