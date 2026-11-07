use chrono::{DateTime, Utc};
use kernel::ApplicationEvent;

use crate::offers::retailer_offer_id::RetailerOfferId;

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfferGtinChangedEvent {
    pub id: RetailerOfferId,
    pub previous_gtin: Option<String>,
    pub gtin: Option<String>,
    pub updated_at: DateTime<Utc>,
}

impl OfferGtinChangedEvent {
    pub fn new(
        id: RetailerOfferId,
        previous_gtin: Option<String>,
        gtin: Option<String>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            previous_gtin,
            gtin,
            updated_at,
        }
    }
}

impl ApplicationEvent for OfferGtinChangedEvent {
    fn event_type(&self) -> &'static str {
        "OfferGtinChanged"
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfferSkuChangedEvent {
    pub id: RetailerOfferId,
    pub previous_sku: Option<String>,
    pub sku: Option<String>,
    pub updated_at: DateTime<Utc>,
}

impl OfferSkuChangedEvent {
    pub fn new(
        id: RetailerOfferId,
        previous_sku: Option<String>,
        sku: Option<String>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            previous_sku,
            sku,
            updated_at,
        }
    }
}

impl ApplicationEvent for OfferSkuChangedEvent {
    fn event_type(&self) -> &'static str {
        "OfferSkuChanged"
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfferShopifyProductGidChangedEvent {
    pub id: RetailerOfferId,
    pub previous_shopify_product_gid: Option<String>,
    pub shopify_product_gid: Option<String>,
    pub updated_at: DateTime<Utc>,
}

impl OfferShopifyProductGidChangedEvent {
    pub fn new(
        id: RetailerOfferId,
        previous_shopify_product_gid: Option<String>,
        shopify_product_gid: Option<String>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            previous_shopify_product_gid,
            shopify_product_gid,
            updated_at,
        }
    }
}

impl ApplicationEvent for OfferShopifyProductGidChangedEvent {
    fn event_type(&self) -> &'static str {
        "OfferShopifyProductGidChanged"
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfferShopifyProductVariantGidChangedEvent {
    pub id: RetailerOfferId,
    pub previous_shopify_product_variant_gid: Option<String>,
    pub shopify_product_variant_gid: Option<String>,
    pub updated_at: DateTime<Utc>,
}

impl OfferShopifyProductVariantGidChangedEvent {
    pub fn new(
        id: RetailerOfferId,
        previous_shopify_product_variant_gid: Option<String>,
        shopify_product_variant_gid: Option<String>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            previous_shopify_product_variant_gid,
            shopify_product_variant_gid,
            updated_at,
        }
    }
}

impl ApplicationEvent for OfferShopifyProductVariantGidChangedEvent {
    fn event_type(&self) -> &'static str {
        "OfferShopifyProductVariantGidChanged"
    }
}
