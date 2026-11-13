//! Extracted collection and its product summaries.

use super::{Image, Seo};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    // #[serde(rename = "__typename")]
    // pub type_name: String,
    pub id: String,
    pub handle: String,
    pub url: Option<String>,
    pub title: String,
    pub description: String,
    pub description_html: String,
    pub image: Option<Image>,
    pub seo: Seo,
    pub products: Vec<ProductSummary>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductSummary {
    // #[serde(rename = "__typename")]
    // pub type_name: String,
    pub id: String,
    pub handle: String,
    pub title: String,
}
