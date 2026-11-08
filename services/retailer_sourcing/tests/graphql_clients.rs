//! Live comparison retaining only successful exhaustive-data HTTP evidence.
use retailer_sourcing::graphql;
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
};
type Download = fn(&Path, usize) -> Result<PathBuf, Box<dyn std::error::Error>>;

fn json_file(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}
/// Remove preliminary operations and unsuccessful attempts from comparison artifacts.
fn retain_exhaustive_evidence(dir: &Path) {
    if !dir.exists() {
        return;
    }
    if dir.join("request.json").exists() {
        let request = json_file(&dir.join("request.json"));
        let response = fs::read(dir.join("response.json"))
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok());
        let metadata = json_file(&dir.join("metadata.json"));
        let keep = request["operationName"] == "Dump"
            && metadata["status"] == 200
            && response.as_ref().is_some_and(|response| {
                response["data"].is_object()
                    && response["errors"].as_array().is_none_or(Vec::is_empty)
            });
        if !keep {
            fs::remove_dir_all(dir).unwrap();
        }
        return;
    }
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            retain_exhaustive_evidence(&path);
            if path.exists() && fs::read_dir(&path).unwrap().next().is_none() {
                fs::remove_dir(path).unwrap();
            }
        }
    }
}

fn check_evidence(dir: &Path) -> usize {
    let mut count = 0;
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            count += check_evidence(&path);
        } else if path.file_name().unwrap() == "request.json" {
            let request = json_file(&path);
            assert!(request["query"].is_string());
            assert_eq!(request["operationName"], "Dump");
            let parent = path.parent().unwrap();
            let response = json_file(&parent.join("response.json"));
            assert!(response["data"].is_object());
            assert!(response["errors"].as_array().is_none_or(Vec::is_empty));
            assert_eq!(json_file(&parent.join("metadata.json"))["status"], 200);
            count += 1;
        }
    }
    count
}

#[test]
#[ignore = "fetches live Anker AU data and retains request/response artifacts"]
fn first_five_products_all_clients() {
    let base = std::env::var_os("GRAPHQL_TEST_OUTPUT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/artifacts/graphql-first-five")
        })
        .join(uuid::Uuid::now_v7().to_string());
    fs::create_dir_all(&base).unwrap();
    println!("GraphQL comparison evidence: {}", base.display());
    let clients: [(&str, Download); 3] = [
        ("custom", graphql::download_ankerau_limited),
        (
            "graphql_client",
            graphql::download_ankerau_graphql_client_limited,
        ),
        ("cynic", graphql::download_ankerau_cynic_limited),
    ];
    let mut manifest = json!({"product_limit":5,"implementations":{}});
    let mut first_ids = None;
    let mut failures = vec![];
    for (name, download) in clients {
        let result = download(&base.join(name), 5);
        retain_exhaustive_evidence(&base.join(name));
        match result {
            Ok(run) => {
                let summary = json_file(&run.join("summary.json"));
                assert_eq!(summary["implementation"], name);
                assert_eq!(summary["status"], "complete");
                assert_eq!(summary["product_limit"], 5);
                assert_eq!(summary["product_count"], 5);
                let ids = summary["products"].as_array().unwrap().clone();
                assert_eq!(ids.len(), 5);
                assert_eq!(fs::read_dir(run.join("AU-EN/products")).unwrap().count(), 5);
                let requests = check_evidence(&run);
                assert!(requests > 0);
                manifest["implementations"][name] = json!({"run":run.strip_prefix(&base).unwrap(),"products":ids,"requests":requests});
                if let Some(expected) = &first_ids {
                    assert_eq!(&ids, expected, "first five IDs differ for {name}");
                } else {
                    first_ids = Some(ids);
                }
            }
            Err(error) => {
                manifest["implementations"][name] = json!({"error":error.to_string()});
                failures.push(format!("{name}: {error}"));
            }
        }
        fs::write(
            base.join("manifest.json"),
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn filters_preliminary_and_failed_attempts_but_keeps_nested_data() {
    let root = std::env::temp_dir().join(uuid::Uuid::now_v7().to_string());
    let cases = [
        ("Schema", 200, json!({"data":{"__schema":{}}}), false),
        ("Catalogue", 200, json!({"data":{"products":{}}}), false),
        ("Dump", 429, json!({"errors":[{"message":"cost"}]}), false),
        (
            "Dump",
            200,
            json!({"data":{},"errors":[{"message":"denied"}]}),
            false,
        ),
        ("Dump", 200, json!({"data":{"product":{"images":{}}}}), true),
    ];
    for (index, (operation, status, response, _)) in cases.iter().enumerate() {
        let folder = root.join(index.to_string()).join("attempt-1");
        fs::create_dir_all(&folder).unwrap();
        for (name, value) in [
            (
                "request.json",
                json!({"operationName":operation,"query":"query Dump { product { id } }"}),
            ),
            ("metadata.json", json!({"status":status})),
            ("response.json", response.clone()),
        ] {
            fs::write(folder.join(name), serde_json::to_vec(&value).unwrap()).unwrap();
        }
    }
    retain_exhaustive_evidence(&root);
    for (index, (_, _, _, keep)) in cases.iter().enumerate() {
        assert_eq!(root.join(index.to_string()).exists(), *keep);
    }
    assert_eq!(check_evidence(&root), 1);
    fs::remove_dir_all(root).unwrap();
}
