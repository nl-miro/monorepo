SELECT DISTINCT ON (p.retailer_code)
        p.retailer_code, p.retrieval_id, pd.document, cd.content
    FROM sitemaps_processed p
    JOIN processed_sitemaps_data pd ON pd.id = p.id
    LEFT JOIN sitemaps_consolidated c ON (c."references" ->> 'processed_sitemap_id')::UUID = p.id
    LEFT JOIN consolidated_sitemaps_data cd ON cd.id = c.id
    ORDER BY p.retailer_code, p.processed_at DESC
