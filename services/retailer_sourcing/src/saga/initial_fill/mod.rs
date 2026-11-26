//! Initial-fill sagas.

mod service;
pub(super) mod shopify_graphql;
mod sitemap_html_details;

pub(crate) use service::Service;
pub(crate) use sitemap_html_details::SitemapHtmlDetailsInitialFillSaga;

use crate::platform::Platform;
use crate::platform::bus::Bus;
use crate::saga::prelude::DieselPgSagaRepository;
use shopify_graphql::Listener as ShopifyGraphQlListener;
use sitemap_html_details::Listener as SitemapHtmlDetailsListener;
use std::sync::Arc;

pub(crate) fn register_initial_fill_listeners(bus: &mut Bus, platform: &Platform) {
    let repository = Arc::new(DieselPgSagaRepository::new(platform.db.clone()));
    let shopify = Arc::new(ShopifyGraphQlListener::new(Arc::clone(&repository)));
    let sitemap = Arc::new(SitemapHtmlDetailsListener::new(repository));

    bus.listen_command_method(
        "shopify_graphql_initial_fill_requested",
        Arc::clone(&shopify),
        ShopifyGraphQlListener::on_initial_fill_requested,
    );
    bus.listen_command_method(
        "shopify_graphql_storefront_token_acquired",
        Arc::clone(&shopify),
        ShopifyGraphQlListener::on_storefront_token_acquired,
    );
    bus.listen_command_method(
        "shopify_graphql_product_catalog_ids_retrieved",
        Arc::clone(&shopify),
        ShopifyGraphQlListener::on_product_catalog_ids_retrieved,
    );
    bus.listen_command_method(
        "shopify_graphql_offer_details_fetched",
        Arc::clone(&shopify),
        ShopifyGraphQlListener::on_offer_details_fetched,
    );
    bus.listen_command_method(
        "shopify_graphql_offer_details_extracted",
        shopify,
        ShopifyGraphQlListener::on_offer_details_extracted,
    );
    bus.listen_command_method(
        "sitemap_html_details_initial_fill_requested",
        Arc::clone(&sitemap),
        SitemapHtmlDetailsListener::on_initial_fill_requested,
    );
    bus.listen_command_method(
        "sitemap_html_details_on_sitemap_retrieved",
        Arc::clone(&sitemap),
        SitemapHtmlDetailsListener::on_sitemap_retrieved,
    );
    bus.listen_command_method(
        "sitemap_html_details_on_sitemap_processed",
        Arc::clone(&sitemap),
        SitemapHtmlDetailsListener::on_sitemap_processed,
    );
    bus.listen_command_method(
        "sitemap_html_details_on_sitemap_consolidated",
        Arc::clone(&sitemap),
        SitemapHtmlDetailsListener::on_sitemap_consolidated,
    );
    bus.listen_command_method(
        "sitemap_html_details_on_offer_discovered",
        Arc::clone(&sitemap),
        SitemapHtmlDetailsListener::on_offer_discovered,
    );
    bus.listen_command_method(
        "sitemap_html_details_on_offer_page_was_downloaded",
        Arc::clone(&sitemap),
        SitemapHtmlDetailsListener::on_offer_page_was_downloaded,
    );
    bus.listen_command_method(
        "sitemap_html_details_on_offer_page_was_extracted",
        Arc::clone(&sitemap),
        SitemapHtmlDetailsListener::on_offer_page_was_extracted,
    );
    bus.listen_command_method(
        "sitemap_html_details_on_offer_was_normalized",
        sitemap,
        SitemapHtmlDetailsListener::on_offer_was_normalized,
    );
}
