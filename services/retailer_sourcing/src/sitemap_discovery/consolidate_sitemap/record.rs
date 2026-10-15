//! PostgreSQL representations and conversions, private to this slice.

use bodul_shared::retailer::RetailerCode;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde_json::{Value, json};
use uuid::Uuid;

use super::super::schema::{
    consolidated_sitemaps_data, processed_sitemaps_data, sitemaps_consolidated, sitemaps_processed,
};
use super::entity::{ConsolidatedSitemap, ProcessedSitemap};
use super::store::StoreError;

#[derive(Queryable, Selectable)]
#[diesel(table_name = sitemaps_processed)]
pub(super) struct ProcessedSitemapPgRecord {
    pub id: Uuid,
    pub retrieval_id: Uuid,
    pub retailer_code: String,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = processed_sitemaps_data)]
pub(super) struct ProcessedSitemapDataPgRecord {
    pub document: Value,
}

impl TryFrom<(ProcessedSitemapPgRecord, ProcessedSitemapDataPgRecord)> for ProcessedSitemap {
    type Error = StoreError;

    fn try_from(
        (record, data): (ProcessedSitemapPgRecord, ProcessedSitemapDataPgRecord),
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            id: record.id,
            retrieval_id: record.retrieval_id,
            retailer_code: RetailerCode::try_from(record.retailer_code.as_str())?,
            document: serde_json::from_value(data.document)?,
        })
    }
}

#[derive(Insertable)]
#[diesel(table_name = sitemaps_consolidated)]
pub(super) struct ConsolidatedSitemapPgRecord {
    pub id: Uuid,
    pub references: Value,
    pub stats: Value,
    pub storage: Value,
    pub consolidated_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = consolidated_sitemaps_data)]
pub(super) struct ConsolidatedSitemapDataPgRecord {
    pub id: Uuid,
    pub content: Value,
}

impl TryFrom<&ConsolidatedSitemap>
    for (ConsolidatedSitemapPgRecord, ConsolidatedSitemapDataPgRecord)
{
    type Error = StoreError;

    fn try_from(entity: &ConsolidatedSitemap) -> Result<Self, Self::Error> {
        let content = serde_json::to_value(entity.links())?;
        let content_size = i32::try_from(content.to_string().len()).map_err(|_| {
            StoreError::Unexpected("consolidated sitemap content is too large for storage".into())
        })?;
        let record = ConsolidatedSitemapPgRecord {
            id: entity.id,
            references: json!({
                "processed_sitemap_id": entity.processed_sitemap_id,
                "retrieval_id": entity.retrieval_id,
                "retailer_code": entity.retailer_code.as_str(),
            }),
            stats: json!({
                "product_count": stored_count(entity.product_count())?,
                "catalog_count": stored_count(entity.catalog_count())?,
                "content_count": stored_count(entity.content_count())?,
                "not_interested_count": stored_count(entity.not_interested_count())?,
                "unknown_count": stored_count(entity.unknown_count())?,
            }),
            storage: json!({"content_size": content_size}),
            consolidated_at: Utc::now(),
        };
        Ok((
            record,
            ConsolidatedSitemapDataPgRecord {
                id: entity.id,
                content,
            },
        ))
    }
}

fn stored_count(count: usize) -> Result<i32, StoreError> {
    i32::try_from(count).map_err(|_| {
        StoreError::Unexpected(format!(
            "consolidated sitemap count is too large for storage: {count}"
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sitemap::{GroupedLinks, SitemapDocument};

    #[test]
    fn maps_entity_counts_and_payload_to_records() {
        let processed = ProcessedSitemap {
            id: Uuid::now_v7(),
            retrieval_id: Uuid::now_v7(),
            retailer_code: RetailerCode::UgreenCa,
            document: SitemapDocument::default(),
        };
        let entity = ConsolidatedSitemap::new(
            &processed,
            GroupedLinks {
                product: vec!["ž 🔎".into()],
                ..Default::default()
            },
        );
        let (record, data) =
            <(ConsolidatedSitemapPgRecord, ConsolidatedSitemapDataPgRecord)>::try_from(&entity)
                .unwrap();
        assert_eq!(
            record.stats,
            json!({"product_count":1,"catalog_count":0,"content_count":0,"not_interested_count":0,"unknown_count":0})
        );
        assert_eq!(record.id, entity.id);
        assert_eq!(
            record.references,
            json!({
                "processed_sitemap_id": processed.id,
                "retrieval_id": processed.retrieval_id,
                "retailer_code": processed.retailer_code.as_str(),
            })
        );
        assert_eq!(data.id, entity.id);
        assert_eq!(data.content, serde_json::to_value(entity.links()).unwrap());
        assert_eq!(
            record.storage,
            json!({"content_size": data.content.to_string().len()})
        );
    }

    #[test]
    fn keeps_counts_representable_in_the_previous_schema() {
        assert_eq!(stored_count(0).unwrap(), 0);
        assert_eq!(stored_count(i32::MAX as usize).unwrap(), i32::MAX);
        assert!(stored_count(i32::MAX as usize + 1).is_err());
    }

    fn records(
        code: &str,
        document: Value,
    ) -> (ProcessedSitemapPgRecord, ProcessedSitemapDataPgRecord) {
        (
            ProcessedSitemapPgRecord {
                id: Uuid::now_v7(),
                retrieval_id: Uuid::now_v7(),
                retailer_code: code.into(),
            },
            ProcessedSitemapDataPgRecord { document },
        )
    }

    #[test]
    fn converts_stored_records_and_rejects_invalid_values() {
        let document = serde_json::to_value(SitemapDocument::default()).unwrap();
        let (record, data) = records("ugreenca", document.clone());
        let id = record.id;
        let retrieval_id = record.retrieval_id;
        let entity = ProcessedSitemap::try_from((record, data)).unwrap();
        assert_eq!(entity.id, id);
        assert_eq!(entity.retrieval_id, retrieval_id);
        assert_eq!(entity.retailer_code, RetailerCode::UgreenCa);
        assert_eq!(serde_json::to_value(entity.document).unwrap(), document);
        assert!(matches!(
            ProcessedSitemap::try_from(records("invalid", document)),
            Err(StoreError::InvalidRetailerCode(_))
        ));
        assert!(matches!(
            ProcessedSitemap::try_from(records("ugreenca", Value::Null)),
            Err(StoreError::Serialization(_))
        ));
    }
}
