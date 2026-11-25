//! Explicit live smoke test: acquire a public token, read the whole catalog, store one complete product.
use super::acquire_storefront_token::AcquireStorefrontToken;
use super::acquire_storefront_token::Handler as AcquireStorefrontTokenHandler;
use super::{offer_details as details, product_catalog_ids as catalog};
use crate::platform::{bus::Handle, http, object_storage::ObjectStorage};
use catalog::FetchProductCatalogIdentifiers;
use catalog::Handler as CatalogHandler;
use details::BUCKET;
use details::Handler as OfferDetailsHandler;
use details::{FetchOfferDetails, SourceType};
use http::Client;
use http::OfferClient;
use retailer_data_extractor::get_storefront_config;
use serde_json::Value;
use serde_json::from_str;
use shared::retailer::RetailerCode;
use std::collections::HashSet;
use std::env::var;
use tokio::runtime::Runtime;

#[test]
#[ignore = "requires live network and OBJECT_STORAGE_URL/TOKEN; run make test-initial-fill-live"]
fn anker_au_token_catalog_and_one_complete_offer() {
    run(RetailerCode::AnkerAu);
}

#[test]
#[ignore = "requires live network and OBJECT_STORAGE_URL/TOKEN; run make test-initial-fill-live"]
fn anker_ca_english_token_catalog_and_one_complete_offer() {
    run(RetailerCode::AnkerCa);
}

#[test]
#[ignore = "requires live network and OBJECT_STORAGE_URL/TOKEN; run make test-initial-fill-live"]
fn remaining_anker_markets_catalog_and_one_complete_offer() {
    for retailer in RetailerCode::ALL {
        if !matches!(retailer, RetailerCode::AnkerAu | RetailerCode::AnkerCa)
            && get_storefront_config(retailer).is_some()
        {
            run(retailer);
        }
    }
}

fn run(retailer: RetailerCode) {
    assert_eq!(
        var("INITIAL_FILL_LIVE").as_deref(),
        Ok("1"),
        "set INITIAL_FILL_LIVE=1 explicitly"
    );
    let runtime = Runtime::new().unwrap();
    let objects = ObjectStorage::from_env(runtime.handle().clone()).unwrap();
    let config = get_storefront_config(retailer).unwrap();
    let command = if config.requires_token() {
        let event = AcquireStorefrontTokenHandler { fetcher: Client }
            .handle(AcquireStorefrontToken {
                retailer_code: retailer,
                url: config.storefront_url().into(),
            })
            .unwrap()
            .remove(0);
        FetchProductCatalogIdentifiers {
            retailer_code: retailer,
            context: Some(config.initial_fill_contexts().unwrap().remove(0)),
            storefront_token: event.storefront_token,
        }
    } else {
        FetchProductCatalogIdentifiers {
            context: None,
            retailer_code: retailer,
            storefront_token: None,
        }
    };
    let http = OfferClient::configured().unwrap();
    let handler = CatalogHandler {
        fetcher: http.clone(),
    };
    let mut events = handler.handle(command).unwrap();
    assert_eq!(events.len(), 1);
    let batch = events.remove(0);
    assert!(!batch.product_ids.is_empty());
    let unique: HashSet<_> = batch.product_ids.iter().collect();
    assert_eq!(unique.len(), batch.product_ids.len());
    let command = FetchOfferDetails {
        retailer_code: retailer,
        context: batch.context,
        storefront_token: batch.storefront_token,
        source: SourceType::GraphQl {
            product_id: batch.product_ids[0].clone(),
        },
    };
    let event = OfferDetailsHandler {
        fetcher: http,
        store: objects.clone(),
    }
    .handle(command)
    .unwrap()
    .remove(0);
    let artifact = objects.get_offer(BUCKET, &event.storage).unwrap();
    let artifact: Value = from_str(&artifact).unwrap();
    assert_eq!(artifact["schema_version"], 1);
    assert!(!artifact["pages"].as_array().unwrap().is_empty());
}
