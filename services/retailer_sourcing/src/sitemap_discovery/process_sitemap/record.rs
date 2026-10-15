//! PostgreSQL representations and explicit conversions for this slice.
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use super::super::schema::{
    processed_sitemaps_data, raw_sitemap_documents_data, sitemaps_processed, sitemaps_raw,
};
use super::entity::{ProcessedSitemap, RawSitemapDocument};
use super::store::StoreError;

#[derive(Queryable, Selectable)]
#[diesel(table_name = sitemaps_raw)]
pub(super) struct RawSitemapDocumentPgRecord {
    pub url: String,
    pub last_modified: Option<DateTime<Utc>>,
    pub body_size: i32,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = raw_sitemap_documents_data)]
pub(super) struct RawSitemapDocumentDataPgRecord {
    pub body: String,
}

impl RawSitemapDocumentPgRecord {
    pub fn into_entity(
        self,
        data: RawSitemapDocumentDataPgRecord,
    ) -> Result<RawSitemapDocument, StoreError> {
        let record = self;
        Ok(RawSitemapDocument {
            url: record.url,
            last_modified: record.last_modified,
            body: data.body,
            body_size: usize::try_from(record.body_size)
                .map_err(|_| StoreError::InvalidBodySize(record.body_size))?,
        })
    }
}

#[derive(Insertable)]
#[diesel(table_name = sitemaps_processed)]
pub(super) struct ProcessedSitemapPgRecord {
    pub id: Uuid,
    pub retrieval_id: Uuid,
    pub retailer_code: String,
    pub url_count: i32,
    pub document_size: i32,
    pub processed_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = processed_sitemaps_data)]
pub(super) struct ProcessedSitemapDataPgRecord {
    pub id: Uuid,
    pub document: serde_json::Value,
}

impl TryFrom<&ProcessedSitemap> for (ProcessedSitemapPgRecord, ProcessedSitemapDataPgRecord) {
    type Error = StoreError;
    fn try_from(entity: &ProcessedSitemap) -> Result<Self, Self::Error> {
        let document = serde_json::to_value(entity.document())?;
        let record = ProcessedSitemapPgRecord {
            id: entity.id,
            retrieval_id: entity.retrieval_id,
            retailer_code: entity.retailer_code.as_str(),
            url_count: stored_count(entity.url_count())?,
            document_size: i32::try_from(document.to_string().len())
                .map_err(|_| StoreError::DocumentTooLarge)?,
            processed_at: entity.processed_at,
        };
        Ok((
            record,
            ProcessedSitemapDataPgRecord {
                id: entity.id,
                document,
            },
        ))
    }
}

fn stored_count(count: usize) -> Result<i32, StoreError> {
    i32::try_from(count).map_err(|_| StoreError::TooManyUrls(count))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bodul_shared::retailer::RetailerCode;
    use sitemap::SitemapDocument;

    #[test]
    fn maps_domain_tree_and_timestamp_without_changing_ids() {
        let entity = ProcessedSitemap::new(
            Uuid::now_v7(),
            RetailerCode::UgreenCa,
            SitemapDocument::default(),
        );
        let (record, data): (ProcessedSitemapPgRecord, ProcessedSitemapDataPgRecord) =
            (&entity).try_into().unwrap();
        assert_eq!(record.id, entity.id);
        assert_eq!(data.id, entity.id);
        assert_eq!(record.retrieval_id, entity.retrieval_id);
        assert_eq!(record.retailer_code, "ugreenca");
        assert_eq!(record.processed_at, entity.processed_at);
        assert_eq!(record.url_count, 0);
        assert_eq!(
            data.document,
            serde_json::to_value(entity.document()).unwrap()
        );
        assert_eq!(
            record.document_size as usize,
            data.document.to_string().len()
        );
        assert_eq!(stored_count(i32::MAX as usize).unwrap(), i32::MAX);
        assert!(matches!(
            stored_count(i32::MAX as usize + 1),
            Err(StoreError::TooManyUrls(_))
        ));
    }

    #[test]
    fn reads_raw_entity_and_rejects_negative_stored_sizes() {
        let modified = Utc::now();
        for size in [0, 7, -1] {
            let record = RawSitemapDocumentPgRecord {
                url: "https://shop.test/ž".into(),
                last_modified: Some(modified),
                body_size: size,
            };
            let data = RawSitemapDocumentDataPgRecord {
                body: "ž 🔎".into(),
            };
            let result = record.into_entity(data);
            if size < 0 {
                assert!(matches!(result, Err(StoreError::InvalidBodySize(-1))));
            } else {
                let entity = result.unwrap();
                assert_eq!(entity.body_size, size as usize);
                assert_eq!(entity.body, "ž 🔎");
                assert_eq!(entity.url, "https://shop.test/ž");
                assert_eq!(entity.last_modified, Some(modified));
            }
        }
    }
}
