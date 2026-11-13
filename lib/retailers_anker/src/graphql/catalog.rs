use super::{Operation, next_cursor, product_gid};
use retailer_sauce::initial_fill::{CatalogBatch, Http};
use retailer_sauce::shopify::initial_fill::StorefrontToken;
use retailers_wip::initial_fill::{LocaleCatalogBatch, RequestedContext};
use serde_json::Value;
use std::collections::HashSet;
const BATCH_SIZE: usize = 500;
pub fn fetch(
    engine: &super::Engine,
    http: &dyn Http,
    token: Option<&StorefrontToken>,
    context: Option<&RequestedContext>,
    after: Option<String>,
) -> Result<LocaleCatalogBatch, String> {
    let mut context = context.cloned();
    let mut after = after;
    let mut cursors = HashSet::new();
    if let Some(cursor) = &after {
        cursors.insert(cursor.clone());
    }
    let mut product_ids = Vec::new();
    let mut seen = HashSet::new();
    loop {
        let first = (BATCH_SIZE - product_ids.len()).min(250) as u16;
        let response = execute(
            engine,
            http,
            token,
            &Operation::Catalog {
                first,
                after: after.clone(),
            },
            context.as_ref(),
        )?;
        if let Some(context) = &mut context {
            context.observe(&response)?;
        }
        let products = response
            .pointer("/data/products")
            .ok_or("catalog response is missing products")?;
        let edges = products["edges"]
            .as_array()
            .ok_or("catalog response is missing edges")?;
        if edges.len() > first as usize {
            return Err("catalog exceeded requested page size".into());
        }
        for edge in edges {
            let id = edge
                .pointer("/node/id")
                .and_then(Value::as_str)
                .ok_or("catalog product is missing an ID")?;
            product_gid(id)?;
            if seen.insert(id.to_owned()) {
                product_ids.push(id.to_owned());
            }
        }
        after = next_cursor(products)?;
        if let Some(cursor) = &after
            && !cursors.insert(cursor.clone())
        {
            return Err("catalog cursor did not advance".into());
        }
        if after.is_none() || product_ids.len() == BATCH_SIZE {
            break;
        }
    }
    Ok(LocaleCatalogBatch {
        batch: CatalogBatch {
            product_ids,
            next_cursor: after,
        },
        context,
    })
}
fn execute(
    engine: &super::Engine,
    http: &dyn Http,
    token: Option<&StorefrontToken>,
    operation: &Operation,
    context: Option<&RequestedContext>,
) -> Result<Value, String> {
    if let Some(context) = context {
        engine.execute(
            http,
            token,
            &Operation::InLocale {
                context: context.clone(),
                operation: Box::new(operation.clone()),
            },
        )
    } else {
        engine.execute(http, token, operation)
    }
}
