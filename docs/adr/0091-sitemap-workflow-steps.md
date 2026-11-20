# 91. Sitemap workflow steps

Status: Accepted

This decision supersedes the sitemap module location in [ADR 4](0004-feature-slices-for-retailer-sourcing.md).

Retrieval, processing and consolidation are reusable workflow steps. They live together under `retailer_sourcing/src/steps/sitemap/`, alongside the GraphQL steps. Each step owns its handler and storage queries. The three steps share the retailer registry and Diesel schema. Offer discovery consumes their final `SitemapConsolidated` event through the steps prelude.

The HTTP endpoint for starting sitemap retrievals is retired. Workers continue to handle queued commands with the existing message names, payloads and tables.
