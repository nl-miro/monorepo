use std::time::Duration;

use bodul_shared::link::LinkKind;
/// Retailer identifier accepted by [`architecture_for`].
use bodul_shared::prelude::RetailerCode;
use bodul_shared::{NormalizedData, SitemapConfig};
use html_extractor::io::Architecture;

/// Returns the registered extraction architecture for a retailer.
///
/// Returns an I/O error with kind `NotFound` when no architecture is registered.
pub fn architecture_for(retailer_code: RetailerCode) -> Result<Architecture, std::io::Error> {
    let architecture = retailers_anker::prelude::architecture_for(retailer_code)
        .or_else(|| retailers_mi::prelude::architecture_for(retailer_code))
        .or_else(|| retailers_minisforum::prelude::architecture_for(retailer_code))
        .or_else(|| retailers_ugreen::prelude::architecture_for(retailer_code))
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("no architecture for retailer {retailer_code:?}"),
            )
        })?;

    Ok(architecture)
}

pub fn normalize_for(
    retailer_code: RetailerCode,
    json: &str,
) -> Option<Result<NormalizedData, String>> {
    retailers_anker::prelude::normalize_for(retailer_code, json)
        .or_else(|| retailers_mi::prelude::normalize_for(retailer_code, json))
        .or_else(|| retailers_minisforum::prelude::normalize_for(retailer_code, json))
        .or_else(|| retailers_ugreen::prelude::normalize_for(retailer_code, json))
}

pub fn sitemap_config_for(retailer_code: &RetailerCode) -> Option<SitemapConfig> {
    retailers_anker::prelude::sitemap_config_for(retailer_code)
        .or_else(|| retailers_mi::prelude::sitemap_config_for(retailer_code))
        .or_else(|| retailers_minisforum::prelude::sitemap_config_for(retailer_code))
        .or_else(|| retailers_ugreen::prelude::sitemap_config_for(retailer_code))
}

/// Whether `retailer_code`'s crawl downloads the child sitemap at `url`.
/// Returns `None` for a retailer this crate doesn't know.
pub fn sitemap_greylisted_for(retailer_code: RetailerCode, url: &str) -> Option<bool> {
    retailers_anker::prelude::sitemap_greylisted_for(retailer_code, url)
        .or_else(|| retailers_mi::prelude::sitemap_greylisted_for(retailer_code, url))
        .or_else(|| retailers_minisforum::prelude::sitemap_greylisted_for(retailer_code, url))
        .or_else(|| retailers_ugreen::prelude::sitemap_greylisted_for(retailer_code, url))
}

/// How long `retailer_code`'s crawl pauses between groups of sitemap
/// downloads. Returns `None` for a retailer this crate doesn't know.
pub fn sitemap_pause_for(retailer_code: RetailerCode) -> Option<Duration> {
    retailers_anker::prelude::sitemap_pause_for(retailer_code)
        .or_else(|| retailers_mi::prelude::sitemap_pause_for(retailer_code))
        .or_else(|| retailers_minisforum::prelude::sitemap_pause_for(retailer_code))
        .or_else(|| retailers_ugreen::prelude::sitemap_pause_for(retailer_code))
}

pub fn classify_link_for(
    retailer_code: RetailerCode,
    url: &str,
    source: &str,
    image_count: usize,
) -> Option<LinkKind> {
    retailers_anker::prelude::classify_link_for(retailer_code, url, source, image_count)
        .or_else(|| {
            retailers_mi::prelude::classify_link_for(retailer_code, url, source, image_count)
        })
        .or_else(|| {
            retailers_minisforum::prelude::classify_link_for(
                retailer_code,
                url,
                source,
                image_count,
            )
        })
        .or_else(|| {
            retailers_ugreen::prelude::classify_link_for(retailer_code, url, source, image_count)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_retailer_with_sitemaps_downloads_every_child_by_default() {
        for retailer_code in [
            RetailerCode::AnkerAu,
            RetailerCode::MiCom,
            RetailerCode::MinisForumAu,
            RetailerCode::UgreenCa,
        ] {
            let greylisted = sitemap_greylisted_for(retailer_code, "https://shop.test/sitemap.xml");
            assert_eq!(greylisted, Some(true), "{retailer_code:?}");
        }
    }

    #[test]
    fn pauses_longer_for_stores_that_rate_limit() {
        for retailer_code in [
            RetailerCode::UgreenCom,
            RetailerCode::UgreenEu,
            RetailerCode::UgreenNasEu,
        ] {
            assert_eq!(
                sitemap_pause_for(retailer_code),
                Some(Duration::from_secs(5)),
                "{retailer_code:?}"
            );
        }
        for retailer_code in [
            RetailerCode::AnkerAu,
            RetailerCode::MiCom,
            RetailerCode::MinisForumAu,
            RetailerCode::UgreenCa,
        ] {
            assert_eq!(
                sitemap_pause_for(retailer_code),
                Some(Duration::from_secs(1)),
                "{retailer_code:?}"
            );
        }
    }
}
