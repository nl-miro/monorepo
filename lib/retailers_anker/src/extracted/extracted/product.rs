//! Extracted product and its grouped attributes.

use super::media::Media;
use chrono::{DateTime, Utc};
use money::Money;
use serde::{Deserialize, Serialize};
use serde_json::Number;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductIdentifiers {
    pub id: String,
    pub handle: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductTaxonomy {
    pub product_type: String,
    pub category: Option<Category>,
    pub collections: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductContent {
    pub vendor: String,
    pub featured_image: Option<String>,
    pub images: Vec<String>,
    pub media: Vec<String>,
    pub description: String,
    pub description_html: String,
    pub title: String,
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub identifiers: ProductIdentifiers,
    pub taxonomy: ProductTaxonomy,
    pub content: ProductContent,
    pub total_inventory: Option<i64>,
    /// Reference nodes expanded by the saved-catalog collector.
    #[serde(rename = "_expandedReferences")]
    pub expanded_references: Option<std::collections::BTreeMap<String, ExpandedVariantReference>>,
    /// Present in this retailer's older full-catalog captures.
    pub metafields: Option<Vec<Option<Metafield>>>,
    // #[serde(rename = "__typename")]
    // pub type_name: String,
    pub available_for_sale: bool,
    // pub compare_at_price_range: PriceRange,
    // pub encoded_variant_availability: Option<String>,
    // pub encoded_variant_existence: Option<String>,
    pub is_gift_card: bool,
    pub retailer_offer_url: Option<String>,
    pub options: Vec<ProductOption>,
    // pub price_range: PriceRange,
    pub requires_selling_plan: bool,
    pub variants: Vec<Variant>,
    // pub variants_count: Option<Count>,
    pub created_at: DateTime<Utc>,
    pub published_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    // #[serde(rename = "__typename")]
    // pub type_name: String,
    pub ancestors: Vec<CategoryAncestor>,
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryAncestor {
    // #[serde(rename = "__typename")]
    // pub type_name: String,
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductOption {
    // #[serde(rename = "__typename")]
    // pub type_name: String,
    pub id: String,
    pub name: String,
    pub option_values: Vec<OptionValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionValue {
    // #[serde(rename = "__typename")]
    // pub type_name: String,
    pub id: String,
    pub name: String,
    pub swatch: Option<Swatch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Swatch {
    // #[serde(rename = "__typename")]
    // pub type_name: String,
    pub color: Option<String>,
    pub image: Option<Media>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantIdentifiers {
    pub id: String,
    pub sku: Option<String>,
    pub barcode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantSpecials {
    pub components: Vec<Component>,
    pub grouped_by: Vec<VariantSummary>,
    pub quantity_price_breaks: Vec<QuantityPriceBreak>,
    pub quantity_rule: QuantityRule,
    pub requires_components: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantFlags {
    pub available_for_sale: bool,
    pub currently_not_in_stock: bool,
    pub taxable: bool,
    pub requires_shipping: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Pricing {
    Unknown,
    Standard(Money),
    /// Current price followed by the comparison price.
    Discounted(Money, Money),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variant {
    pub identifiers: VariantIdentifiers,
    pub flags: VariantFlags,
    pub specials: VariantSpecials,
    pub title: String,
    pub pricing: Pricing,
    pub shop_pay_installments_pricing: Option<()>,
    pub quantity_available: Option<i64>,
    pub metafields: Option<Vec<Option<Metafield>>>,
    // #[serde(rename = "__typename")]
    // pub type_name: String,
    pub image: Option<String>,
    pub selected_options: Vec<SelectedOption>,
    pub store_availability: Vec<StoreAvailability>,
    pub unit_price: Option<Money>,
    pub unit_price_measurement: Option<UnitPriceMeasurement>,
    pub weight: Option<Number>,
    pub weight_unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantSummary {
    // #[serde(rename = "__typename")]
    // pub type_name: String,
    pub id: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    // #[serde(rename = "__typename")]
    // pub type_name: String,
    pub product_variant: VariantSummary,
    pub quantity: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantityPriceBreak {
    // #[serde(rename = "__typename")]
    // pub type_name: String,
    pub minimum_quantity: i64,
    pub price: Money,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantityRule {
    // #[serde(rename = "__typename")]
    // pub type_name: String,
    pub increment: i64,
    pub maximum: Option<i64>,
    pub minimum: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectedOption {
    // #[serde(rename = "__typename")]
    // pub type_name: String,
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitPriceMeasurement {
    // #[serde(rename = "__typename")]
    // pub type_name: String,
    pub measured_type: Option<String>,
    pub quantity_unit: Option<String>,
    pub quantity_value: Number,
    pub reference_unit: Option<String>,
    pub reference_value: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreAvailability {
    // #[serde(rename = "__typename")]
    // pub type_name: String,
    pub available: bool,
    pub location: Location,
    pub pick_up_time: String,
    pub quantity_available: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    // #[serde(rename = "__typename")]
    // pub type_name: String,
    pub address: Address,
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    // #[serde(rename = "__typename")]
    // pub type_name: String,
    pub address1: Option<String>,
    pub address2: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub formatted: Vec<String>,
    pub latitude: Option<Number>,
    pub longitude: Option<Number>,
    pub phone: Option<String>,
    pub province: Option<String>,
    pub province_code: Option<String>,
    pub zip: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metafield {
    // #[serde(rename = "__typename")]
    // pub type_name: String,
    pub created_at: DateTime<Utc>,
    pub description: Option<String>,
    pub id: String,
    pub key: String,
    pub list: bool,
    pub namespace: String,
    pub reference: Option<()>,
    pub references: Option<()>,
    #[serde(rename = "type")]
    pub value_type: String,
    pub updated_at: DateTime<Utc>,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpandedVariantReference {
    // #[serde(rename = "__typename")]
    // pub type_name: String,
    pub id: String,
    pub metafields: Vec<Option<Metafield>>,
}
