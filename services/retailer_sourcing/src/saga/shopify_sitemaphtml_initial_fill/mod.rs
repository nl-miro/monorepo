//! Shopify Sitemaphtml initial fill saga.

#[expect(dead_code, reason = "saga command lists have no callers yet")]
pub struct ShopifySitemapHthmlInitialFillSaga;

#[expect(dead_code, reason = "saga command lists have no callers yet")]
impl ShopifySitemapHthmlInitialFillSaga {
    pub fn commands(&self) -> Vec<String> {
        vec![
            "RetrieveSitemap".into(),
            "ProcessSitemap".into(),
            "ConsolidateSitemap".into(),
            "DownloadOfferPage".into(),
            "ExtractOfferDetails".into(),
            "NormalizeOfferDetails".into(),
            "StructuralizeOfferDetails".into(),
        ]
    }
}
