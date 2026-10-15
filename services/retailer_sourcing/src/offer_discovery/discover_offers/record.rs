//! PostgreSQL projection and conversion for discovery's sitemap input.
use super::entity::ConsolidatedSitemap;
use super::store::StoreError;
use diesel::prelude::*;
use serde::Deserialize;
use uuid::Uuid;

// The columns this slice reads from sitemap discovery's table. A consolidated
// sitemap's content is keyed by the sitemap's id.
diesel::table! {
    consolidated_sitemaps_data (id) {
        id -> Uuid,
        content -> Jsonb,
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = consolidated_sitemaps_data)]
pub(super) struct ConsolidatedSitemapPgRecord {
    pub id: Uuid,
    pub content: serde_json::Value,
}

#[derive(Deserialize)]
struct ConsolidatedContent {
    product: Vec<String>,
}

impl TryFrom<ConsolidatedSitemapPgRecord> for ConsolidatedSitemap {
    type Error = StoreError;
    fn try_from(record: ConsolidatedSitemapPgRecord) -> Result<Self, Self::Error> {
        let content: ConsolidatedContent = serde_json::from_value(record.content)?;
        Ok(Self {
            id: record.id,
            product_urls: content.product,
        })
    }
}
