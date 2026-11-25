# 92. Business processes, sagas and steps in retailer sourcing

Status: Accepted

When a retailer comes on board, the business wants an initial set of its
offers. A Shopify storefront can expose those offers through GraphQL. Another
way to collect them is to retrieve the retailer's sitemaps and download the
offer pages they identify.

Represent that outcome once as `business_process::retailer_initial_fill`.
Keep the choice of collection technique in the saga implementation. A GraphQL
saga can fetch catalog identifiers and then fetch details for those products.
A sitemap-and-HTML saga can retrieve, process and consolidate sitemaps before
downloading offer pages.

Each saga composes small actions from `steps`. For example, a sitemap update
can reuse the same retrieval step as an initial fill. The step owns the work
of retrieving a sitemap. Each saga decides what follows it.
