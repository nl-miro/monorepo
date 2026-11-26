use crate::platform::object_storage::OfferStorageMetadata;
use crate::saga::initial_fill::shopify_graphql::saga::StartingCommand;
use crate::steps::prelude::{
    AcquireStorefrontToken, ExtractOfferDetails, FetchOfferDetails, FetchProductCatalogIdentifiers,
    SourceType, StructuralizeOfferDetails,
};
use chrono::{DateTime, Utc};
use retailer_sauce::initial_fill::RequestedContext;

use retailer_sauce::prelude::{StorefrontConfig, StorefrontToken};
use shared::prelude::RetailerCode;

pub fn build_acquire_storefront_token(cfg: StorefrontConfig) -> StartingCommand {
    let cmd = AcquireStorefrontToken::new(cfg.retailer_code(), cfg.storefront_url().into());

    StartingCommand::Token(cmd)
}

pub fn build_fetch_identifiers(
    cfg: StorefrontConfig,
    token: Option<StorefrontToken>,
) -> Result<Vec<StartingCommand>, std::io::Error> {
    let mut commands = Vec::new();

    for context in cfg.initial_fill_contexts().unwrap() {
        let cmd = FetchProductCatalogIdentifiers::new(cfg.retailer_code(), context, token.clone());

        let wrapped = StartingCommand::CatalogProductIds(cmd);

        commands.push(wrapped);
    }

    Ok(commands)
}

pub fn build_fetch_offer_details(
    retailer_code: RetailerCode,
    product_ids: Vec<String>,
    context: Option<RequestedContext>,
    token: Option<StorefrontToken>,
) -> Result<Vec<StartingCommand>, std::io::Error> {
    let mut commands = vec![];

    for product_id in product_ids {
        let source = SourceType::GraphQl { product_id };
        let cmd = FetchOfferDetails::new(
            retailer_code.clone(),
            context.clone(),
            token.clone(),
            source,
        );

        let wrapped = StartingCommand::FetchOfferDetails(cmd);

        commands.push(wrapped);
    }

    Ok(commands)
}

pub fn build_extract_offer_details(
    retailer_code: RetailerCode,
    source: SourceType,
    storage: OfferStorageMetadata,
    fetched_at: DateTime<Utc>,
) -> StartingCommand {
    let cmd = ExtractOfferDetails {
        retailer_code,
        source,
        storage,
        fetched_at,
    };

    StartingCommand::ExtractOfferDetails(cmd)
}

pub fn build_structuralize_offer_details(
    retailer_code: RetailerCode,
    source: SourceType,
    storage: OfferStorageMetadata,
    fetched_at: DateTime<Utc>,
) -> StartingCommand {
    let cmd = StructuralizeOfferDetails {
        retailer_code,
        source,
        storage,
        fetched_at,
    };

    StartingCommand::StructuralizeOfferDetails(cmd)
}
