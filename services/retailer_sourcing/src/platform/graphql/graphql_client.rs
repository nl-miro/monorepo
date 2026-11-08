//! Experimental blocking Shopify catalogue downloader.
use serde_json::{Value, json};
use std::{
    collections::{BTreeSet, HashMap},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
type Error = Box<dyn std::error::Error>;
type Result<T, E = Error> = std::result::Result<T, E>;
#[derive(Clone, Debug)]
struct Field {
    name: String,
    parent: String,
    args: Value,
    child: Vec<Field>,
    connection: bool,
}
struct Dump {
    http: reqwest::blocking::Client,
    endpoint: String,
    root: PathBuf,
    folder: PathBuf,
    sequence: usize,
    types: HashMap<String, Value>,
    disabled: BTreeSet<(String, String)>,
    boundaries: BTreeSet<String>,
    gaps: Vec<Value>,
}
fn save(path: &Path, value: &Value) -> Result<()> {
    std::fs::create_dir_all(path.parent().ok_or("missing parent")?)?;
    let temporary = path.with_extension("json.new");
    std::fs::write(&temporary, serde_json::to_vec_pretty(value)?)?;
    std::fs::rename(temporary, path)?;
    Ok(())
}
fn named(mut value: &Value) -> Result<&str> {
    while value["ofType"].is_object() {
        value = &value["ofType"];
    }
    value["name"]
        .as_str()
        .ok_or_else(|| "missing type name".into())
}
fn literal(v: &Value) -> String {
    match v {
        Value::Object(m) => format!(
            "{{{}}}",
            m.iter()
                .map(|(k, v)| format!("{k}:{}", literal(v)))
                .collect::<Vec<_>>()
                .join(",")
        ),
        Value::Array(a) => format!("[{}]", a.iter().map(literal).collect::<Vec<_>>().join(",")),
        _ => v.to_string(),
    }
}
fn render(fields: &[Field]) -> String {
    fields
        .iter()
        .map(|f| {
            let args = f.args.as_object().unwrap();
            let mut s = f.name.clone();
            if !args.is_empty() {
                s += &format!(
                    "({})",
                    args.iter()
                        .map(|(k, v)| format!("{k}:{}", literal(v)))
                        .collect::<Vec<_>>()
                        .join(",")
                );
            }
            if !f.child.is_empty() {
                let mut child = render(&f.child);
                if f.connection {
                    child = format!(
                        "pageInfo {{ hasNextPage endCursor }} edges {{ cursor node {{ {child} }} }}"
                    );
                }
                s += &format!(" {{ {child} }}");
            }
            s
        })
        .collect::<Vec<_>>()
        .join(" ")
}
fn cost(v: &Value) -> bool {
    v["errors"].as_array().is_some_and(|a| {
        a.iter().any(|e| {
            e["message"]
                .as_str()
                .unwrap_or("")
                .to_lowercase()
                .contains("cost")
                || matches!(
                    e["extensions"]["code"].as_str(),
                    Some("MAX_COST_EXCEEDED" | "MAX_COMPLEXITY_EXCEEDED")
                )
        })
    })
}
fn errors(v: &Value) -> bool {
    v["errors"].as_array().is_some_and(|a| !a.is_empty())
}
fn unfinished(v: &Value) -> bool {
    match v {
        Value::Object(o) => v["pageInfo"]["hasNextPage"] == true || o.values().any(unfinished),
        Value::Array(a) => a.iter().any(unfinished),
        _ => false,
    }
}
type Step = (String, Value, bool);
fn wrap(path: &[Step], mut leaf: String) -> String {
    for (key, args, connection) in path.iter().rev() {
        let f = Field {
            name: key.clone(),
            parent: String::new(),
            args: args.clone(),
            child: vec![],
            connection: false,
        };
        let key = render(&[f]);
        if *connection {
            leaf = format!("edges {{ node {{ {leaf} }} }}");
        }
        leaf = format!("{key} {{ {leaf} }}");
    }
    leaf
}
fn extract<'a>(mut data: &'a Value, path: &[Step]) -> Result<&'a Value> {
    for (key, _, connection) in path {
        if key.starts_with("... on ") {
            continue;
        }
        data = data.get(key).ok_or("missing response path")?;
        if *connection {
            data = data.pointer("/edges/0/node").ok_or("missing parent node")?;
        }
    }
    Ok(data)
}
impl Dump {
    fn query(&mut self, query: String, operation: &str, variables: Value) -> Result<Value> {
        self.sequence += 1;
        let folder = self
            .folder
            .join("http")
            .join(format!("{:06}", self.sequence));
        let body = json!({"query":query,"operationName":operation,"variables":variables});
        for attempt in 1..=5 {
            let evidence = folder.join(format!("attempt-{attempt}"));
            save(&evidence.join("request.json"), &body)?;
            let start = Instant::now();
            let sent = self
                .http
                .post(&self.endpoint)
                .header("Content-Type", "application/json")
                .body(serde_json::to_vec(&body)?)
                .send();
            let mut status = 0;
            let mut retry = None;
            let bytes = match sent {
                Ok(r) => {
                    status = r.status().as_u16();
                    retry = r
                        .headers()
                        .get("Retry-After")
                        .and_then(|v| v.to_str().ok())
                        .map(str::to_owned);
                    r.bytes().map(|b| b.to_vec())
                }
                Err(e) => Err(e),
            };
            let value = match bytes {
                Ok(bytes) => match serde_json::from_slice::<Value>(&bytes) {
                    Ok(v) => {
                        save(&evidence.join("response.json"), &v)?;
                        v
                    }
                    Err(_) => {
                        std::fs::write(evidence.join("response.txt"), bytes)?;
                        json!({"errors":[{"message":"Non-JSON response"}]})
                    }
                },
                Err(e) => {
                    status = 0;
                    let v = json!({"network_error":e.to_string()});
                    save(&evidence.join("error.json"), &v)?;
                    save(&evidence.join("response.json"), &v)?;
                    v
                }
            };
            save(
                &evidence.join("metadata.json"),
                &json!({"status":status,"seconds":start.elapsed().as_secs_f64(),"endpoint":self.endpoint,"requested_context":variables}),
            )?;
            if cost(&value) {
                return Ok(value);
            }
            let throttled = value["errors"]
                .as_array()
                .is_some_and(|a| a.iter().any(|e| e["extensions"]["code"] == "THROTTLED"));
            if status != 0 && status != 429 && status < 500 && !throttled {
                if status != 200 {
                    return Err(format!("HTTP {status}; evidence {}", evidence.display()).into());
                }
                return Ok(value);
            }
            if attempt < 5 {
                let seconds = retry
                    .and_then(|s| {
                        s.parse::<f64>().ok().or_else(|| {
                            chrono::DateTime::parse_from_rfc2822(&s).ok().map(|d| {
                                (d.timestamp() - chrono::Utc::now().timestamp()).max(0) as f64
                            })
                        })
                    })
                    .unwrap_or(0.0)
                    .max((1u32 << attempt) as f64);
                std::thread::sleep(Duration::from_secs_f64(seconds));
            }
        }
        Err(format!("retries exhausted: {}", folder.display()).into())
    }
    fn fetch(&mut self, field: String) -> Result<Value> {
        self.query(format!("query Dump($country: CountryCode!, $language: LanguageCode!) @inContext(country:$country, language:$language) {{ {field} }}"),"Dump",json!({"country":"AU","language":"EN"}))
    }
    fn build(&mut self, name: &str, seen: &[String], depth: usize) -> Result<Vec<Field>> {
        let typ = self.types.get(name).ok_or("unknown schema type")?.clone();
        let mut result = vec![Field {
            name: "__typename".into(),
            parent: name.into(),
            args: json!({}),
            child: vec![],
            connection: false,
        }];
        if matches!(typ["kind"].as_str(), Some("UNION" | "INTERFACE")) {
            for t in typ["possibleTypes"]
                .as_array()
                .ok_or("missing concrete types")?
            {
                let n = t["name"].as_str().ok_or("missing concrete name")?;
                result.push(Field {
                    name: format!("... on {n}"),
                    parent: name.into(),
                    args: json!({}),
                    child: self.build(n, seen, depth)?,
                    connection: false,
                });
            }
            return Ok(result);
        }
        let boundary = seen.iter().any(|s| s == name)
            || depth > 6
            || (depth > 0 && name == "Product")
            || (matches!(name, "ProductVariant" | "Collection")
                && seen.iter().any(|s| s == "Metafield"));
        if boundary {
            self.boundaries.insert(name.into());
        }
        for f in typ["fields"].as_array().into_iter().flatten() {
            let key = f["name"].as_str().ok_or("missing field")?;
            if self.disabled.contains(&(name.into(), key.into()))
                || (boundary && !matches!(key, "id" | "handle" | "title" | "name" | "type" | "url"))
                || matches!(
                    key,
                    "parentResource"
                        | "adjacentVariants"
                        | "selectedOrFirstAvailableVariant"
                        | "firstSelectableVariant"
                        | "product"
                        | "products"
                        | "trackingParameters"
                )
            {
                continue;
            }
            let child_type = named(&f["type"])?;
            let mut args = json!({});
            if key == "metafields" {
                if depth != 0 {
                    continue;
                }
                args["identifiers"] = serde_json::from_str(include_str!(
                    "../graphql_support/metafield-identifiers.json"
                ))?;
            } else if f["args"]
                .as_array()
                .into_iter()
                .flatten()
                .any(|a| a["type"]["kind"] == "NON_NULL" && a["defaultValue"].is_null())
            {
                continue;
            }
            let connection = child_type.ends_with("Connection");
            let mut next = seen.to_vec();
            next.push(name.into());
            let child = if connection {
                args["first"] = json!(if matches!(
                    key,
                    "images" | "media" | "collections" | "variants"
                ) {
                    10
                } else {
                    1
                });
                let edge = self.field_type(child_type, "edges")?;
                let node = self.field_type(&edge, "node")?;
                self.build(&node, &next, depth + 1)?
            } else if !matches!(
                self.types[child_type]["kind"].as_str(),
                Some("SCALAR" | "ENUM")
            ) {
                self.build(child_type, &next, depth + 1)?
            } else {
                vec![]
            };
            result.push(Field {
                name: key.into(),
                parent: name.into(),
                args,
                child,
                connection,
            });
        }
        Ok(result)
    }
    fn field_type(&self, name: &str, key: &str) -> Result<String> {
        let f = self.types[name]["fields"]
            .as_array()
            .ok_or("missing fields")?
            .iter()
            .find(|f| f["name"] == key)
            .ok_or("missing connection field")?;
        Ok(named(&f["type"])?.into())
    }
    fn complete(&mut self, tree: &[Field], data: &mut Value, path: &[Step]) -> Result<()> {
        for f in tree {
            if let Some(kind) = f.name.strip_prefix("... on ") {
                if data["__typename"] == kind {
                    self.complete(&f.child, data, path)?;
                }
                continue;
            }
            if f.child.is_empty() {
                continue;
            }
            let Some(value) = data.get_mut(&f.name) else {
                continue;
            };
            if value.is_null() {
                continue;
            }
            if f.connection {
                let mut seen = BTreeSet::new();
                while value["pageInfo"]["hasNextPage"] == true {
                    let cursor = value["pageInfo"]["endCursor"]
                        .as_str()
                        .filter(|s| !s.is_empty())
                        .ok_or("missing cursor")?
                        .to_owned();
                    if !seen.insert(cursor.clone()) {
                        return Err("repeated cursor".into());
                    }
                    let mut page = f.clone();
                    page.args["first"] = json!(50);
                    page.args["after"] = json!(cursor);
                    let response = loop {
                        let r = self.fetch(wrap(path, render(&[page.clone()])))?;
                        if cost(&r) && page.args["first"].as_u64().unwrap() > 1 {
                            page.args["first"] = json!(page.args["first"].as_u64().unwrap() / 2);
                            continue;
                        }
                        if errors(&r) {
                            return Err(r["errors"].to_string().into());
                        }
                        break r;
                    };
                    let next = &extract(&response["data"], path)?[&f.name];
                    value["edges"]
                        .as_array_mut()
                        .ok_or("missing edges")?
                        .extend(
                            next["edges"]
                                .as_array()
                                .ok_or("missing page edges")?
                                .clone(),
                        );
                    value["pageInfo"] = next["pageInfo"].clone();
                }
                let mut previous = f.args["after"].clone();
                let mut ids = BTreeSet::new();
                let mut unique = vec![];
                for mut edge in value["edges"].as_array().ok_or("missing edges")?.clone() {
                    let mut args = f.args.clone();
                    args["first"] = json!(1);
                    if !previous.is_null() {
                        args["after"] = previous;
                    }
                    let mut p = path.to_vec();
                    p.push((f.name.clone(), args, true));
                    self.complete(&f.child, &mut edge["node"], &p)?;
                    previous = edge["cursor"].clone();
                    let id = edge["node"]
                        .get("id")
                        .unwrap_or(&edge["cursor"])
                        .to_string();
                    if ids.insert(id) {
                        unique.push(edge);
                    }
                }
                value["edges"] = json!(unique);
            } else if let Some(items) = value.as_array_mut() {
                for item in items {
                    let kind = item["__typename"].as_str().unwrap_or("").to_owned();
                    let node = self.types.get(&kind).is_some_and(|t| {
                        t["interfaces"]
                            .as_array()
                            .into_iter()
                            .flatten()
                            .any(|i| i["name"] == "Node")
                    });
                    if node && !item["id"].is_null() {
                        let p = vec![
                            ("node".into(), json!({"id":item["id"]}), false),
                            (format!("... on {kind}"), json!({}), false),
                        ];
                        self.complete(&f.child, item, &p)?;
                    } else if f.name == "fields" && item["key"].is_string() {
                        let mut p = path.to_vec();
                        p.push(("field".into(), json!({"key":item["key"]}), false));
                        self.complete(&f.child, item, &p)?;
                    } else if unfinished(item) {
                        return Err("unaddressable connection".into());
                    }
                }
            } else {
                let mut p = path.to_vec();
                p.push((f.name.clone(), f.args.clone(), false));
                self.complete(&f.child, value, &p)?;
            }
        }
        Ok(())
    }
    fn enrich(&mut self, product: &mut Value) -> Result<()> {
        if self
            .disabled
            .contains(&("Product".into(), "metafields".into()))
        {
            return Ok(());
        }
        fn refs(v: &Value, out: &mut Vec<(String, String)>) {
            if matches!(
                v["__typename"].as_str(),
                Some("Metaobject" | "ProductVariant")
            ) && let (Some(id), Some(k)) = (v["id"].as_str(), v["__typename"].as_str())
            {
                out.push((id.into(), k.into()));
            }
            match v {
                Value::Object(o) => {
                    for v in o.values() {
                        refs(v, out)
                    }
                }
                Value::Array(a) => {
                    for v in a {
                        refs(v, out)
                    }
                }
                _ => (),
            }
        }
        let mut pending = vec![];
        refs(product, &mut pending);
        let mut seen = BTreeSet::new();
        let mut expanded = json!({});
        while let Some((id, kind)) = pending.pop() {
            if !seen.insert(id.clone()) {
                continue;
            }
            let mut tree = self.build(&kind, &[], 0)?;
            if kind == "ProductVariant" {
                tree.retain(|f| matches!(f.name.as_str(), "id" | "__typename" | "metafields"));
            }
            let path = vec![
                ("node".into(), json!({"id":id}), false),
                (format!("... on {kind}"), json!({}), false),
            ];
            let r = self.fetch(wrap(&path, render(&tree)))?;
            if errors(&r) {
                if r["errors"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|e| e["extensions"]["code"] == "ACCESS_DENIED")
                {
                    self.gaps.push(json!({"reference":id,"errors":r["errors"]}));
                    continue;
                }
                return Err(r["errors"].to_string().into());
            }
            let mut node = r["data"]["node"].clone();
            if node.is_null() {
                self.gaps
                    .push(json!({"reference":id,"reason":"No longer publicly visible"}));
                continue;
            }
            self.complete(&tree, &mut node, &path)?;
            let mut found = vec![];
            refs(&node, &mut found);
            pending.extend(found.into_iter().filter(|(_, k)| k == "Metaobject"));
            expanded[&id] = node;
        }
        if !expanded.as_object().unwrap().is_empty() {
            product["_expandedReferences"] = expanded;
        }
        Ok(())
    }
    fn run(&mut self, limit: Option<usize>) -> Result<Value> {
        let mut ty = "kind name".to_owned();
        for _ in 0..6 {
            ty = format!("kind name ofType {{ {ty} }}");
        }
        let schema=self.query(format!("query Schema {{ __schema {{ types {{ kind name interfaces {{ name }} possibleTypes {{ name }} fields(includeDeprecated:false) {{ name args {{ name defaultValue type {{ {ty} }} }} type {{ {ty} }} }} }} }} }}"),"Schema",json!({}))?;
        if errors(&schema) {
            return Err(schema["errors"].to_string().into());
        }
        for t in schema["data"]["__schema"]["types"]
            .as_array()
            .ok_or("missing schema")?
        {
            self.types
                .insert(t["name"].as_str().ok_or("unnamed type")?.into(), t.clone());
        }
        self.folder = self.root.join("AU-EN");
        let checkpoint = self.folder.join("checkpoint.json");
        let mut state = if checkpoint.exists() {
            serde_json::from_slice(&std::fs::read(&checkpoint)?)?
        } else {
            json!({"status":"running","after":null,"products":[],"pages":0,"gaps":[]})
        };
        if state["status"] == "complete" {
            return Ok(state);
        }
        for pair in state["disabled"].as_array().into_iter().flatten() {
            self.disabled.insert((
                pair[0].as_str().ok_or("invalid disabled type")?.into(),
                pair[1].as_str().ok_or("invalid disabled field")?.into(),
            ));
        }
        self.gaps = state["gaps"].as_array().cloned().unwrap_or_default();
        let mut count = 50;
        let mut cursors = BTreeSet::new();
        loop {
            if let Some(limit) = limit {
                let remaining = limit.saturating_sub(state["products"].as_array().unwrap().len());
                count = count.min(remaining.min(50) as i64);
            }
            let typed = self.typed_catalogue(count, state["after"].as_str().map(str::to_owned))?;
            let tree = self.build("Product", &[], 0)?;
            let after = if state["after"].is_null() {
                String::new()
            } else {
                format!(",after:{}", literal(&state["after"]))
            };
            let r=self.fetch(format!("products(first:{count},sortKey:ID{after}) {{ pageInfo {{ hasNextPage endCursor }} edges {{ cursor node {{ {} }} }} }} localization {{ country {{ isoCode currency {{ isoCode }} }} language {{ isoCode }} }}",render(&tree)))?;
            if cost(&r) && count > 1 {
                count = (count / 2).max(1);
                continue;
            }
            if errors(&r) {
                let mut removed = vec![];
                for e in r["errors"].as_array().unwrap() {
                    let pair = if let (Some(t), Some(f)) = (
                        e["extensions"]["typeName"].as_str(),
                        e["extensions"]["fieldName"].as_str(),
                    ) {
                        Some((t.to_owned(), f.to_owned()))
                    } else {
                        let mut fields = &tree;
                        let mut last = None;
                        for key in e["path"]
                            .as_array()
                            .into_iter()
                            .flatten()
                            .filter_map(Value::as_str)
                        {
                            if matches!(key, "products" | "edges" | "node" | "nodes") {
                                continue;
                            }
                            let found = fields.iter().find(|f| f.name == key).or_else(|| {
                                fields
                                    .iter()
                                    .filter(|f| f.name.starts_with("... on "))
                                    .flat_map(|f| &f.child)
                                    .find(|f| f.name == key)
                            });
                            if let Some(f) = found {
                                last = Some((f.parent.clone(), f.name.clone()));
                                fields = &f.child;
                            } else {
                                break;
                            }
                        }
                        last
                    };
                    if let Some(pair) = pair
                        && self.disabled.insert(pair.clone())
                    {
                        removed.push(pair);
                    }
                }
                self.gaps
                    .push(json!({"errors":r["errors"],"disabled":removed}));
                state["gaps"] = json!(self.gaps);
                state["disabled"] = json!(self.disabled);
                save(&checkpoint, &state)?;
                if removed.is_empty() {
                    return Err(r["errors"].to_string().into());
                }
                count = 50;
                continue;
            }
            let context = &r["data"]["localization"];
            if context["country"]["isoCode"] != "AU" || context["language"]["isoCode"] != "EN" {
                return Err("server did not honor AU/EN context".into());
            }
            state["effective_context"] = context.clone();
            save(&self.folder.join("effective-context.json"), context)?;
            let connection = &r["data"]["products"];
            let typed_ids: Vec<_> = typed["products"]["edges"]
                .as_array()
                .ok_or("missing typed edges")?
                .iter()
                .map(|e| e["node"]["id"].clone())
                .collect();
            let dynamic_ids: Vec<_> = connection["edges"]
                .as_array()
                .ok_or("missing dynamic edges")?
                .iter()
                .map(|e| e["node"]["id"].clone())
                .collect();
            if typed_ids != dynamic_ids || typed["products"]["pageInfo"] != connection["pageInfo"] {
                return Err("catalogue changed between typed traversal and dynamic enrichment; retry a fresh run".into());
            }

            for edge in connection["edges"].as_array().ok_or("missing products")? {
                let mut product = edge["node"].clone();
                let id = product["id"]
                    .as_str()
                    .ok_or("missing product id")?
                    .to_owned();
                if state["products"].as_array().unwrap().contains(&json!(id)) {
                    continue;
                }
                self.complete(
                    &tree,
                    &mut product,
                    &[("product".into(), json!({"id":id}), false)],
                )?;
                self.enrich(&mut product)?;
                if unfinished(&product) {
                    return Err("unfinished nested pagination".into());
                }
                let name = id.rsplit('/').next().ok_or("invalid product id")?;
                if name.is_empty() || name.contains(['\\', '.']) {
                    return Err("invalid product filename".into());
                }
                save(
                    &self.folder.join("products").join(format!("{name}.json")),
                    &product,
                )?;
                state["products"].as_array_mut().unwrap().push(json!(id));
                state["gaps"] = json!(self.gaps);
                save(&checkpoint, &state)?;
            }
            state["pages"] = json!(state["pages"].as_u64().unwrap() + 1);
            let reached_limit =
                limit.is_some_and(|limit| state["products"].as_array().unwrap().len() >= limit);
            if reached_limit || connection["pageInfo"]["hasNextPage"] == false {
                state["completion_reason"] = json!(if reached_limit {
                    "product_limit"
                } else {
                    "catalogue_exhausted"
                });
                state["catalogue_has_next_page"] = connection["pageInfo"]["hasNextPage"].clone();
                state["status"] = json!("complete");
                state["disabled"] = json!(self.disabled);
                state["coverage"] = json!({"graph_boundaries":self.boundaries,"unknown_metafields":"Cannot enumerate unknown namespace/key pairs","binary_media":"URLs and metadata only","private_fields":"Public Storefront visibility only","backreferences":"Finite product graph; alternative variant selectors and tracking parameters omitted"});
                save(&checkpoint, &state)?;
                return Ok(state);
            }
            let cursor = connection["pageInfo"]["endCursor"]
                .as_str()
                .filter(|s| !s.is_empty())
                .ok_or("missing catalogue cursor")?;
            if !cursors.insert(cursor.to_owned()) {
                return Err("repeated catalogue cursor".into());
            }
            state["after"] = json!(cursor);
            save(&checkpoint, &state)?;
        }
    }
}
/// Downloads the public AU/EN catalogue, retaining separate evidence for every attempt.
pub(super) fn download_ankerau(output_dir: &Path) -> Result<PathBuf> {
    download_ankerau_limited(output_dir, None)
}

pub(super) fn download_ankerau_limited(output_dir: &Path, limit: Option<usize>) -> Result<PathBuf> {
    if limit == Some(0) {
        return Err("product limit must be positive".into());
    }

    let root = output_dir.join(uuid::Uuid::now_v7().to_string());
    std::fs::create_dir_all(&root)?;
    let mut dump = Dump {
        http: reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(60))
            .user_agent("PublicCatalogueResearch/1.0")
            .build()?,
        endpoint: "https://oneanker-au.myshopify.com/api/2026-10/graphql.json".into(),
        folder: root.clone(),
        root: root.clone(),
        sequence: 0,
        types: HashMap::new(),
        disabled: BTreeSet::new(),
        boundaries: BTreeSet::new(),
        gaps: vec![],
    };
    let mut summary = json!({"status":"running","country":"AU","language":"EN","implementation":"graphql_client"});
    save(&root.join("summary.json"), &summary)?;
    summary["implementation"] = json!("graphql_client");
    summary["product_limit"] = json!(limit);
    save(&root.join("summary.json"), &summary)?;
    match dump.run(limit) {
        Ok(state) => {
            summary
                .as_object_mut()
                .unwrap()
                .extend(state.as_object().ok_or("invalid state")?.clone());
            summary["product_count"] = json!(state["products"].as_array().unwrap().len());
            save(&root.join("summary.json"), &summary)?;
            Ok(root)
        }
        Err(e) => {
            summary["status"] = json!("failed");
            summary["error"] = json!(e.to_string());
            summary["gaps"] = json!(dump.gaps);
            save(&root.join("summary.json"), &summary)?;
            Err(format!("{e}; evidence {}", root.display()).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "downloads the live AU/EN catalogue"]
    fn live_catalogue() {
        let run = download_ankerau(&std::env::temp_dir().join("graphql_client-live")).unwrap();
        println!("{}", run.display());
    }
    #[test]
    fn discovers_fields_absent_from_compiled_queries() {
        let mut d = dump();
        let schema: Value =
            serde_json::from_str(include_str!("../graphql_support/schema-fixture.json")).unwrap();
        for t in schema["data"]["__schema"]["types"].as_array().unwrap() {
            d.types
                .insert(t["name"].as_str().unwrap().into(), t.clone());
        }
        d.types.get_mut("Product").unwrap()["fields"]
            .as_array_mut()
            .unwrap()
            .push(
                json!({"name":"newPublicField","args":[],"type":{"kind":"SCALAR","name":"String"}}),
            );
        let tree = d.build("Product", &[], 0).unwrap();
        assert!(render(&tree).contains("newPublicField"));
    }
    fn dump() -> Dump {
        let root = std::env::temp_dir().join(uuid::Uuid::now_v7().to_string());
        Dump {
            http: reqwest::blocking::Client::new(),
            endpoint: String::new(),
            folder: root.clone(),
            root,
            sequence: 0,
            types: HashMap::new(),
            disabled: BTreeSet::new(),
            boundaries: BTreeSet::new(),
            gaps: vec![],
        }
    }
    #[test]
    fn schema_preserves_nested_connections_and_bounds_backreferences() {
        let mut d = dump();
        let schema: Value =
            serde_json::from_str(include_str!("../graphql_support/schema-fixture.json")).unwrap();
        for t in schema["data"]["__schema"]["types"].as_array().unwrap() {
            d.types
                .insert(t["name"].as_str().unwrap().into(), t.clone());
        }
        let tree = d.build("Product", &[], 0).unwrap();
        let collections = tree.iter().find(|f| f.name == "collections").unwrap();
        assert!(!collections.child.iter().any(|f| f.name == "products"));
        let variants = tree.iter().find(|f| f.name == "variants").unwrap();
        assert!(
            variants
                .child
                .iter()
                .any(|f| f.name == "sellingPlanAllocations")
        );
        assert!(render(&tree).contains("pageInfo { hasNextPage endCursor }"));
    }
    #[test]
    fn graphql_literals_and_inline_paths() {
        assert_eq!(literal(&json!({"id":"a\"b"})), "{id:\"a\\\"b\"}");
        let path = vec![
            ("node".into(), json!({"id":"x"}), false),
            ("... on Metafield".into(), json!({}), false),
        ];
        assert_eq!(
            extract(&json!({"node":{"id":"x"}}), &path).unwrap(),
            &json!({"id":"x"})
        );
        assert!(wrap(&path, "id".into()).contains("... on Metafield { id }"));
    }
    #[test]
    fn detects_cost_and_unfinished_nested_lists() {
        assert!(cost(
            &json!({"errors":[{"extensions":{"code":"MAX_COMPLEXITY_EXCEEDED"}}]})
        ));
        assert!(unfinished(
            &json!({"variants":[{"images":{"pageInfo":{"hasNextPage":true}}}]})
        ));
        assert!(!unfinished(&json!({"pageInfo":{"hasNextPage":false}})));
    }
    #[test]
    fn saves_partial_error_response_before_interpreting_it() {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let mut d = dump();
        d.endpoint = format!("http://{}", listener.local_addr().unwrap());
        let response = json!({"data":{"id":"x"},"errors":[{"message":"denied"}]});
        let body = response.to_string();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut bytes = [0; 8192];
            let mut received = Vec::new();
            loop {
                let n = stream.read(&mut bytes).unwrap();
                assert!(n > 0);
                received.extend_from_slice(&bytes[..n]);
                if let Some(end) = received.windows(4).position(|w| w == b"\r\n\r\n") {
                    let headers = String::from_utf8_lossy(&received[..end]).to_lowercase();
                    let length: usize = headers
                        .lines()
                        .find_map(|line| line.strip_prefix("content-length: "))
                        .unwrap()
                        .parse()
                        .unwrap();
                    if received.len() >= end + 4 + length {
                        break;
                    }
                }
            }
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            )
            .unwrap();
        });
        let result = d.query("query { id }".into(), "Dump", json!({})).unwrap();
        server.join().unwrap();
        assert_eq!(result, response);
        let evidence = d.folder.join("http/000001/attempt-1");
        assert!(evidence.join("request.json").exists());
        assert_eq!(
            serde_json::from_slice::<Value>(
                &std::fs::read(evidence.join("response.json")).unwrap()
            )
            .unwrap(),
            response
        );
        std::fs::remove_dir_all(d.root).unwrap();
    }
}

#[derive(::graphql_client::GraphQLQuery)]
#[graphql(
    schema_path = "src/platform/graphql/schema.graphql",
    query_path = "src/platform/graphql/catalogue.graphql",
    response_derives = "Debug, Serialize"
)]
struct Catalogue;
impl Dump {
    fn typed_catalogue(&mut self, first: i64, after: Option<String>) -> Result<Value> {
        use ::graphql_client::GraphQLQuery;
        let body = Catalogue::build_query(catalogue::Variables { first, after });
        let raw = self.query(
            body.query.into(),
            body.operation_name,
            serde_json::to_value(body.variables)?,
        )?;
        if errors(&raw) {
            return Err(raw["errors"].to_string().into());
        }
        let response: ::graphql_client::Response<catalogue::ResponseData> =
            serde_json::from_value(raw)?;
        Ok(serde_json::to_value(
            response.data.ok_or("missing typed catalogue data")?,
        )?)
    }
}

#[path = "graphql_client_operations.rs"]
pub(super) mod operations;
