//! PostgreSQL representations and conversions for retrieved documents.
use chrono::{DateTime, Utc};
use diesel::Insertable;
use uuid::Uuid;

use super::super::schema::{raw_sitemap_documents_data, sitemaps_raw};
use super::entity::RetrievedSitemapDocument;
use super::store::StoreError;

#[derive(Insertable)]
#[diesel(table_name = sitemaps_raw)]
pub(super) struct RawSitemapDocumentPgRecord<'a> {
    pub id: Uuid,
    pub retrieval_id: Uuid,
    pub url: &'a str,
    pub last_modified: Option<DateTime<Utc>>,
    pub body_size: i32,
    pub fetched_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = raw_sitemap_documents_data)]
pub(super) struct RawSitemapDocumentDataPgRecord<'a> {
    pub id: Uuid,
    pub body: &'a str,
}

impl<'a> TryFrom<&'a RetrievedSitemapDocument>
    for (
        RawSitemapDocumentPgRecord<'a>,
        RawSitemapDocumentDataPgRecord<'a>,
    )
{
    type Error = StoreError;
    fn try_from(entity: &'a RetrievedSitemapDocument) -> Result<Self, Self::Error> {
        let document = &entity.document;
        Ok((
            RawSitemapDocumentPgRecord {
                id: entity.id,
                retrieval_id: entity.retrieval_id,
                url: &document.url,
                last_modified: document.last_modified,
                body_size: i32::try_from(document.body_size)
                    .map_err(|_| StoreError::BodyTooLarge(document.body_size))?,
                fetched_at: entity.fetched_at,
            },
            RawSitemapDocumentDataPgRecord {
                id: entity.id,
                body: &document.body,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sitemap::RawSitemapDocument;

    #[test]
    fn preserves_batch_identity_timestamps_and_unicode_and_checks_sizes() {
        let body = "ž 🔎";
        let modified = Utc::now();
        let retrieval_id = Uuid::now_v7();
        let mut entities = RetrievedSitemapDocument::from_documents(
            retrieval_id,
            vec![
                RawSitemapDocument {
                    url: "https://shop.test/sitemap.xml".into(),
                    last_modified: Some(modified),
                    body: body.into(),
                    body_size: body.len(),
                };
                2
            ],
        );
        assert_ne!(entities[0].id, entities[1].id);
        assert_eq!(entities[0].fetched_at, entities[1].fetched_at);
        for entity in &entities {
            let (record, data): (
                RawSitemapDocumentPgRecord<'_>,
                RawSitemapDocumentDataPgRecord<'_>,
            ) = entity.try_into().unwrap();
            assert_eq!(record.id, entity.id);
            assert_eq!(data.id, entity.id);
            assert_eq!(record.retrieval_id, retrieval_id);
            assert_eq!(record.fetched_at, entity.fetched_at);
            assert_eq!(record.last_modified, Some(modified));
            assert_eq!(record.body_size as usize, body.len());
            assert_eq!(record.url, entity.document.url);
            assert_eq!(data.body, body);
        }
        entities[0].document.body_size = i32::MAX as usize + 1;
        assert!(matches!(
            <(
                RawSitemapDocumentPgRecord<'_>,
                RawSitemapDocumentDataPgRecord<'_>
            )>::try_from(&entities[0]),
            Err(StoreError::BodyTooLarge(_))
        ));
        assert!(RetrievedSitemapDocument::from_documents(retrieval_id, vec![]).is_empty());
    }
}
