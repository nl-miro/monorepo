//! Writes every downloaded offer page in `offer_details_raw` to
//! `<data_dir>/<retailer>/offer_details/sources/<url path>.html`.
//!
//! Pages whose URL the retailer's link classifier disowns are left out. See [`disowns`].
//!
//! Usage: `DATABASE_URL=... export_offer_sources <data_dir>`

use std::collections::HashSet;
use std::panic;
use std::path::{Path, PathBuf};
use std::{env, fs, process};

use bodul_shared::link::LinkKind;
use bodul_shared::retailer::RetailerCode;
use diesel::prelude::*;
use diesel::sql_types::{Integer, Text, Uuid as SqlUuid};
use percent_encoding::percent_decode_str;
use reqwest::Url;
use uuid::Uuid;

/// Pages read per query, to keep only a few bodies in memory at once.
const BATCH_SIZE: i32 = 200;

/// Each page after `$1` (by id) with its retailer code, up to `$2` pages.
const PAGES_QUERY: &str = "
    SELECT r.id, r.url, d.body, r.retailer_code
    FROM offer_details_raw r
    JOIN raw_offers_data d ON d.id = r.id
    WHERE r.id > $1
    ORDER BY r.id
    LIMIT $2";

#[derive(QueryableByName)]
struct Page {
    #[diesel(sql_type = SqlUuid)]
    id: Uuid,
    #[diesel(sql_type = Text)]
    url: String,
    #[diesel(sql_type = Text)]
    body: String,
    #[diesel(sql_type = Text)]
    retailer_code: String,
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
        .ok_or("usage: export_offer_sources <data_dir>")?;
    let database_url = env::var("DATABASE_URL").map_err(|_| "DATABASE_URL must be set")?;
    let mut connection = PgConnection::establish(&database_url).map_err(|e| e.to_string())?;

    let mut written = 0;
    let mut disowned = 0;
    let mut skipped = 0;
    let mut unclassifiable = HashSet::new();
    let mut after = Uuid::nil();
    loop {
        let pages: Vec<Page> = diesel::sql_query(PAGES_QUERY)
            .bind::<SqlUuid, _>(after)
            .bind::<Integer, _>(BATCH_SIZE)
            .load(&mut connection)
            .map_err(|e| e.to_string())?;
        let Some(last) = pages.last() else { break };
        after = last.id;

        for page in &pages {
            match export_page(&data_dir, page, &mut unclassifiable) {
                Ok(Exported::Written) => written += 1,
                Ok(Exported::Disowned) => disowned += 1,
                Err(error) => {
                    eprintln!("skipped {}: {error}", page.url);
                    skipped += 1;
                }
            }
        }
    }

    println!(
        "wrote {written} pages to {}, left out {disowned} disowned by their retailer, skipped {skipped}",
        data_dir.display()
    );
    Ok(())
}

enum Exported {
    Written,
    Disowned,
}

fn export_page(
    data_dir: &Path,
    page: &Page,
    unclassifiable: &mut HashSet<RetailerCode>,
) -> Result<Exported, String> {
    let retailer =
        RetailerCode::try_from(page.retailer_code.as_str()).map_err(|e| e.to_string())?;
    if disowns(retailer, &page.url, unclassifiable) {
        return Ok(Exported::Disowned);
    }

    write_page(data_dir, retailer, page)?;
    Ok(Exported::Written)
}

/// Whether `retailer`'s link classifier calls `url` none of its business.
///
/// `offer_details_raw` still holds pages discovered before a classifier learned to reject the other
/// markets on a shared domain, such as `www.anker.com/de/...` filed under `ankerau`. Only
/// `NotInterested` counts: no sitemap is at hand here, so a classifier that needs one may answer
/// `Unknown` for a page that is the retailer's own.
///
/// A classifier that is not written yet panics; that retailer's pages are all kept, and it is
/// only asked once.
fn disowns(retailer: RetailerCode, url: &str, unclassifiable: &mut HashSet<RetailerCode>) -> bool {
    if unclassifiable.contains(&retailer) {
        return false;
    }

    match panic::catch_unwind(|| retailer_data_extractor::classify_link_for(retailer, url, "", 0)) {
        Ok(kind) => kind == Some(LinkKind::NotInterested),
        Err(_) => {
            unclassifiable.insert(retailer);
            false
        }
    }
}

fn write_page(data_dir: &Path, retailer: RetailerCode, page: &Page) -> Result<(), String> {
    let dir = data_dir
        .join(retailer.as_str())
        .join("offer_details")
        .join("sources");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let path = dir.join(file_name(&page.url)?);
    fs::write(&path, &page.body).map_err(|e| format!("{}: {e}", path.display()))
}

/// The decoded URL path with `/` replaced by `-`, as in the data repos:
/// `https://www.anker.com/au/products/a1256` becomes `au-products-a1256.html`.
fn file_name(url: &str) -> Result<String, String> {
    let url = Url::parse(url).map_err(|e| e.to_string())?;
    // Decoded, since percent-encoded non-Latin paths exceed the 255-byte limit
    // on file names.
    let path = percent_decode_str(url.path()).decode_utf8_lossy();
    let path = path.trim_matches('/').replace('/', "-");
    if path.is_empty() {
        return Err("the URL has no path".to_string());
    }
    Ok(format!("{path}.html"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_a_file_after_the_url_path() {
        assert_eq!(
            file_name("https://www.anker.com/au/products/a1256?variant=1").unwrap(),
            "au-products-a1256.html"
        );
    }

    #[test]
    fn decodes_a_percent_encoded_path() {
        assert_eq!(
            file_name("https://ankerkorea.co.kr/product/%EC%95%A4%EC%BB%A4-a3936/93/").unwrap(),
            "product-앤커-a3936-93.html"
        );
    }

    #[test]
    fn another_markets_page_is_disowned() {
        let mut unclassifiable = HashSet::new();

        assert!(disowns(
            RetailerCode::AnkerAu,
            "https://www.anker.com/de/products/a1256",
            &mut unclassifiable
        ));
        assert!(disowns(
            RetailerCode::AnkerCom,
            "https://www.anker.com/ca-fr/products/a1256",
            &mut unclassifiable
        ));
    }

    #[test]
    fn the_retailers_own_page_is_kept() {
        let mut unclassifiable = HashSet::new();

        assert!(!disowns(
            RetailerCode::AnkerAu,
            "https://www.anker.com/au/products/a1256",
            &mut unclassifiable
        ));
        assert!(!disowns(
            RetailerCode::AnkerCom,
            "https://www.anker.com/products/a1256",
            &mut unclassifiable
        ));
    }

    #[test]
    fn an_unwritten_classifier_keeps_every_page() {
        let mut unclassifiable = HashSet::new();

        assert!(!disowns(
            RetailerCode::MiCom,
            "https://www.mi.com/global/product/x",
            &mut unclassifiable
        ));
        assert!(unclassifiable.contains(&RetailerCode::MiCom));
    }

    #[test]
    fn rejects_a_url_without_a_path() {
        assert!(file_name("https://www.anker.com/").is_err());
    }
}
