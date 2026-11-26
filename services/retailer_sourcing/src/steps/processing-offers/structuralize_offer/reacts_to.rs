use crate::steps::prelude::{OfferWasNormalized, StructuralizeOffer};

/// Structuralizes every offer as soon as it is normalized.

impl From<OfferWasNormalized> for StructuralizeOffer {
    fn from(event: OfferWasNormalized) -> Self {
        Self {
            storage: event.storage,
            scraped_at: event.scraped_at,
            offer_id: event.offer_id,
            consolidated_sitemap_id: event.consolidated_sitemap_id,
            retailer_code: event.retailer_code,
            url: event.url,
        }
    }
}

#[cfg(test)]
mod tests {
    use shared::retailer::RetailerCode;
    use uuid::Uuid;

    use super::*;

    #[test]
    fn structuralizes_the_offer_that_was_normalized() {
        let event = OfferWasNormalized {
            storage: None,
            scraped_at: None,
            offer_id: Uuid::now_v7(),
            consolidated_sitemap_id: Uuid::now_v7(),
            retailer_code: RetailerCode::AnkerAu,
            url: "https://shop.test/products/a".to_string(),
        };

        let expected = StructuralizeOffer {
            storage: event.storage,
            scraped_at: event.scraped_at,
            offer_id: event.offer_id,
            consolidated_sitemap_id: event.consolidated_sitemap_id,
            retailer_code: RetailerCode::AnkerAu,
            url: event.url.clone(),
        };
        assert_eq!(StructuralizeOffer::from(event), expected);
    }
}
