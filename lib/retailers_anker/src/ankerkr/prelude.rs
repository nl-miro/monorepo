pub use super::offer_details::v1::{
    architecture as offer_detail_architecture_v1,
    normalize as offer_detail_normalize_v1, //
};
#[allow(unused_imports)] // Stubs awaiting v2 pipeline integration.
pub use super::offer_details::v2::{
    architecture as offer_detail_architecture_v2,
    normalize as offer_detail_normalize_v2, //
};
pub use super::sitemap::v1::{
    classify_link as classify_link_v1,
    sitemap_config as sitemap_config_v1,
    sitemap_greylisted as sitemap_greylisted_v1,
    sitemap_pause as sitemap_pause_v1, //
};
