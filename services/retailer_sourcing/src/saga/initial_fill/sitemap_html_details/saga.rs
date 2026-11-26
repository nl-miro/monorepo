//! Entities for the sitemap and HTML details initial-fill saga.

use crate::saga::prelude::RecordableSaga;
use crate::saga::record::{DieselPgSagaRecord, SagaRecordError, SagaTypeMismatch};
use crate::steps::prelude::{
    ConsolidateSitemap, DiscoverOffers, DownloadOfferPage, ExtractOfferPage, NormalizeOffer,
    OfferDiscovered, OfferPageWasDownloaded, OfferPageWasExtracted, OfferWasNormalized,
    ProcessSitemap, RetrieveSitemap, SitemapConsolidated, SitemapProcessed, SitemapRetrieved,
    StructuralizeOffer,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use shared::retailer::RetailerCode;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub(crate) struct SitemapHtmlDetailsInitialFillSaga {
    saga_id: Uuid,
    correlation_id: Uuid,
    retailer_code: RetailerCode,
}

impl RecordableSaga for SitemapHtmlDetailsInitialFillSaga {}

impl SitemapHtmlDetailsInitialFillSaga {
    pub(crate) const SAGA_TYPE: &'static str = "SitemapHtmlDetailsInitialFill";

    pub(super) fn saga_id(&self) -> Uuid {
        self.saga_id
    }

    pub(crate) fn new(saga_id: Uuid, correlation_id: Uuid, retailer_code: RetailerCode) -> Self {
        Self {
            saga_id,
            correlation_id,
            retailer_code,
        }
    }
}

pub(crate) enum SitemapHtmlDetailsCommand {
    RetrieveSitemap(RetrieveSitemap),
    ProcessSitemap(ProcessSitemap),
    ConsolidateSitemap(ConsolidateSitemap),
    DiscoverOffers(DiscoverOffers),
    DownloadOfferPage(DownloadOfferPage),
    ExtractOfferPage(ExtractOfferPage),
    NormalizeOffer(NormalizeOffer),
    StructuralizeOffer(StructuralizeOffer),
}

#[rustfmt::skip]
impl SitemapHtmlDetailsInitialFillSaga {
    pub fn on_initial_fill_requested(&self) -> SitemapHtmlDetailsCommand {
        let cmd = RetrieveSitemap::new(self.retailer_code);

        SitemapHtmlDetailsCommand::RetrieveSitemap(cmd)
    }

    pub(super) fn on_sitemap_retrieved(&self, event: SitemapRetrieved) -> SitemapHtmlDetailsCommand {
        let command = ProcessSitemap::from(event);

        SitemapHtmlDetailsCommand::ProcessSitemap(command)
    }

    pub(super) fn on_sitemap_processed(&self, event: SitemapProcessed) -> SitemapHtmlDetailsCommand {
        let command = ConsolidateSitemap::from(event);

        SitemapHtmlDetailsCommand::ConsolidateSitemap(command)
    }

    pub(super) fn on_sitemap_consolidated(&self, event: SitemapConsolidated) -> Vec<SitemapHtmlDetailsCommand> {

        if event.product_count == 0 {
            return Vec::new();
        }

        let command = DiscoverOffers::from(event);

        vec![SitemapHtmlDetailsCommand::DiscoverOffers(command)]
    }

    pub(super) fn on_offer_discovered(&self, event: OfferDiscovered) -> SitemapHtmlDetailsCommand {

        let command = DownloadOfferPage::from(event);

        SitemapHtmlDetailsCommand::DownloadOfferPage(command)
    }

    pub(super) fn on_offer_page_was_downloaded(&self, event: OfferPageWasDownloaded) -> SitemapHtmlDetailsCommand {

        let command = ExtractOfferPage::from(event);

        SitemapHtmlDetailsCommand::ExtractOfferPage(command)
    }

    pub(super) fn on_offer_page_was_extracted(&self, event: OfferPageWasExtracted) -> SitemapHtmlDetailsCommand {

        let command = NormalizeOffer::from(event);

        SitemapHtmlDetailsCommand::NormalizeOffer(command)
    }

    pub(super) fn on_offer_was_normalized(&self, event: OfferWasNormalized) -> SitemapHtmlDetailsCommand {

        let command = StructuralizeOffer::from(event);

        SitemapHtmlDetailsCommand::StructuralizeOffer(command)
    }
}

impl TryFrom<SitemapHtmlDetailsInitialFillSaga> for DieselPgSagaRecord {
    type Error = SagaTypeMismatch;

    fn try_from(fill: SitemapHtmlDetailsInitialFillSaga) -> Result<Self, Self::Error> {
        let res = DieselPgSagaRecord {
            saga_id: fill.saga_id,
            correlation_id: fill.correlation_id,
            saga_type: SitemapHtmlDetailsInitialFillSaga::SAGA_TYPE.into(),
            payload: json!({"retailer_code": fill.retailer_code}),
        };

        Ok(res)
    }
}

impl TryFrom<DieselPgSagaRecord> for SitemapHtmlDetailsInitialFillSaga {
    type Error = SagaRecordError;

    fn try_from(record: DieselPgSagaRecord) -> Result<Self, Self::Error> {
        if record.saga_type != Self::SAGA_TYPE {
            let err = SagaTypeMismatch::new(Self::SAGA_TYPE, record.saga_type);

            return Err(err.into());
        }

        let retailer_code = RetailerCode::deserialize(&record.payload["retailer_code"])?;
        let res = Self::new(record.saga_id, record.correlation_id, retailer_code);

        Ok(res)
    }
}
