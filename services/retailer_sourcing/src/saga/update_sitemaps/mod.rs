//! Saga for updating retailer sitemaps.

#[expect(dead_code, reason = "saga command lists have no callers yet")]
struct UpdateSitemapsSaga;

#[expect(dead_code, reason = "saga command lists have no callers yet")]
impl UpdateSitemapsSaga {
    pub fn commands(&self) -> Vec<String> {
        Vec::new()
    }
}
