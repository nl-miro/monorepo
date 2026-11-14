//! Retailer-owned typed operations; raw JSON is retained after validation.
use graphql_client::{GraphQLQuery, Response};

use crate::graphql::Operation;
use retailer_sauce::shopify::initial_fill::{GraphqlRequest, StorefrontConfig};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

#[allow(clippy::upper_case_acronyms)]
type HTML = String;
#[allow(clippy::upper_case_acronyms)]
type URL = String;
type DateTime = String;
type Decimal = String;
type Color = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/operations/schema.graphql",
    query_path = "src/graphql/operations/catalog.graphql",
    variables_derives = "Deserialize"
)]
struct CatalogIdentifiers;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/operations/schema.graphql",
    query_path = "src/graphql/operations/offer_details.graphql",
    variables_derives = "Deserialize"
)]
struct OfferDetails;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/operations/schema.graphql",
    query_path = "src/graphql/operations/ProductCollections.graphql",
    variables_derives = "Deserialize"
)]
struct ProductCollections;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/operations/schema.graphql",
    query_path = "src/graphql/operations/ProductImages.graphql",
    variables_derives = "Deserialize"
)]
struct ProductImages;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/operations/schema.graphql",
    query_path = "src/graphql/operations/ProductMedia.graphql",
    variables_derives = "Deserialize"
)]
struct ProductMedia;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/operations/schema.graphql",
    query_path = "src/graphql/operations/ProductVariants.graphql",
    variables_derives = "Deserialize"
)]
struct ProductVariants;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/operations/schema.graphql",
    query_path = "src/graphql/operations/VariantComponents.graphql",
    variables_derives = "Deserialize"
)]
struct VariantComponents;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/operations/schema.graphql",
    query_path = "src/graphql/operations/VariantGroupedBy.graphql",
    variables_derives = "Deserialize"
)]
struct VariantGroupedBy;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/operations/schema.graphql",
    query_path = "src/graphql/operations/VariantQuantityPriceBreaks.graphql",
    variables_derives = "Deserialize"
)]
struct VariantQuantityPriceBreaks;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/operations/schema.graphql",
    query_path = "src/graphql/operations/VariantStoreAvailability.graphql",
    variables_derives = "Deserialize"
)]
struct VariantStoreAvailability;

fn build<Q: GraphQLQuery>(
    mut variables: Value,
    config: &StorefrontConfig,
) -> Result<GraphqlRequest, String>
where
    Q::Variables: DeserializeOwned,
{
    variables["language"] = json!(config.language);
    for size in [1, 5, 10, 20, 25] {
        variables[format!("pageSize{size}")] = json!(size);
    }
    let request = Q::build_query(serde_json::from_value(variables).map_err(|e| e.to_string())?);
    let mut variables = serde_json::to_value(request.variables).map_err(|e| e.to_string())?;
    variables
        .as_object_mut()
        .ok_or("query variables must be an object")?
        .remove("country");
    Ok(GraphqlRequest {
        operation_name: request.operation_name.into(),
        query: request.query.into(),
        variables,
    })
}
fn validate<Q: GraphQLQuery>(operation: &str, body: &Value) -> Result<(), String> {
    retailers_wip::graphql_errors::check(operation, body)?;
    let response: Response<Q::ResponseData> =
        serde_json::from_value(body.clone()).map_err(|error| {
            format!("Storefront {operation} response does not match the selected GraphQL operation: {error}")
        })?;
    if response.data.is_none() {
        return Err(format!(
            "Storefront {operation} returned missing or null data without GraphQL errors"
        ));
    }
    Ok(())
}

pub fn request(operation: &Operation, config: &StorefrontConfig) -> Result<GraphqlRequest, String> {
    match operation {
        Operation::Markets => Ok(super::markets::request()),
        Operation::InLocale { context, operation } => {
            let mut request = request(operation, config)?;
            if let Some(country) = &context.country {
                request.variables["country"] = json!(country);
            }
            request.variables["language"] = json!(context.language);
            Ok(request)
        }
        Operation::InContext { context, operation } => {
            let mut request = request(operation, config)?;
            request.variables["country"] = json!(context.country);
            request.variables["language"] = json!(context.language);
            Ok(request)
        }
        Operation::Catalog { first, after } => {
            build::<CatalogIdentifiers>(json!({"first": first, "after": after}), config)
        }
        Operation::Details { id } => build::<OfferDetails>(json!({"id": id}), config),
        Operation::ProductConnection { id, field, after } => {
            let variables = json!({"id": id, "after": after});
            match field.as_str() {
                "collections" => build::<ProductCollections>(variables, config),
                "images" => build::<ProductImages>(variables, config),
                "media" => build::<ProductMedia>(variables, config),
                "variants" => build::<ProductVariants>(variables, config),
                _ => Err("unsupported product connection".into()),
            }
        }
        Operation::VariantConnection { id, field, after } => {
            let variables = json!({"id": id, "after": after});
            match field.as_str() {
                "components" => build::<VariantComponents>(variables, config),
                "groupedBy" => build::<VariantGroupedBy>(variables, config),
                "quantityPriceBreaks" => build::<VariantQuantityPriceBreaks>(variables, config),
                "storeAvailability" => build::<VariantStoreAvailability>(variables, config),
                _ => Err("unsupported variant connection".into()),
            }
        }
    }
}

pub fn validate_response(
    operation: &Operation,
    body: &Value,
    config: &StorefrontConfig,
) -> Result<(), String> {
    if let Operation::Markets = operation {
        return super::markets::contexts(body).map(|_| ());
    }
    if let Operation::InContext { operation, .. } | Operation::InLocale { operation, .. } =
        operation
    {
        return validate_response(operation, body, config);
    }
    let name = request(operation, config)?.operation_name;
    match name.as_str() {
        "CatalogIdentifiers" => validate::<CatalogIdentifiers>(&name, body),
        "OfferDetails" => validate::<OfferDetails>(&name, body),
        "ProductCollections" => validate::<ProductCollections>(&name, body),
        "ProductImages" => validate::<ProductImages>(&name, body),
        "ProductMedia" => validate::<ProductMedia>(&name, body),
        "ProductVariants" => validate::<ProductVariants>(&name, body),
        "VariantComponents" => validate::<VariantComponents>(&name, body),
        "VariantGroupedBy" => validate::<VariantGroupedBy>(&name, body),
        "VariantQuantityPriceBreaks" => validate::<VariantQuantityPriceBreaks>(&name, body),
        "VariantStoreAvailability" => validate::<VariantStoreAvailability>(&name, body),
        _ => Err("unsupported GraphQL operation".into()),
    }
}
