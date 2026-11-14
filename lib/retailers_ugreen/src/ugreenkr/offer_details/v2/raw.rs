pub use crate::graphql::raw::*;
#[cfg(test)]
use serde_json::Value;
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn captured_product_preserves_all_source_fields() {
        let source: Value = serde_json::from_str(include_str!("fixtures/product.json")).unwrap();
        let product: Product = serde_json::from_value(source.clone()).unwrap();
        let typed = serde_json::to_value(product).unwrap();
        fn preserved(source: &Value, typed: &Value) {
            match source {
                Value::Object(fields) => {
                    // Older captures carry selling plans, which are no longer modeled.
                    for (key, value) in fields.iter().filter(|(key, _)| {
                        !matches!(key.as_str(), "sellingPlanGroups" | "sellingPlanAllocations")
                    }) {
                        preserved(value, &typed[key]);
                    }
                }
                Value::Array(values) => {
                    assert_eq!(values.len(), typed.as_array().unwrap().len());
                    for (index, value) in values.iter().enumerate() {
                        preserved(value, &typed[index]);
                    }
                }
                _ => assert_eq!(source, typed),
            }
        }
        preserved(&source, &typed);
    }
    #[test]
    #[ignore = "reads external data3 corpus"]
    fn saved_graphql_corpus() {
        let root =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../data3/ugreenkr/runs");
        fn walk(path: &std::path::Path, count: &mut usize) {
            for entry in std::fs::read_dir(path).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    walk(&path, count);
                } else if path.parent().unwrap().file_name().unwrap() == "products"
                    && path.extension().is_some_and(|x| x == "json")
                {
                    let _: Product = serde_json::from_str(&std::fs::read_to_string(&path).unwrap())
                        .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
                    *count += 1;
                }
            }
        }
        let mut count = 0;
        walk(&root, &mut count);
        assert!(count > 0);
    }
}
