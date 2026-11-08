// Generated from ankerau/offer_details.graphql with cynic-querygen 3.14.0.
// The operation directive is applied by the caller.
#![allow(dead_code)]
use super::schema;

#[derive(cynic::QueryVariables, Debug)]
pub struct OfferDetailsVariables<'a> {
    pub id: &'a cynic::Id,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "QueryRoot", variables = "OfferDetailsVariables")]
pub struct OfferDetails {
    #[arguments(id: $id)]
    pub product: Option<Product>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct Product {
    pub __typename: String,
    pub available_for_sale: bool,
    pub category: Option<TaxonomyCategory>,
    pub created_at: DateTime,
    pub description: String,
    pub description_html: Html,
    pub encoded_variant_availability: Option<String>,
    pub encoded_variant_existence: Option<String>,
    pub featured_image: Option<Image>,
    pub handle: String,
    pub id: cynic::Id,
    pub is_gift_card: bool,
    pub online_store_url: Option<Url>,
    pub options: Vec<ProductOption>,
    pub product_type: String,
    pub published_at: DateTime,
    pub requires_selling_plan: bool,
    pub seo: Seo,
    pub tags: Vec<String>,
    pub title: String,
    pub updated_at: DateTime,
    pub variants_count: Option<Count>,
    pub vendor: String,
    #[arguments(first: 25, after: null)]
    pub collections: CollectionConnection,
    #[arguments(first: 25, after: null)]
    pub images: ImageConnection,
    #[arguments(first: 25, after: null)]
    pub media: MediaConnection,
    #[arguments(first: 20, after: null)]
    pub variants: ProductVariantConnection,
    #[arguments(first: 20, after: null)]
    pub selling_plan_groups: SellingPlanGroupConnection,
    pub compare_at_price_range: ProductPriceRange,
    pub price_range: ProductPriceRange2,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "ProductPriceRange")]
pub struct ProductPriceRange2 {
    pub __typename: String,
    pub max_variant_price: MoneyV2,
    pub min_variant_price: MoneyV22,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "MoneyV2")]
pub struct MoneyV22 {
    pub __typename: String,
    pub amount: Decimal,
    pub currency_code: CurrencyCode,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct MoneyV2 {
    pub __typename: String,
    pub amount: Decimal,
    pub currency_code: CurrencyCode,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct ProductPriceRange {
    pub __typename: String,
    pub max_variant_price: MoneyV23,
    pub min_variant_price: MoneyV24,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "MoneyV2")]
pub struct MoneyV24 {
    pub __typename: String,
    pub amount: Decimal,
    pub currency_code: CurrencyCode,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "MoneyV2")]
pub struct MoneyV23 {
    pub __typename: String,
    pub amount: Decimal,
    pub currency_code: CurrencyCode,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct SellingPlanGroupConnection {
    pub page_info: PageInfo,
    pub edges: Vec<SellingPlanGroupEdge>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct SellingPlanGroupEdge {
    pub cursor: String,
    pub node: SellingPlanGroup,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct SellingPlanGroup {
    pub __typename: String,
    pub app_name: Option<String>,
    pub name: String,
    pub options: Vec<SellingPlanGroupOption>,
    #[arguments(first: 5)]
    pub selling_plans: SellingPlanConnection,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct SellingPlanConnection {
    pub page_info: PageInfo2,
    pub edges: Vec<SellingPlanEdge>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct SellingPlanEdge {
    pub cursor: String,
    pub node: SellingPlan,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct SellingPlan {
    pub __typename: String,
    pub billing_policy: Option<SellingPlanBillingPolicy>,
    pub checkout_charge: SellingPlanCheckoutCharge,
    pub delivery_policy: Option<SellingPlanDeliveryPolicy>,
    pub description: Option<String>,
    pub id: cynic::Id,
    pub name: String,
    pub options: Vec<SellingPlanOption>,
    pub price_adjustments: Vec<SellingPlanPriceAdjustment>,
    pub recurring_deliveries: bool,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct SellingPlanPriceAdjustment {
    pub __typename: String,
    pub adjustment_value: SellingPlanPriceAdjustmentValue,
    pub order_count: Option<i32>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct SellingPlanPercentagePriceAdjustment {
    pub adjustment_percentage: f64,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct SellingPlanFixedPriceAdjustment {
    pub price: MoneyV25,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "MoneyV2")]
pub struct MoneyV25 {
    pub __typename: String,
    pub amount: Decimal,
    pub currency_code: CurrencyCode,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct SellingPlanFixedAmountPriceAdjustment {
    pub adjustment_amount: MoneyV26,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "MoneyV2")]
pub struct MoneyV26 {
    pub __typename: String,
    pub amount: Decimal,
    pub currency_code: CurrencyCode,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct SellingPlanOption {
    pub __typename: String,
    pub name: Option<String>,
    pub value: Option<String>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct SellingPlanRecurringDeliveryPolicy {
    pub interval: SellingPlanInterval,
    pub interval_count: i32,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct SellingPlanCheckoutCharge {
    pub __typename: String,
    #[cynic(rename = "type")]
    pub type_: SellingPlanCheckoutChargeType,
    pub value: SellingPlanCheckoutChargeValue,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct SellingPlanCheckoutChargePercentageValue {
    pub percentage: f64,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "MoneyV2")]
pub struct MoneyV27 {
    pub amount: Decimal,
    pub currency_code: CurrencyCode,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct SellingPlanRecurringBillingPolicy {
    pub interval: SellingPlanInterval,
    pub interval_count: i32,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "PageInfo")]
pub struct PageInfo2 {
    pub has_next_page: bool,
    pub end_cursor: Option<String>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct SellingPlanGroupOption {
    pub __typename: String,
    pub name: String,
    pub values: Vec<String>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct PageInfo {
    pub has_next_page: bool,
    pub end_cursor: Option<String>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct ProductVariantConnection {
    pub page_info: PageInfo3,
    pub edges: Vec<ProductVariantEdge>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct ProductVariantEdge {
    pub cursor: String,
    pub node: ProductVariant,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct ProductVariant {
    pub __typename: String,
    pub available_for_sale: bool,
    pub barcode: Option<String>,
    pub compare_at_price: Option<MoneyV28>,
    #[arguments(first: 5)]
    pub components: ProductVariantComponentConnection,
    pub currently_not_in_stock: bool,
    #[arguments(first: 5)]
    pub grouped_by: ProductVariantConnection2,
    pub id: cynic::Id,
    pub image: Option<Image>,
    pub price: MoneyV29,
    #[arguments(first: 5)]
    pub quantity_price_breaks: QuantityPriceBreakConnection,
    pub quantity_rule: QuantityRule,
    pub requires_components: bool,
    pub requires_shipping: bool,
    pub selected_options: Vec<SelectedOption>,
    #[arguments(first: 5)]
    pub selling_plan_allocations: SellingPlanAllocationConnection,
    pub sku: Option<String>,
    #[arguments(first: 5)]
    pub store_availability: StoreAvailabilityConnection,
    pub taxable: bool,
    pub title: String,
    pub unit_price: Option<MoneyV210>,
    pub unit_price_measurement: Option<UnitPriceMeasurement>,
    pub weight: Option<f64>,
    pub weight_unit: WeightUnit,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct UnitPriceMeasurement {
    pub __typename: String,
    pub measured_type: Option<UnitPriceMeasurementMeasuredType>,
    pub quantity_unit: Option<UnitPriceMeasurementMeasuredUnit>,
    pub quantity_value: f64,
    pub reference_unit: Option<UnitPriceMeasurementMeasuredUnit>,
    pub reference_value: i32,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "MoneyV2")]
pub struct MoneyV210 {
    pub __typename: String,
    pub amount: Decimal,
    pub currency_code: CurrencyCode,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct StoreAvailabilityConnection {
    pub page_info: PageInfo4,
    pub edges: Vec<StoreAvailabilityEdge>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct StoreAvailabilityEdge {
    pub cursor: String,
    pub node: StoreAvailability,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct StoreAvailability {
    pub __typename: String,
    pub available: bool,
    pub location: Location,
    pub pick_up_time: String,
    pub quantity_available: i32,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct Location {
    pub __typename: String,
    pub address: LocationAddress,
    pub id: cynic::Id,
    pub name: String,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct LocationAddress {
    pub __typename: String,
    pub address1: Option<String>,
    pub address2: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub formatted: Vec<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub phone: Option<String>,
    pub province: Option<String>,
    pub province_code: Option<String>,
    pub zip: Option<String>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "PageInfo")]
pub struct PageInfo4 {
    pub has_next_page: bool,
    pub end_cursor: Option<String>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct SellingPlanAllocationConnection {
    pub page_info: PageInfo5,
    pub edges: Vec<SellingPlanAllocationEdge>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct SellingPlanAllocationEdge {
    pub cursor: String,
    pub node: SellingPlanAllocation,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct SellingPlanAllocation {
    pub __typename: String,
    pub checkout_charge_amount: MoneyV211,
    pub price_adjustments: Vec<SellingPlanAllocationPriceAdjustment>,
    pub remaining_balance_charge_amount: MoneyV212,
    pub selling_plan: SellingPlan2,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "SellingPlan")]
pub struct SellingPlan2 {
    pub __typename: String,
    pub billing_policy: Option<SellingPlanBillingPolicy2>,
    pub checkout_charge: SellingPlanCheckoutCharge2,
    pub delivery_policy: Option<SellingPlanDeliveryPolicy2>,
    pub description: Option<String>,
    pub id: cynic::Id,
    pub name: String,
    pub options: Vec<SellingPlanOption2>,
    pub price_adjustments: Vec<SellingPlanPriceAdjustment2>,
    pub recurring_deliveries: bool,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "SellingPlanPriceAdjustment")]
pub struct SellingPlanPriceAdjustment2 {
    pub __typename: String,
    pub adjustment_value: SellingPlanPriceAdjustmentValue2,
    pub order_count: Option<i32>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "SellingPlanPercentagePriceAdjustment")]
pub struct SellingPlanPercentagePriceAdjustment2 {
    pub adjustment_percentage: f64,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "SellingPlanFixedPriceAdjustment")]
pub struct SellingPlanFixedPriceAdjustment2 {
    pub price: MoneyV213,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "MoneyV2")]
pub struct MoneyV213 {
    pub __typename: String,
    pub amount: Decimal,
    pub currency_code: CurrencyCode,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "SellingPlanFixedAmountPriceAdjustment")]
pub struct SellingPlanFixedAmountPriceAdjustment2 {
    pub adjustment_amount: MoneyV214,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "MoneyV2")]
pub struct MoneyV214 {
    pub __typename: String,
    pub amount: Decimal,
    pub currency_code: CurrencyCode,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "SellingPlanOption")]
pub struct SellingPlanOption2 {
    pub __typename: String,
    pub name: Option<String>,
    pub value: Option<String>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "SellingPlanRecurringDeliveryPolicy")]
pub struct SellingPlanRecurringDeliveryPolicy2 {
    pub interval: SellingPlanInterval,
    pub interval_count: i32,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "SellingPlanCheckoutCharge")]
pub struct SellingPlanCheckoutCharge2 {
    pub __typename: String,
    #[cynic(rename = "type")]
    pub type_: SellingPlanCheckoutChargeType,
    pub value: SellingPlanCheckoutChargeValue2,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "SellingPlanCheckoutChargePercentageValue")]
pub struct SellingPlanCheckoutChargePercentageValue2 {
    pub percentage: f64,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "MoneyV2")]
pub struct MoneyV215 {
    pub amount: Decimal,
    pub currency_code: CurrencyCode,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "SellingPlanRecurringBillingPolicy")]
pub struct SellingPlanRecurringBillingPolicy2 {
    pub interval: SellingPlanInterval,
    pub interval_count: i32,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "MoneyV2")]
pub struct MoneyV212 {
    pub __typename: String,
    pub amount: Decimal,
    pub currency_code: CurrencyCode,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct SellingPlanAllocationPriceAdjustment {
    pub __typename: String,
    pub compare_at_price: MoneyV216,
    pub per_delivery_price: MoneyV217,
    pub price: MoneyV218,
    pub unit_price: Option<MoneyV219>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "MoneyV2")]
pub struct MoneyV219 {
    pub __typename: String,
    pub amount: Decimal,
    pub currency_code: CurrencyCode,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "MoneyV2")]
pub struct MoneyV218 {
    pub __typename: String,
    pub amount: Decimal,
    pub currency_code: CurrencyCode,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "MoneyV2")]
pub struct MoneyV217 {
    pub __typename: String,
    pub amount: Decimal,
    pub currency_code: CurrencyCode,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "MoneyV2")]
pub struct MoneyV216 {
    pub __typename: String,
    pub amount: Decimal,
    pub currency_code: CurrencyCode,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "MoneyV2")]
pub struct MoneyV211 {
    pub __typename: String,
    pub amount: Decimal,
    pub currency_code: CurrencyCode,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "PageInfo")]
pub struct PageInfo5 {
    pub has_next_page: bool,
    pub end_cursor: Option<String>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct SelectedOption {
    pub __typename: String,
    pub name: String,
    pub value: String,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct QuantityRule {
    pub __typename: String,
    pub increment: i32,
    pub maximum: Option<i32>,
    pub minimum: i32,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct QuantityPriceBreakConnection {
    pub page_info: PageInfo6,
    pub edges: Vec<QuantityPriceBreakEdge>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct QuantityPriceBreakEdge {
    pub cursor: String,
    pub node: QuantityPriceBreak,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct QuantityPriceBreak {
    pub __typename: String,
    pub minimum_quantity: i32,
    pub price: MoneyV220,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "MoneyV2")]
pub struct MoneyV220 {
    pub __typename: String,
    pub amount: Decimal,
    pub currency_code: CurrencyCode,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "PageInfo")]
pub struct PageInfo6 {
    pub has_next_page: bool,
    pub end_cursor: Option<String>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "MoneyV2")]
pub struct MoneyV29 {
    pub __typename: String,
    pub amount: Decimal,
    pub currency_code: CurrencyCode,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "ProductVariantConnection")]
pub struct ProductVariantConnection2 {
    pub page_info: PageInfo7,
    pub edges: Vec<ProductVariantEdge2>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "ProductVariantEdge")]
pub struct ProductVariantEdge2 {
    pub cursor: String,
    pub node: ProductVariant2,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "ProductVariant")]
pub struct ProductVariant2 {
    pub __typename: String,
    pub id: cynic::Id,
    pub title: String,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "PageInfo")]
pub struct PageInfo7 {
    pub has_next_page: bool,
    pub end_cursor: Option<String>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct ProductVariantComponentConnection {
    pub page_info: PageInfo8,
    pub edges: Vec<ProductVariantComponentEdge>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct ProductVariantComponentEdge {
    pub cursor: String,
    pub node: ProductVariantComponent,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct ProductVariantComponent {
    pub __typename: String,
    pub product_variant: ProductVariant3,
    pub quantity: i32,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "ProductVariant")]
pub struct ProductVariant3 {
    pub __typename: String,
    pub id: cynic::Id,
    pub title: String,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "PageInfo")]
pub struct PageInfo8 {
    pub has_next_page: bool,
    pub end_cursor: Option<String>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "MoneyV2")]
pub struct MoneyV28 {
    pub __typename: String,
    pub amount: Decimal,
    pub currency_code: CurrencyCode,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "PageInfo")]
pub struct PageInfo3 {
    pub has_next_page: bool,
    pub end_cursor: Option<String>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct MediaConnection {
    pub page_info: PageInfo9,
    pub edges: Vec<MediaEdge>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct MediaEdge {
    pub cursor: String,
    pub node: Media,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct Video {
    pub alt: Option<String>,
    pub media_content_type: MediaContentType,
    pub presentation: Option<MediaPresentation>,
    pub preview_image: Option<Image>,
    pub sources: Vec<VideoSource>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct VideoSource {
    pub __typename: String,
    pub format: String,
    pub height: i32,
    pub mime_type: String,
    pub url: String,
    pub width: i32,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct MediaPresentation {
    pub __typename: String,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct Model3d {
    pub alt: Option<String>,
    pub media_content_type: MediaContentType,
    pub presentation: Option<MediaPresentation2>,
    pub preview_image: Option<Image>,
    pub sources: Vec<Model3dSource>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct Model3dSource {
    pub __typename: String,
    pub filesize: i32,
    pub format: String,
    pub mime_type: String,
    pub url: String,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "MediaPresentation")]
pub struct MediaPresentation2 {
    pub __typename: String,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct MediaImage {
    pub alt: Option<String>,
    pub id: cynic::Id,
    pub image: Option<Image>,
    pub media_content_type: MediaContentType,
    pub presentation: Option<MediaPresentation3>,
    pub preview_image: Option<Image>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "MediaPresentation")]
pub struct MediaPresentation3 {
    pub __typename: String,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct ExternalVideo {
    pub alt: Option<String>,
    pub embed_url: Url,
    pub host: MediaHost,
    pub media_content_type: MediaContentType,
    pub origin_url: Url,
    pub presentation: Option<MediaPresentation4>,
    pub preview_image: Option<Image>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "MediaPresentation")]
pub struct MediaPresentation4 {
    pub __typename: String,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "PageInfo")]
pub struct PageInfo9 {
    pub has_next_page: bool,
    pub end_cursor: Option<String>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct ImageConnection {
    pub page_info: PageInfo10,
    pub edges: Vec<ImageEdge>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct ImageEdge {
    pub cursor: String,
    pub node: Image,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "PageInfo")]
pub struct PageInfo10 {
    pub has_next_page: bool,
    pub end_cursor: Option<String>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct CollectionConnection {
    pub page_info: PageInfo11,
    pub edges: Vec<CollectionEdge>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct CollectionEdge {
    pub cursor: String,
    pub node: Collection,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct Collection {
    pub __typename: String,
    pub description: String,
    pub description_html: Html,
    pub handle: String,
    pub id: cynic::Id,
    pub image: Option<Image>,
    pub online_store_url: Option<Url>,
    pub seo: SEO2,
    pub title: String,
    pub updated_at: DateTime,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "SEO")]
pub struct SEO2 {
    pub __typename: String,
    pub description: Option<String>,
    pub title: Option<String>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "PageInfo")]
pub struct PageInfo11 {
    pub has_next_page: bool,
    pub end_cursor: Option<String>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct Count {
    pub __typename: String,
    pub count: i32,
    pub precision: CountPrecision,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "SEO")]
pub struct Seo {
    pub __typename: String,
    pub description: Option<String>,
    pub title: Option<String>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct ProductOption {
    pub __typename: String,
    pub id: cynic::Id,
    pub name: String,
    pub option_values: Vec<ProductOptionValue>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct ProductOptionValue {
    pub __typename: String,
    pub id: cynic::Id,
    pub name: String,
    pub swatch: Option<ProductOptionValueSwatch>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct ProductOptionValueSwatch {
    pub __typename: String,
    pub color: Option<Color>,
    pub image: Option<Media2>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "Video")]
pub struct Video2 {
    pub alt: Option<String>,
    pub media_content_type: MediaContentType,
    pub presentation: Option<MediaPresentation5>,
    pub preview_image: Option<Image>,
    pub sources: Vec<VideoSource2>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "VideoSource")]
pub struct VideoSource2 {
    pub __typename: String,
    pub format: String,
    pub height: i32,
    pub mime_type: String,
    pub url: String,
    pub width: i32,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "MediaPresentation")]
pub struct MediaPresentation5 {
    pub __typename: String,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "Model3d")]
pub struct Model3d2 {
    pub alt: Option<String>,
    pub media_content_type: MediaContentType,
    pub presentation: Option<MediaPresentation6>,
    pub preview_image: Option<Image>,
    pub sources: Vec<Model3dSource2>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "Model3dSource")]
pub struct Model3dSource2 {
    pub __typename: String,
    pub filesize: i32,
    pub format: String,
    pub mime_type: String,
    pub url: String,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "MediaPresentation")]
pub struct MediaPresentation6 {
    pub __typename: String,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "MediaImage")]
pub struct MediaImage2 {
    pub alt: Option<String>,
    pub id: cynic::Id,
    pub image: Option<Image>,
    pub media_content_type: MediaContentType,
    pub presentation: Option<MediaPresentation7>,
    pub preview_image: Option<Image>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "MediaPresentation")]
pub struct MediaPresentation7 {
    pub __typename: String,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "ExternalVideo")]
pub struct ExternalVideo2 {
    pub alt: Option<String>,
    pub embed_url: Url,
    pub host: MediaHost,
    pub media_content_type: MediaContentType,
    pub origin_url: Url,
    pub presentation: Option<MediaPresentation8>,
    pub preview_image: Option<Image>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct Image {
    pub __typename: String,
    pub alt_text: Option<String>,
    pub height: Option<i32>,
    pub id: Option<cynic::Id>,
    pub thumbhash: Option<String>,
    pub url: Url,
    pub width: Option<i32>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "MediaPresentation")]
pub struct MediaPresentation8 {
    pub __typename: String,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub struct TaxonomyCategory {
    pub __typename: String,
    pub ancestors: Vec<TaxonomyCategory2>,
    pub id: cynic::Id,
    pub name: String,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "TaxonomyCategory")]
pub struct TaxonomyCategory2 {
    pub __typename: String,
    pub id: cynic::Id,
    pub name: String,
}

#[derive(cynic::InlineFragments, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "Media")]
pub enum Media2 {
    ExternalVideo2(ExternalVideo2),
    MediaImage2(MediaImage2),
    Model3d2(Model3d2),
    Video2(Video2),
    #[cynic(fallback)]
    Unknown,
}

#[derive(cynic::InlineFragments, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub enum Media {
    ExternalVideo(ExternalVideo),
    MediaImage(MediaImage),
    Model3d(Model3d),
    Video(Video),
    #[cynic(fallback)]
    Unknown,
}

#[derive(cynic::InlineFragments, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "SellingPlanBillingPolicy")]
pub enum SellingPlanBillingPolicy2 {
    SellingPlanRecurringBillingPolicy2(SellingPlanRecurringBillingPolicy2),
    #[cynic(fallback)]
    Unknown,
}

#[derive(cynic::InlineFragments, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "SellingPlanCheckoutChargeValue")]
pub enum SellingPlanCheckoutChargeValue2 {
    MoneyV215(MoneyV215),
    SellingPlanCheckoutChargePercentageValue2(SellingPlanCheckoutChargePercentageValue2),
    #[cynic(fallback)]
    Unknown,
}

#[derive(cynic::InlineFragments, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "SellingPlanDeliveryPolicy")]
pub enum SellingPlanDeliveryPolicy2 {
    SellingPlanRecurringDeliveryPolicy2(SellingPlanRecurringDeliveryPolicy2),
    #[cynic(fallback)]
    Unknown,
}

#[derive(cynic::InlineFragments, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
#[cynic(graphql_type = "SellingPlanPriceAdjustmentValue")]
pub enum SellingPlanPriceAdjustmentValue2 {
    SellingPlanFixedAmountPriceAdjustment2(SellingPlanFixedAmountPriceAdjustment2),
    SellingPlanFixedPriceAdjustment2(SellingPlanFixedPriceAdjustment2),
    SellingPlanPercentagePriceAdjustment2(SellingPlanPercentagePriceAdjustment2),
    #[cynic(fallback)]
    Unknown,
}

#[derive(cynic::InlineFragments, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub enum SellingPlanBillingPolicy {
    SellingPlanRecurringBillingPolicy(SellingPlanRecurringBillingPolicy),
    #[cynic(fallback)]
    Unknown,
}

#[derive(cynic::InlineFragments, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub enum SellingPlanCheckoutChargeValue {
    MoneyV27(MoneyV27),
    SellingPlanCheckoutChargePercentageValue(SellingPlanCheckoutChargePercentageValue),
    #[cynic(fallback)]
    Unknown,
}

#[derive(cynic::InlineFragments, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub enum SellingPlanDeliveryPolicy {
    SellingPlanRecurringDeliveryPolicy(SellingPlanRecurringDeliveryPolicy),
    #[cynic(fallback)]
    Unknown,
}

#[derive(cynic::InlineFragments, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub enum SellingPlanPriceAdjustmentValue {
    SellingPlanFixedAmountPriceAdjustment(SellingPlanFixedAmountPriceAdjustment),
    SellingPlanFixedPriceAdjustment(SellingPlanFixedPriceAdjustment),
    SellingPlanPercentagePriceAdjustment(SellingPlanPercentagePriceAdjustment),
    #[cynic(fallback)]
    Unknown,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub enum CountPrecision {
    Exact,
    AtLeast,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub enum CurrencyCode {
    Usd,
    Eur,
    Gbp,
    Cad,
    Afn,
    All,
    Dzd,
    Aoa,
    Ars,
    Amd,
    Awg,
    Aud,
    Bbd,
    Azn,
    Bdt,
    Bsd,
    Bhd,
    Bif,
    Bzd,
    Bmd,
    Btn,
    Bam,
    Brl,
    Bob,
    Bwp,
    Bnd,
    Bgn,
    Mmk,
    Khr,
    Cve,
    Xcg,
    Kyd,
    Xaf,
    Clp,
    Cny,
    Cop,
    Kmf,
    Cdf,
    Crc,
    Hrk,
    Czk,
    Dkk,
    Dop,
    Xcd,
    Egp,
    Ern,
    Etb,
    Fkp,
    Xpf,
    Fjd,
    Gip,
    Gmd,
    Ghs,
    Gtq,
    Gyd,
    Gel,
    Htg,
    Hnl,
    Hkd,
    Huf,
    Isk,
    Inr,
    Idr,
    Ils,
    Iqd,
    Jmd,
    Jpy,
    Jep,
    Jod,
    Kzt,
    Kes,
    Kid,
    Kwd,
    Kgs,
    Lak,
    Lvl,
    Lbp,
    Lsl,
    Lrd,
    Ltl,
    Mga,
    Mkd,
    Mop,
    Mwk,
    Mvr,
    Mru,
    Mxn,
    Myr,
    Mur,
    Mdl,
    Mad,
    Mnt,
    Mzn,
    Nad,
    Npr,
    Ang,
    Nzd,
    Nio,
    Ngn,
    Nok,
    Omr,
    Pab,
    Pkr,
    Pgk,
    Pyg,
    Pen,
    Php,
    Pln,
    Qar,
    Ron,
    Rub,
    Rwf,
    Wst,
    Shp,
    Sar,
    Rsd,
    Scr,
    Sgd,
    Sdg,
    Sos,
    Syp,
    Zar,
    Krw,
    Ssp,
    Sbd,
    Lkr,
    Srd,
    Szl,
    Sek,
    Chf,
    Twd,
    Thb,
    Tzs,
    Ttd,
    Tnd,
    Try,
    Tmt,
    Ugx,
    Uah,
    Aed,
    Uyu,
    Uzs,
    Vuv,
    Ves,
    Vnd,
    Xof,
    Yer,
    Zmw,
    Byn,
    Byr,
    Djf,
    Gnf,
    Irr,
    Lyd,
    Sll,
    Std,
    Stn,
    Tjs,
    Top,
    Ved,
    Vef,
    Xxx,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub enum MediaContentType {
    ExternalVideo,
    Image,
    #[cynic(rename = "MODEL_3D")]
    Model3D,
    Video,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub enum MediaHost {
    Youtube,
    Vimeo,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub enum SellingPlanCheckoutChargeType {
    Percentage,
    Price,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub enum SellingPlanInterval {
    Day,
    Month,
    Week,
    Year,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub enum UnitPriceMeasurementMeasuredType {
    Volume,
    Weight,
    Length,
    Area,
    Count,
    Unknown,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub enum UnitPriceMeasurementMeasuredUnit {
    Ml,
    Cl,
    L,
    M3,
    Floz,
    Pt,
    Qt,
    Gal,
    Mg,
    G,
    Kg,
    Lb,
    Oz,
    Mm,
    Cm,
    M,
    In,
    Ft,
    Yd,
    M2,
    Ft2,
    Item,
    Unknown,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
#[cynic(schema_path = "src/platform/graphql/ankerau/schema.graphql")]
pub enum WeightUnit {
    Kilograms,
    Grams,
    Pounds,
    Ounces,
}

#[derive(cynic::Scalar, Debug, Clone)]
pub struct Color(pub String);

#[derive(cynic::Scalar, Debug, Clone)]
pub struct DateTime(pub String);

#[derive(cynic::Scalar, Debug, Clone)]
pub struct Decimal(pub String);

#[derive(cynic::Scalar, Debug, Clone)]
#[cynic(graphql_type = "HTML")]
pub struct Html(pub String);

#[derive(cynic::Scalar, Debug, Clone)]
#[cynic(graphql_type = "URL")]
pub struct Url(pub String);
