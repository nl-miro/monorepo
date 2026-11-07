use bodul_shared::retailer::RetailerCode;
use chrono::{DateTime, Utc};
use money::Money;

use super::retailer_offer_id::RetailerOfferId;

#[derive(Debug, PartialEq)]
pub struct HistoricalRetailerOfferState {
    pub(super) facts: super::event::OfferFacts,
    pub(super) identifiers: retailer_sauce::identifier::OfferIdentifiers,
    pub(super) localization: retailer_sauce::localization::Localization,
    id: RetailerOfferId,
    retailer_code: RetailerCode,
    title: String,
    price: Money,
    url: String,
    images: Vec<String>,
    created_at: DateTime<Utc>,
    scraped_at: Option<DateTime<Utc>>,
}

impl HistoricalRetailerOfferState {
    pub fn new(
        id: RetailerOfferId,
        retailer_code: RetailerCode,
        title: impl Into<String>,
        price: Money,
        url: impl Into<String>,
        created_at: DateTime<Utc>,
    ) -> Self {
        let title = title.into();
        let facts = super::event::OfferFacts {
            content: retailer_sauce::offer_content::singleton(title.clone(), None, None),
            ..Default::default()
        };
        Self {
            facts,
            identifiers: Default::default(),
            localization: Default::default(),
            id,
            retailer_code,
            title,
            price,
            url: url.into(),
            images: Vec::new(),
            scraped_at: None,
            created_at,
        }
    }

    pub fn scraped_at(&self) -> Option<DateTime<Utc>> {
        self.scraped_at
    }

    pub fn images(&self) -> &[String] {
        &self.images
    }

    pub fn id(&self) -> &RetailerOfferId {
        &self.id
    }

    pub fn retailer_code(&self) -> &RetailerCode {
        &self.retailer_code
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn price(&self) -> &Money {
        &self.price
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }
}
