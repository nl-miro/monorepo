//! Extracted offer-details model owned by minisforumau, copied one-to-one from `graphql::raw`.
//! Converted from raw via `raw_to_extracted`; reshape here, not in raw.
//! Monetary values use currency-aware integer minor units via `money::Money`;
//! other numeric scalars retain their JSON representation. See fixtures/README.md.

pub mod graphql;
pub mod not_interested;

use chrono::{DateTime, Utc};
pub use money::Money;
use serde::{Deserialize, Serialize};
use serde_json::Number;

/// Extracted product data without raw artifact metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(try_from = "ArtifactWire")]
pub struct ExtractedOfferDetails {
    // pub schema_version: u32,
    pub acquired_at: Option<DateTime<Utc>>,
    pub context: Option<retailer_sauce::shopify::markets::MarketContext>,
    pub product: Product,
    pub collections: Vec<Collection>,
    pub images: Vec<Image>,
    pub media: Vec<Media>,
    // Original request/response evidence belongs to the raw artifact.
    // pub pages: Vec<RecordedPage>,
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct IsoCode {
//     pub iso_code: String,
// }

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub identifiers: ProductIdentifiers,
    pub taxonomy: ProductTaxonomy,
    pub content: ProductContent,
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
    pub selling_plan_groups: Vec<SellingPlanGroup>,
    pub seo: Seo,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Seo {
    // #[serde(rename = "__typename")]
    // pub type_name: String,
    pub description: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    // #[serde(rename = "__typename")]
    // pub type_name: String,
    pub id: Option<String>,
    pub thumbhash: Option<String>,
    pub url: String,
    pub alt_text: Option<String>,
    pub width: Option<i64>,
    pub height: Option<i64>,
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
pub struct MediaPresentation {
    // #[serde(rename = "__typename")]
    // pub type_name: String,
    pub id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoSource {
    // #[serde(rename = "__typename")]
    // pub type_name: String,
    pub format: String,
    pub height: i64,
    pub mime_type: String,
    pub url: String,
    pub width: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSource {
    // #[serde(rename = "__typename")]
    // pub type_name: String,
    pub filesize: i64,
    pub format: String,
    pub mime_type: String,
    pub url: String,
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
    pub selling_plan_allocations: Vec<SellingPlanAllocation>,
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
pub struct SellingPlanAllocation {
    // #[serde(rename = "__typename")]
    // pub type_name: String,
    pub checkout_charge_amount: Money,
    pub price_adjustments: Vec<AllocationPriceAdjustment>,
    pub remaining_balance_charge_amount: Money,
    pub selling_plan: SellingPlan,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationPriceAdjustment {
    // #[serde(rename = "__typename")]
    // pub type_name: String,
    pub compare_at_price: Money,
    pub per_delivery_price: Money,
    pub price: Money,
    pub unit_price: Option<Money>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SellingPlanGroup {
    // #[serde(rename = "__typename")]
    // pub type_name: String,
    pub app_name: Option<String>,
    pub name: String,
    pub options: Vec<SellingPlanGroupOption>,
    pub selling_plans: Vec<SellingPlan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SellingPlanGroupOption {
    // #[serde(rename = "__typename")]
    // pub type_name: String,
    pub name: String,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SellingPlan {
    // #[serde(rename = "__typename")]
    // pub type_name: String,
    pub billing_policy: Option<BillingPolicy>,
    pub checkout_charge: CheckoutCharge,
    pub delivery_policy: Option<DeliveryPolicy>,
    pub description: Option<String>,
    pub id: String,
    pub name: String,
    pub options: Vec<SellingPlanOption>,
    pub price_adjustments: Vec<PlanPriceAdjustment>,
    pub recurring_deliveries: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SellingPlanOption {
    // #[serde(rename = "__typename")]
    // pub type_name: String,
    pub name: Option<String>,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutCharge {
    // #[serde(rename = "__typename")]
    // pub type_name: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub value: CheckoutChargeValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanPriceAdjustment {
    // #[serde(rename = "__typename")]
    // pub type_name: String,
    pub adjustment_value: PriceAdjustmentValue,
    pub order_count: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "__typename")]
pub enum Media {
    MediaImage {
        alt: Option<String>,
        id: Option<String>,
        media_content_type: String,
        presentation: Option<MediaPresentation>,
        preview_image: Option<Image>,
        image: Option<Image>,
    },
    ExternalVideo {
        alt: Option<String>,
        id: Option<String>,
        media_content_type: String,
        presentation: Option<MediaPresentation>,
        preview_image: Option<Image>,
        embed_url: String,
        host: String,
        origin_url: String,
    },
    Video {
        alt: Option<String>,
        id: Option<String>,
        media_content_type: String,
        presentation: Option<MediaPresentation>,
        preview_image: Option<Image>,
        sources: Vec<VideoSource>,
    },
    Model3d {
        alt: Option<String>,
        id: Option<String>,
        media_content_type: String,
        presentation: Option<MediaPresentation>,
        preview_image: Option<Image>,
        sources: Vec<ModelSource>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "__typename")]
pub enum BillingPolicy {
    SellingPlanRecurringBillingPolicy {
        interval: String,
        interval_count: i64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "__typename")]
pub enum DeliveryPolicy {
    SellingPlanRecurringDeliveryPolicy {
        interval: String,
        interval_count: i64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "__typename")]
pub enum CheckoutChargeValue {
    MoneyV2 {
        amount: String,
        currency_code: String,
    },
    SellingPlanCheckoutChargePercentageValue {
        percentage: Number,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "__typename")]
pub enum PriceAdjustmentValue {
    SellingPlanFixedAmountPriceAdjustment { adjustment_amount: Money },
    SellingPlanFixedPriceAdjustment { price: Money },
    SellingPlanPercentagePriceAdjustment { adjustment_percentage: Number },
}

#[cfg(test)]
mod contract_tests {
    use super::*;
    use serde_json::{Value, json};
    #[test]
    fn nested_fields_and_money_are_strict() {
        let product: Value = serde_json::from_str(include_str!(
            "../minisforumau/offer_details/v2/fixtures/product.json"
        ))
        .unwrap();
        for pointer in ["", "/priceRange/minVariantPrice"] {
            let mut changed = product.clone();
            changed
                .pointer_mut(pointer)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .insert("unmodeledField".into(), Value::Bool(true));
            assert!(serde_json::from_value::<Product>(changed).is_err());
        }
        let mut changed = product.clone();
        changed.as_object_mut().unwrap().remove("variants");
        assert!(serde_json::from_value::<Product>(changed).is_err());
        let mut changed = product;
        changed["priceRange"]["minVariantPrice"]["amount"] = json!(12.34);
        assert!(serde_json::from_value::<Product>(changed).is_err());
        let adjustment = json!({"__typename":"SellingPlanPercentagePriceAdjustment","adjustmentPercentage":12.5});
        let typed: PriceAdjustmentValue = serde_json::from_value(adjustment.clone()).unwrap();
        assert_eq!(serde_json::to_value(typed).unwrap(), adjustment);
    }
    #[test]
    fn historical_and_contextual_artifacts_remain_readable() {
        let product: Value = serde_json::from_str(include_str!(
            "../minisforumau/offer_details/v2/fixtures/product.json"
        ))
        .unwrap();
        let mut artifact = json!({"schema_version":1,"product":product,"pages":[]});
        assert!(serde_json::from_value::<ExtractedOfferDetails>(artifact.clone()).is_ok());
        artifact["schema_version"] = json!(2);
        assert!(serde_json::from_value::<ExtractedOfferDetails>(artifact.clone()).is_err());
        artifact["context"] = json!({"country":"CA","language":"FR","currency":"CAD"});
        assert!(serde_json::from_value::<ExtractedOfferDetails>(artifact.clone()).is_ok());
        artifact["schema_version"] = json!(1);
        assert!(serde_json::from_value::<ExtractedOfferDetails>(artifact).is_err());
        //         let error: ProductResponse = serde_json::from_value(
        //             json!({"data":{"product":null},"errors":[{"message":"missing"}]}),
        //         )
        //         .unwrap();
        //         assert!(error.data.unwrap().product.is_none());
    }
}
