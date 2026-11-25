use super::SitemapHtmlDetailsInitialFillSaga;
use super::entity::StartingCommand;
use crate::platform::bus::{EventCommand, EventContext};
use crate::saga::prelude::SagaId;
use crate::steps::prelude::{SitemapConsolidated, SitemapProcessed, SitemapRetrieved};
use serde_json::{Value, from_str, from_value, json};
use shared::retailer::RetailerCode;
use uuid::Uuid;

fn assert_command(command: StartingCommand, name: &str, payload: Value) {
    let id = Uuid::now_v7();
    let context = EventContext {
        event_id: Uuid::now_v7(),
        correlation_id: Some(id),
        causation_id: None,
        source: None,
    };
    let envelope = command.into_envelope(id, "test", &context).unwrap();
    assert_eq!(envelope.command.command_type, name);
    assert_eq!(
        from_str::<Value>(&envelope.command.payload).unwrap(),
        payload
    );
}

#[test]
fn advances_sitemaps_and_discovers_only_when_products_exist() {
    let saga = SitemapHtmlDetailsInitialFillSaga::new(
        SagaId::new(),
        Uuid::now_v7(),
        RetailerCode::AnkerAu,
    );
    let retrieval = Uuid::now_v7();
    let processed = Uuid::now_v7();
    let consolidated = Uuid::now_v7();
    let retrieved = SitemapRetrieved {
        retrieval_id: retrieval,
        retailer_code: RetailerCode::AnkerAu,
        document_count: 2,
    };
    assert_command(
        saga.on_sitemap_retrieved(retrieved),
        "ProcessSitemap",
        json!({"retrieval_id":retrieval,"retailer_code":"ankerau"}),
    );
    let parsed = SitemapProcessed {
        retrieval_id: retrieval,
        processed_sitemap_id: processed,
        retailer_code: RetailerCode::AnkerAu,
        url_count: 3,
    };
    assert_command(
        saga.on_sitemap_processed(parsed),
        "ConsolidateSitemap",
        json!({"processed_sitemap_id":processed}),
    );
    let mut classified = SitemapConsolidated {
        retrieval_id: retrieval,
        processed_sitemap_id: processed,
        consolidated_sitemap_id: consolidated,
        retailer_code: RetailerCode::AnkerAu,
        product_count: 2,
        catalog_count: 0,
        content_count: 1,
        not_interested_count: 0,
        unknown_count: 0,
    };
    assert_command(
        saga.on_sitemap_consolidated(classified.clone())
            .pop()
            .unwrap(),
        "DiscoverOffers",
        json!({"consolidated_sitemap_id":consolidated,"retailer_code":"ankerau"}),
    );
    classified.product_count = 0;
    assert!(saga.on_sitemap_consolidated(classified).is_empty());
}

#[test]
fn advances_individual_offers_with_current_and_legacy_artifact_metadata() {
    let saga = SitemapHtmlDetailsInitialFillSaga::new(
        SagaId::new(),
        Uuid::now_v7(),
        RetailerCode::AnkerAu,
    );
    let sitemap = Uuid::now_v7();
    for item in ["a", "b"] {
        let mut offer = json!({
            "consolidated_sitemap_id": sitemap,
            "offer_id": Uuid::now_v7(),
            "url": format!("https://shop.test/{item}"),
            "retailer_code": "ankerau",
        });
        let discovered = from_value(offer.clone()).unwrap();
        assert_command(
            saga.on_offer_discovered(discovered),
            "DownloadOfferPage",
            offer.clone(),
        );

        offer["scraped_at"] = Value::Null;
        if item == "a" {
            offer["scraped_at"] = json!("2026-11-25T12:00:00Z");
            offer["storage"] = json!({
                "content_size": 123,
                "external_id": Uuid::now_v7(),
                "object_storage_id": Uuid::now_v7(),
            });
        }
        let downloaded = from_value(offer.clone()).unwrap();
        assert_command(
            saga.on_offer_page_was_downloaded(downloaded),
            "ExtractOfferPage",
            offer.clone(),
        );
        let extracted = from_value(offer.clone()).unwrap();
        assert_command(
            saga.on_offer_page_was_extracted(extracted),
            "NormalizeOffer",
            offer.clone(),
        );
        let normalized = from_value(offer.clone()).unwrap();
        assert_command(
            saga.on_offer_was_normalized(normalized),
            "StructuralizeOffer",
            offer,
        );
    }
}

#[test]
fn sitemap_and_offer_page_commands_use_their_own_wire_type_and_event_metadata() {
    let saga_id = Uuid::now_v7();
    let context = EventContext {
        event_id: Uuid::now_v7(),
        correlation_id: Some(saga_id),
        causation_id: None,
        source: None,
    };
    let check = |command: StartingCommand, name: &str, payload: &Value| {
        let envelope = command
            .into_envelope(saga_id, "TestEvent", &context)
            .unwrap();
        assert_eq!(envelope.command.command_type, name);
        assert_eq!(
            from_str::<Value>(&envelope.command.payload).unwrap(),
            *payload
        );
        let metadata = envelope.metadata.unwrap();
        assert_eq!(metadata.command_id.get_version_num(), 7);
        assert_eq!(metadata.correlation_id, Some(saga_id));
        assert_eq!(metadata.causation_id, Some(context.event_id));
        assert_eq!(metadata.source.as_deref(), Some("event:TestEvent"));
    };

    let retrieval = json!({"retrieval_id": Uuid::now_v7(), "retailer_code": "ankerau"});
    check(
        StartingCommand::RetrieveSitemap(from_value(retrieval.clone()).unwrap()),
        "RetrieveSitemap",
        &retrieval,
    );
    check(
        StartingCommand::ProcessSitemap(from_value(retrieval.clone()).unwrap()),
        "ProcessSitemap",
        &retrieval,
    );

    let processed = json!({"processed_sitemap_id": Uuid::now_v7()});
    check(
        StartingCommand::ConsolidateSitemap(from_value(processed.clone()).unwrap()),
        "ConsolidateSitemap",
        &processed,
    );

    let discovery = json!({
        "consolidated_sitemap_id": Uuid::now_v7(), "retailer_code": "ankerau",
    });
    check(
        StartingCommand::DiscoverOffers(from_value(discovery.clone()).unwrap()),
        "DiscoverOffers",
        &discovery,
    );

    let download = json!({
        "offer_id": Uuid::now_v7(), "consolidated_sitemap_id": Uuid::now_v7(),
        "retailer_code": "ankerau", "url": "https://example.com/item",
    });
    check(
        StartingCommand::DownloadOfferPage(from_value(download.clone()).unwrap()),
        "DownloadOfferPage",
        &download,
    );

    let offer = json!({
        "offer_id": Uuid::now_v7(), "consolidated_sitemap_id": Uuid::now_v7(),
        "retailer_code": "ankerau", "url": "https://example.com/item", "scraped_at": null,
    });
    check(
        StartingCommand::ExtractOfferPage(from_value(offer.clone()).unwrap()),
        "ExtractOfferPage",
        &offer,
    );
    check(
        StartingCommand::NormalizeOffer(from_value(offer.clone()).unwrap()),
        "NormalizeOffer",
        &offer,
    );
    check(
        StartingCommand::StructuralizeOffer(from_value(offer.clone()).unwrap()),
        "StructuralizeOffer",
        &offer,
    );
}
