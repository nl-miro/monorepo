//! Writes each retailer's latest sitemap crawl to `<data_dir>/<retailer>/sitemap`, one folder
//! per stage, each named for the crawl's retrieval id:
//!
//! - `sources/<retrieval id>/<url path>.xml`, every sitemap file as fetched,
//! - `processed/<retrieval id>.json`, the files parsed into one sitemap tree,
//! - `consolidated/<retrieval id>.json`, the tree's URLs grouped by kind of page, when the crawl
//!   has been consolidated.
//!
//! Usage: `DATABASE_URL=... export_sitemaps <data_dir>`

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::{env, fs, process};

use bodul_shared::retailer::RetailerCode;
use diesel::prelude::*;
use diesel::sql_types::{Jsonb, Nullable, Text, Uuid as SqlUuid};
use percent_encoding::percent_decode_str;
use reqwest::Url;
use serde_json::Value;
use uuid::Uuid;

/// Each retailer's most recently processed crawl, with its consolidated content if it has one.
const CRAWLS_QUERY: &str = include_str!("export_sitemaps.sql");

/// The sitemap files fetched by the crawl `$1`.
const SOURCES_QUERY: &str = "
    SELECT r.url, d.body
    FROM sitemaps_raw r
    JOIN raw_sitemap_documents_data d ON d.id = r.id
    WHERE r.retrieval_id = $1
    ORDER BY r.url";

#[derive(QueryableByName)]
struct Crawl {
    #[diesel(sql_type = Text)]
    retailer_code: String,
    #[diesel(sql_type = SqlUuid)]
    retrieval_id: Uuid,
    #[diesel(sql_type = Jsonb)]
    document: Value,
    #[diesel(sql_type = Nullable<Jsonb>)]
    content: Option<Value>,
}

#[derive(QueryableByName)]
struct Source {
    #[diesel(sql_type = Text)]
    url: String,
    #[diesel(sql_type = Text)]
    body: String,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let data_dir = env::args()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("usage: export_sitemaps <data_dir>")?;
    let database_url = env::var("DATABASE_URL").map_err(|_| "DATABASE_URL must be set")?;
    let mut connection = PgConnection::establish(&database_url).map_err(|e| e.to_string())?;

    let crawls: Vec<Crawl> = diesel::sql_query(CRAWLS_QUERY)
        .load(&mut connection)
        .map_err(|e| e.to_string())?;

    let mut exported = 0;
    let mut skipped = 0;
    for crawl in &crawls {
        let sources: Vec<Source> = diesel::sql_query(SOURCES_QUERY)
            .bind::<SqlUuid, _>(crawl.retrieval_id)
            .load(&mut connection)
            .map_err(|e| e.to_string())?;

        match write_crawl(&data_dir, crawl, &sources) {
            Ok(()) => {
                println!(
                    "{} {}: {} sources, {}",
                    crawl.retailer_code,
                    crawl.retrieval_id,
                    sources.len(),
                    if crawl.content.is_some() {
                        "processed and consolidated"
                    } else {
                        "processed, not consolidated yet"
                    }
                );
                exported += 1;
            }
            Err(error) => {
                eprintln!("skipped {}: {error}", crawl.retailer_code);
                skipped += 1;
            }
        }
    }

    println!(
        "exported {exported} sitemaps to {}, skipped {skipped}",
        data_dir.display()
    );
    Ok(())
}

fn write_crawl(data_dir: &Path, crawl: &Crawl, sources: &[Source]) -> Result<(), String> {
    let retailer =
        RetailerCode::try_from(crawl.retailer_code.as_str()).map_err(|e| e.to_string())?;
    let dir = data_dir.join(retailer.as_str()).join("sitemap");

    // Named before anything is written, so a crawl whose URLs collide leaves no partial export.
    let mut named = BTreeMap::new();
    for source in sources {
        let name = file_name(&source.url)?;
        if let Some(other) = named.insert(name.clone(), source) {
            return Err(format!(
                "{} and {} would both be written to {name}",
                other.url, source.url
            ));
        }
    }

    let retrieval_id = crawl.retrieval_id.to_string();
    let sources_dir = dir.join("sources").join(&retrieval_id);
    fs::create_dir_all(&sources_dir).map_err(|e| e.to_string())?;
    for (name, source) in named {
        write(&sources_dir.join(name), &source.body)?;
    }

    write_json(&dir.join("processed"), &retrieval_id, &crawl.document)?;
    if let Some(content) = &crawl.content {
        write_json(&dir.join("consolidated"), &retrieval_id, content)?;
    }
    Ok(())
}

fn write_json(dir: &Path, retrieval_id: &str, value: &Value) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let json = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    write(&dir.join(format!("{retrieval_id}.json")), &json)
}

fn write(path: &Path, contents: &str) -> Result<(), String> {
    fs::write(path, contents).map_err(|e| format!("{}: {e}", path.display()))
}

/// The decoded URL path with `/` replaced by `-`, as for offer pages, followed by the query with
/// `&` replaced by `_`, since Shopify pages its sitemaps by query:
/// `https://www.ankerjapan.com/sitemap_products_1.xml?from=1&to=2` becomes
/// `sitemap_products_1_from=1_to=2.xml`.
fn file_name(url: &str) -> Result<String, String> {
    let url = Url::parse(url).map_err(|e| e.to_string())?;
    // Decoded, since percent-encoded non-Latin paths exceed the 255-byte limit
    // on file names.
    let path = percent_decode_str(url.path()).decode_utf8_lossy();
    let path = path.trim_matches('/').replace('/', "-");
    if path.is_empty() {
        return Err("the URL has no path".to_string());
    }

    let stem = path.strip_suffix(".xml").unwrap_or(&path);
    match url.query().filter(|query| !query.is_empty()) {
        Some(query) => Ok(format!("{stem}_{}.xml", query.replace('&', "_"))),
        None => Ok(format!("{stem}.xml")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_a_file_after_the_url_path() {
        assert_eq!(
            file_name("https://www.anker.com/server-sitemap-index-products.xml").unwrap(),
            "server-sitemap-index-products.xml"
        );
        assert_eq!(
            file_name("https://www.anker.com/au/sitemap/products.xml").unwrap(),
            "au-sitemap-products.xml"
        );
    }

    #[test]
    fn keeps_the_query_that_pages_a_shopify_sitemap() {
        assert_eq!(
            file_name(
                "https://www.ankerjapan.com/sitemap_products_1.xml?from=5917111746721&to=7761509187745"
            )
            .unwrap(),
            "sitemap_products_1_from=5917111746721_to=7761509187745.xml"
        );
    }

    #[test]
    fn gives_a_file_without_an_xml_extension_one() {
        assert_eq!(
            file_name("https://example.com/sitemap").unwrap(),
            "sitemap.xml"
        );
    }

    #[test]
    fn rejects_a_url_without_a_path() {
        assert!(file_name("https://www.anker.com/").is_err());
    }
}
