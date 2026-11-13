//! One-to-one raw-to-extracted conversions: exactly one `TryFrom` per raw type.
//! Every impl destructures its raw value exhaustively, so a new raw field fails to compile
//! until it is mapped here.
use super::extracted::prelude::*;
use crate::graphql::raw;
use chrono::{DateTime, Utc};

fn opt<T, U: TryFrom<T, Error = String>>(value: Option<T>) -> Result<Option<U>, String> {
    value.map(U::try_from).transpose()
}
fn vec<T, U: TryFrom<T, Error = String>>(value: Vec<T>) -> Result<Vec<U>, String> {
    value.into_iter().map(U::try_from).collect()
}

/// Parse a Shopify `DateTime` (RFC 3339) and normalise it to UTC.
fn utc(field: &str, value: String) -> Result<DateTime<Utc>, String> {
    DateTime::parse_from_rfc3339(&value)
        .map(|parsed| parsed.with_timezone(&Utc))
        .map_err(|error| format!("{field} is not an RFC 3339 instant ({value:?}): {error}"))
}

impl TryFrom<raw::RawOfferDetails> for ExtractedOfferDetails {
    type Error = String;
    fn try_from(value: raw::RawOfferDetails) -> Result<Self, Self::Error> {
        let raw::RawOfferDetails {
            schema_version: _,
            acquired_at,
            context,
            product,
            pages: _,
        } = value;
        let collections = product
            .collections
            .edges
            .iter()
            .map(|edge| Collection::try_from(edge.node.clone()))
            .collect::<Result<Vec<_>, String>>()?;
        let mut images = product
            .images
            .edges
            .iter()
            .map(|edge| Image::try_from(edge.node.clone()))
            .collect::<Result<Vec<_>, String>>()?;
        for edge in &product.variants.edges {
            if let Some(image) = &edge.node.image {
                images.push(Image::try_from(image.clone())?);
            }
        }
        if let Some(image) = &product.featured_image {
            images.push(Image::try_from(image.clone())?);
        }
        let media = product
            .media
            .edges
            .iter()
            .map(|edge| Media::try_from(edge.node.clone()))
            .collect::<Result<Vec<_>, String>>()?;
        Ok(Self {
            // schema_version,
            acquired_at: acquired_at
                .map(|value| utc("acquired_at", value))
                .transpose()?,
            context,
            product: Product::try_from(product)?,
            collections,
            images,
            media,
            // pages: vec(pages)?,
        })
    }
}

impl TryFrom<raw::Product> for Product {
    type Error = String;
    fn try_from(value: raw::Product) -> Result<Self, Self::Error> {
        let raw::Product {
            type_name: _,
            available_for_sale,
            category,
            collections,
            compare_at_price_range: _,
            created_at,
            description,
            description_html,
            encoded_variant_availability: _,
            encoded_variant_existence: _,
            featured_image,
            handle,
            id,
            images,
            is_gift_card,
            media,
            online_store_url,
            options,
            price_range: _,
            product_type,
            published_at,
            requires_selling_plan,
            selling_plan_groups,
            seo,
            tags,
            title,
            updated_at,
            variants,
            variants_count,
            vendor,
        } = value;
        if let Some(count) = &variants_count {
            let collected = variants.edges.len();
            let expected = usize::try_from(count.count)
                .map_err(|_| "product.variants_count must be nonnegative".to_string())?;
            let mismatch = match count.precision.as_str() {
                "EXACT" => collected != expected,
                "AT_LEAST" => collected < expected,
                _ => false,
            };
            if mismatch {
                return Err(format!(
                    "product.variants_count expected {} {} variants, collected {collected}",
                    count.precision, count.count
                ));
            }
        }
        Ok(Self {
            identifiers: ProductIdentifiers { id, handle },
            taxonomy: ProductTaxonomy {
                product_type,
                category: category
                    .filter(|category| category.id != "gid://shopify/TaxonomyCategory/na")
                    .map(Category::try_from)
                    .transpose()?,
                collections: collections
                    .edges
                    .into_iter()
                    .map(|edge| edge.node.id)
                    .collect(),
                tags,
            },
            content: ProductContent {
                vendor,
                featured_image: featured_image
                    .map(|image| {
                        image
                            .id
                            .ok_or_else(|| "product.featured_image is missing id".to_string())
                    })
                    .transpose()?,
                images: images
                    .edges
                    .into_iter()
                    .map(|edge| {
                        edge.node
                            .id
                            .ok_or_else(|| "product.images item is missing id".to_string())
                    })
                    .collect::<Result<Vec<_>, String>>()?,
                media: media
                    .edges
                    .into_iter()
                    .map(|edge| match edge.node {
                        raw::Media::MediaImage { id, .. }
                        | raw::Media::ExternalVideo { id, .. }
                        | raw::Media::Video { id, .. }
                        | raw::Media::Model3d { id, .. } => {
                            id.ok_or_else(|| "product.media item is missing id".to_string())
                        }
                    })
                    .collect::<Result<Vec<_>, String>>()?,
                description,
                description_html,
                title,
            },
            // type_name,
            available_for_sale,
            // compare_at_price_range: PriceRange::try_from(compare_at_price_range)?,
            // encoded_variant_availability,
            // encoded_variant_existence,
            is_gift_card,
            retailer_offer_url: online_store_url,
            options: options
                .into_iter()
                .filter(|option| {
                    !(variants.edges.len() == 1
                        && option.name == "Title"
                        && option.option_values.len() == 1
                        && matches!(
                            option.option_values[0].name.as_str(),
                            "Default Title" | "Default title"
                        ))
                })
                .map(ProductOption::try_from)
                .collect::<Result<Vec<_>, String>>()?,
            // price_range: PriceRange::try_from(price_range)?,
            requires_selling_plan,
            selling_plan_groups: selling_plan_groups
                .edges
                .into_iter()
                .map(|edge| SellingPlanGroup::try_from(edge.node))
                .collect::<Result<Vec<_>, String>>()?,
            seo: Seo::try_from(seo)?,
            variants: variants
                .edges
                .into_iter()
                .map(|edge| Variant::try_from(edge.node))
                .collect::<Result<Vec<_>, String>>()?,
            // variants_count: opt(variants_count)?,
            created_at: utc("product.created_at", created_at)?,
            published_at: utc("product.published_at", published_at)?,
            updated_at: utc("product.updated_at", updated_at)?,
        })
    }
}
impl TryFrom<raw::Category> for Category {
    type Error = String;
    fn try_from(value: raw::Category) -> Result<Self, Self::Error> {
        let raw::Category {
            type_name: _,
            ancestors,
            id,
            name,
        } = value;
        Ok(Self {
            // type_name,
            ancestors: vec(ancestors)?,
            id,
            name,
        })
    }
}
impl TryFrom<raw::CategoryAncestor> for CategoryAncestor {
    type Error = String;
    fn try_from(value: raw::CategoryAncestor) -> Result<Self, Self::Error> {
        let raw::CategoryAncestor {
            type_name: _,
            id,
            name,
        } = value;
        Ok(Self {
            // type_name,
            id,
            name,
        })
    }
}
impl TryFrom<raw::Collection> for Collection {
    type Error = String;
    fn try_from(value: raw::Collection) -> Result<Self, Self::Error> {
        let raw::Collection {
            type_name: _,
            description,
            description_html,
            handle,
            id,
            image,
            online_store_url,
            seo,
            title,
            updated_at,
            products,
        } = value;
        Ok(Self {
            // type_name,
            id,
            handle,
            url: online_store_url,
            title,
            description,
            description_html,
            image: opt(image)?,
            seo: Seo::try_from(seo)?,
            products: products
                .into_iter()
                .flat_map(|connection| connection.edges)
                .map(|edge| ProductSummary::try_from(edge.node))
                .collect::<Result<Vec<_>, String>>()?,
            updated_at: utc("collection.updated_at", updated_at)?,
        })
    }
}
impl TryFrom<raw::ProductSummary> for ProductSummary {
    type Error = String;
    fn try_from(value: raw::ProductSummary) -> Result<Self, Self::Error> {
        let raw::ProductSummary {
            type_name: _,
            id,
            handle,
            title,
        } = value;
        Ok(Self {
            // type_name,
            id,
            handle,
            title,
        })
    }
}
impl TryFrom<raw::Seo> for Seo {
    type Error = String;
    fn try_from(value: raw::Seo) -> Result<Self, Self::Error> {
        let raw::Seo {
            type_name: _,
            description,
            title,
        } = value;
        Ok(Self {
            // type_name,
            description,
            title,
        })
    }
}

impl TryFrom<raw::Image> for Image {
    type Error = String;
    fn try_from(value: raw::Image) -> Result<Self, Self::Error> {
        let raw::Image {
            type_name: _,
            alt_text,
            height,
            id,
            thumbhash,
            url,
            width,
        } = value;
        Ok(Self {
            // type_name,
            id,
            thumbhash,
            url,
            alt_text,
            width,
            height,
        })
    }
}
impl TryFrom<raw::Money> for Money {
    type Error = String;
    fn try_from(value: raw::Money) -> Result<Self, Self::Error> {
        let raw::Money {
            type_name: _,
            amount,
            currency_code,
        } = value;
        let currency: money::Currency = currency_code
            .parse()
            .map_err(|error| format!("invalid money currency {currency_code:?}: {error}"))?;
        let invalid = || format!("invalid money amount {amount:?} for {currency_code}");
        let (negative, unsigned) = match amount.strip_prefix('-') {
            Some(unsigned) => (true, unsigned),
            None => (false, amount.strip_prefix('+').unwrap_or(&amount)),
        };
        let (whole, fraction) = unsigned.split_once('.').unwrap_or((unsigned, ""));
        if whole.is_empty()
            || !whole.bytes().all(|b| b.is_ascii_digit())
            || !fraction.bytes().all(|b| b.is_ascii_digit())
        {
            return Err(invalid());
        }
        let fraction = fraction.trim_end_matches('0');
        let exponent = usize::from(currency.exponent());
        if fraction.len() > exponent {
            return Err(format!(
                "money amount {amount:?} exceeds {currency_code} precision"
            ));
        }
        let mut minor = 0_i128;
        for digit in whole.bytes().chain(fraction.bytes()) {
            minor = minor
                .checked_mul(10)
                .and_then(|value| value.checked_add(i128::from(digit - b'0')))
                .ok_or_else(invalid)?;
        }
        for _ in fraction.len()..exponent {
            minor = minor.checked_mul(10).ok_or_else(invalid)?;
        }
        if negative {
            minor = -minor;
        }
        let minor = i64::try_from(minor).map_err(|_| invalid())?;
        Ok(Money::new(minor, currency))
    }
}

impl TryFrom<raw::ProductOption> for ProductOption {
    type Error = String;
    fn try_from(value: raw::ProductOption) -> Result<Self, Self::Error> {
        let raw::ProductOption {
            type_name: _,
            id,
            name,
            option_values,
        } = value;
        Ok(Self {
            // type_name,
            id,
            name,
            option_values: vec(option_values)?,
        })
    }
}
impl TryFrom<raw::OptionValue> for OptionValue {
    type Error = String;
    fn try_from(value: raw::OptionValue) -> Result<Self, Self::Error> {
        let raw::OptionValue {
            type_name: _,
            id,
            name,
            swatch,
        } = value;
        Ok(Self {
            // type_name,
            id,
            name,
            swatch: opt(swatch)?,
        })
    }
}
impl TryFrom<raw::Swatch> for Swatch {
    type Error = String;
    fn try_from(value: raw::Swatch) -> Result<Self, Self::Error> {
        let raw::Swatch {
            type_name: _,
            color,
            image,
        } = value;
        Ok(Self {
            // type_name,
            color,
            image: opt(image)?,
        })
    }
}
impl TryFrom<raw::MediaPresentation> for MediaPresentation {
    type Error = String;
    fn try_from(value: raw::MediaPresentation) -> Result<Self, Self::Error> {
        let raw::MediaPresentation { type_name: _, id } = value;
        Ok(Self {
            // type_name,
            id,
        })
    }
}
impl TryFrom<raw::VideoSource> for VideoSource {
    type Error = String;
    fn try_from(value: raw::VideoSource) -> Result<Self, Self::Error> {
        let raw::VideoSource {
            type_name: _,
            format,
            height,
            mime_type,
            url,
            width,
        } = value;
        Ok(Self {
            // type_name,
            format,
            height,
            mime_type,
            url,
            width,
        })
    }
}
impl TryFrom<raw::ModelSource> for ModelSource {
    type Error = String;
    fn try_from(value: raw::ModelSource) -> Result<Self, Self::Error> {
        let raw::ModelSource {
            type_name: _,
            filesize,
            format,
            mime_type,
            url,
        } = value;
        Ok(Self {
            // type_name,
            filesize,
            format,
            mime_type,
            url,
        })
    }
}
impl TryFrom<raw::Variant> for Variant {
    type Error = String;
    fn try_from(value: raw::Variant) -> Result<Self, Self::Error> {
        let raw::Variant {
            type_name: _,
            available_for_sale,
            barcode,
            compare_at_price,
            components,
            currently_not_in_stock,
            grouped_by,
            id,
            image,
            price,
            quantity_price_breaks,
            quantity_rule,
            requires_components,
            requires_shipping,
            selected_options,
            selling_plan_allocations,
            sku,
            store_availability,
            taxable,
            title,
            unit_price,
            unit_price_measurement,
            weight,
            weight_unit,
        } = value;
        let price = Money::try_from(price)?;
        let compare_at_price: Option<Money> = opt(compare_at_price)?;
        let pricing = if price.amount() == 0 {
            Pricing::Unknown
        } else {
            match compare_at_price {
                Some(comparison)
                    if comparison.currency() == price.currency()
                        && comparison.amount() > price.amount() =>
                {
                    Pricing::Discounted(price, comparison)
                }
                _ => Pricing::Standard(price),
            }
        };
        Ok(Self {
            identifiers: VariantIdentifiers { id, sku, barcode },
            flags: VariantFlags {
                available_for_sale,
                currently_not_in_stock,
                taxable,
                requires_shipping,
            },
            specials: VariantSpecials {
                components: components
                    .edges
                    .into_iter()
                    .map(|edge| Component::try_from(edge.node))
                    .collect::<Result<Vec<_>, String>>()?,
                grouped_by: grouped_by
                    .edges
                    .into_iter()
                    .map(|edge| VariantSummary::try_from(edge.node))
                    .collect::<Result<Vec<_>, String>>()?,
                quantity_price_breaks: quantity_price_breaks
                    .edges
                    .into_iter()
                    .map(|edge| QuantityPriceBreak::try_from(edge.node))
                    .collect::<Result<Vec<_>, String>>()?,
                quantity_rule: QuantityRule::try_from(quantity_rule)?,
                requires_components,
                selling_plan_allocations: selling_plan_allocations
                    .edges
                    .into_iter()
                    .map(|edge| SellingPlanAllocation::try_from(edge.node))
                    .collect::<Result<Vec<_>, String>>()?,
            },
            title,
            pricing,
            // type_name,
            image: image
                .map(|image| {
                    image
                        .id
                        .ok_or_else(|| "variant.image is missing id".to_string())
                })
                .transpose()?,
            selected_options: selected_options
                .into_iter()
                .filter(|option| {
                    !(option.name == "Title"
                        && matches!(option.value.as_str(), "Default Title" | "Default title"))
                })
                .map(SelectedOption::try_from)
                .collect::<Result<Vec<_>, String>>()?,
            store_availability: store_availability
                .edges
                .into_iter()
                .map(|edge| StoreAvailability::try_from(edge.node))
                .collect::<Result<Vec<_>, String>>()?,
            unit_price: opt(unit_price)?,
            unit_price_measurement: opt(unit_price_measurement.filter(|measurement| {
                !(measurement.quantity_unit.is_none()
                    && measurement.quantity_value.as_f64() == Some(0.0)
                    && measurement.reference_unit.is_none()
                    && measurement.reference_value == 0)
            }))?,
            weight,
            weight_unit,
        })
    }
}
impl TryFrom<raw::VariantSummary> for VariantSummary {
    type Error = String;
    fn try_from(value: raw::VariantSummary) -> Result<Self, Self::Error> {
        let raw::VariantSummary {
            type_name: _,
            id,
            title,
        } = value;
        Ok(Self {
            // type_name,
            id,
            title,
        })
    }
}
impl TryFrom<raw::Component> for Component {
    type Error = String;
    fn try_from(value: raw::Component) -> Result<Self, Self::Error> {
        let raw::Component {
            type_name: _,
            product_variant,
            quantity,
        } = value;
        Ok(Self {
            // type_name,
            product_variant: VariantSummary::try_from(product_variant)?,
            quantity,
        })
    }
}
impl TryFrom<raw::QuantityPriceBreak> for QuantityPriceBreak {
    type Error = String;
    fn try_from(value: raw::QuantityPriceBreak) -> Result<Self, Self::Error> {
        let raw::QuantityPriceBreak {
            type_name: _,
            minimum_quantity,
            price,
        } = value;
        Ok(Self {
            // type_name,
            minimum_quantity,
            price: Money::try_from(price)?,
        })
    }
}
impl TryFrom<raw::QuantityRule> for QuantityRule {
    type Error = String;
    fn try_from(value: raw::QuantityRule) -> Result<Self, Self::Error> {
        let raw::QuantityRule {
            type_name: _,
            increment,
            maximum,
            minimum,
        } = value;
        Ok(Self {
            // type_name,
            increment,
            maximum,
            minimum,
        })
    }
}
impl TryFrom<raw::SelectedOption> for SelectedOption {
    type Error = String;
    fn try_from(value: raw::SelectedOption) -> Result<Self, Self::Error> {
        let raw::SelectedOption {
            type_name: _,
            name,
            value,
        } = value;
        Ok(Self {
            // type_name,
            name,
            value,
        })
    }
}
impl TryFrom<raw::UnitPriceMeasurement> for UnitPriceMeasurement {
    type Error = String;
    fn try_from(value: raw::UnitPriceMeasurement) -> Result<Self, Self::Error> {
        let raw::UnitPriceMeasurement {
            type_name: _,
            measured_type,
            quantity_unit,
            quantity_value,
            reference_unit,
            reference_value,
        } = value;
        Ok(Self {
            // type_name,
            measured_type,
            quantity_unit,
            quantity_value,
            reference_unit,
            reference_value,
        })
    }
}
impl TryFrom<raw::StoreAvailability> for StoreAvailability {
    type Error = String;
    fn try_from(value: raw::StoreAvailability) -> Result<Self, Self::Error> {
        let raw::StoreAvailability {
            type_name: _,
            available,
            location,
            pick_up_time,
            quantity_available,
        } = value;
        Ok(Self {
            // type_name,
            available,
            location: Location::try_from(location)?,
            pick_up_time,
            quantity_available,
        })
    }
}
impl TryFrom<raw::Location> for Location {
    type Error = String;
    fn try_from(value: raw::Location) -> Result<Self, Self::Error> {
        let raw::Location {
            type_name: _,
            address,
            id,
            name,
        } = value;
        Ok(Self {
            // type_name,
            address: Address::try_from(address)?,
            id,
            name,
        })
    }
}
impl TryFrom<raw::Address> for Address {
    type Error = String;
    fn try_from(value: raw::Address) -> Result<Self, Self::Error> {
        let raw::Address {
            type_name: _,
            address1,
            address2,
            city,
            country,
            country_code,
            formatted,
            latitude,
            longitude,
            phone,
            province,
            province_code,
            zip,
        } = value;
        Ok(Self {
            // type_name,
            address1,
            address2,
            city,
            country,
            country_code,
            formatted,
            latitude,
            longitude,
            phone,
            province,
            province_code,
            zip,
        })
    }
}
impl TryFrom<raw::SellingPlanAllocation> for SellingPlanAllocation {
    type Error = String;
    fn try_from(value: raw::SellingPlanAllocation) -> Result<Self, Self::Error> {
        let raw::SellingPlanAllocation {
            type_name: _,
            checkout_charge_amount,
            price_adjustments,
            remaining_balance_charge_amount,
            selling_plan,
        } = value;
        Ok(Self {
            // type_name,
            checkout_charge_amount: Money::try_from(checkout_charge_amount)?,
            price_adjustments: vec(price_adjustments)?,
            remaining_balance_charge_amount: Money::try_from(remaining_balance_charge_amount)?,
            selling_plan: SellingPlan::try_from(selling_plan)?,
        })
    }
}
impl TryFrom<raw::AllocationPriceAdjustment> for AllocationPriceAdjustment {
    type Error = String;
    fn try_from(value: raw::AllocationPriceAdjustment) -> Result<Self, Self::Error> {
        let raw::AllocationPriceAdjustment {
            type_name: _,
            compare_at_price,
            per_delivery_price,
            price,
            unit_price,
        } = value;
        Ok(Self {
            // type_name,
            compare_at_price: Money::try_from(compare_at_price)?,
            per_delivery_price: Money::try_from(per_delivery_price)?,
            price: Money::try_from(price)?,
            unit_price: opt(unit_price)?,
        })
    }
}
impl TryFrom<raw::SellingPlanGroup> for SellingPlanGroup {
    type Error = String;
    fn try_from(value: raw::SellingPlanGroup) -> Result<Self, Self::Error> {
        let raw::SellingPlanGroup {
            type_name: _,
            app_name,
            name,
            options,
            selling_plans,
        } = value;
        Ok(Self {
            // type_name,
            app_name,
            name,
            options: vec(options)?,
            selling_plans: selling_plans
                .edges
                .into_iter()
                .map(|edge| SellingPlan::try_from(edge.node))
                .collect::<Result<Vec<_>, String>>()?,
        })
    }
}
impl TryFrom<raw::SellingPlanGroupOption> for SellingPlanGroupOption {
    type Error = String;
    fn try_from(value: raw::SellingPlanGroupOption) -> Result<Self, Self::Error> {
        let raw::SellingPlanGroupOption {
            type_name: _,
            name,
            values,
        } = value;
        Ok(Self {
            // type_name,
            name,
            values,
        })
    }
}
impl TryFrom<raw::SellingPlan> for SellingPlan {
    type Error = String;
    fn try_from(value: raw::SellingPlan) -> Result<Self, Self::Error> {
        let raw::SellingPlan {
            type_name: _,
            billing_policy,
            checkout_charge,
            delivery_policy,
            description,
            id,
            name,
            options,
            price_adjustments,
            recurring_deliveries,
        } = value;
        Ok(Self {
            // type_name,
            billing_policy: opt(billing_policy)?,
            checkout_charge: CheckoutCharge::try_from(checkout_charge)?,
            delivery_policy: opt(delivery_policy)?,
            description,
            id,
            name,
            options: vec(options)?,
            price_adjustments: vec(price_adjustments)?,
            recurring_deliveries,
        })
    }
}
impl TryFrom<raw::SellingPlanOption> for SellingPlanOption {
    type Error = String;
    fn try_from(value: raw::SellingPlanOption) -> Result<Self, Self::Error> {
        let raw::SellingPlanOption {
            type_name: _,
            name,
            value,
        } = value;
        Ok(Self {
            // type_name,
            name,
            value,
        })
    }
}
impl TryFrom<raw::CheckoutCharge> for CheckoutCharge {
    type Error = String;
    fn try_from(value: raw::CheckoutCharge) -> Result<Self, Self::Error> {
        let raw::CheckoutCharge {
            type_name: _,
            kind,
            value,
        } = value;
        Ok(Self {
            // type_name,
            kind,
            value: CheckoutChargeValue::try_from(value)?,
        })
    }
}
impl TryFrom<raw::PlanPriceAdjustment> for PlanPriceAdjustment {
    type Error = String;
    fn try_from(value: raw::PlanPriceAdjustment) -> Result<Self, Self::Error> {
        let raw::PlanPriceAdjustment {
            type_name: _,
            adjustment_value,
            order_count,
        } = value;
        Ok(Self {
            // type_name,
            adjustment_value: PriceAdjustmentValue::try_from(adjustment_value)?,
            order_count,
        })
    }
}
impl TryFrom<raw::Media> for Media {
    type Error = String;
    fn try_from(value: raw::Media) -> Result<Self, Self::Error> {
        Ok(match value {
            raw::Media::MediaImage {
                alt,
                id,
                media_content_type,
                presentation,
                preview_image,
                image,
            } => Self::MediaImage {
                alt,
                id,
                media_content_type,
                presentation: opt(presentation)?,
                preview_image: opt(preview_image)?,
                image: opt(image)?,
            },
            raw::Media::ExternalVideo {
                alt,
                id,
                media_content_type,
                presentation,
                preview_image,
                embed_url,
                host,
                origin_url,
            } => Self::ExternalVideo {
                alt,
                id,
                media_content_type,
                presentation: opt(presentation)?,
                preview_image: opt(preview_image)?,
                embed_url,
                host,
                origin_url,
            },
            raw::Media::Video {
                alt,
                id,
                media_content_type,
                presentation,
                preview_image,
                sources,
            } => Self::Video {
                alt,
                id,
                media_content_type,
                presentation: opt(presentation)?,
                preview_image: opt(preview_image)?,
                sources: vec(sources)?,
            },
            raw::Media::Model3d {
                alt,
                id,
                media_content_type,
                presentation,
                preview_image,
                sources,
            } => Self::Model3d {
                alt,
                id,
                media_content_type,
                presentation: opt(presentation)?,
                preview_image: opt(preview_image)?,
                sources: vec(sources)?,
            },
        })
    }
}
impl TryFrom<raw::BillingPolicy> for BillingPolicy {
    type Error = String;
    fn try_from(value: raw::BillingPolicy) -> Result<Self, Self::Error> {
        Ok(match value {
            raw::BillingPolicy::SellingPlanRecurringBillingPolicy {
                interval,
                interval_count,
            } => Self::SellingPlanRecurringBillingPolicy {
                interval,
                interval_count,
            },
        })
    }
}
impl TryFrom<raw::DeliveryPolicy> for DeliveryPolicy {
    type Error = String;
    fn try_from(value: raw::DeliveryPolicy) -> Result<Self, Self::Error> {
        Ok(match value {
            raw::DeliveryPolicy::SellingPlanRecurringDeliveryPolicy {
                interval,
                interval_count,
            } => Self::SellingPlanRecurringDeliveryPolicy {
                interval,
                interval_count,
            },
        })
    }
}
impl TryFrom<raw::CheckoutChargeValue> for CheckoutChargeValue {
    type Error = String;
    fn try_from(value: raw::CheckoutChargeValue) -> Result<Self, Self::Error> {
        Ok(match value {
            raw::CheckoutChargeValue::MoneyV2 {
                amount,
                currency_code,
            } => Self::MoneyV2 {
                amount,
                currency_code,
            },
            raw::CheckoutChargeValue::SellingPlanCheckoutChargePercentageValue { percentage } => {
                Self::SellingPlanCheckoutChargePercentageValue { percentage }
            }
        })
    }
}
impl TryFrom<raw::PriceAdjustmentValue> for PriceAdjustmentValue {
    type Error = String;
    fn try_from(value: raw::PriceAdjustmentValue) -> Result<Self, Self::Error> {
        Ok(match value {
            raw::PriceAdjustmentValue::SellingPlanFixedAmountPriceAdjustment {
                adjustment_amount,
            } => Self::SellingPlanFixedAmountPriceAdjustment {
                adjustment_amount: Money::try_from(adjustment_amount)?,
            },
            raw::PriceAdjustmentValue::SellingPlanFixedPriceAdjustment { price } => {
                Self::SellingPlanFixedPriceAdjustment {
                    price: Money::try_from(price)?,
                }
            }
            raw::PriceAdjustmentValue::SellingPlanPercentagePriceAdjustment {
                adjustment_percentage,
            } => Self::SellingPlanPercentagePriceAdjustment {
                adjustment_percentage,
            },
        })
    }
}
