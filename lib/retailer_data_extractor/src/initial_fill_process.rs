use shared::prelude::RetailerCode;

/// A step in the initial offer fill process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    DiscoverMarkets,
    AcquireStorefrontToken,
    ProductCatalogIdentifiers,
    OfferDetails,
}

/// The steps in a retailer's initial offer fill process.
#[derive(Debug, Clone)]
pub struct InitialFillProcess {
    pub steps: Vec<Step>,
}

/// Derive steps from the connected integration; unsupported retailers have no steps.
pub fn initial_fill_process(retailer_code: RetailerCode) -> InitialFillProcess {
    let Some(config) = crate::initial_fill_config_for(retailer_code) else {
        return InitialFillProcess { steps: Vec::new() };
    };
    let mut steps = Vec::new();
    if config.attempts_token() {
        steps.push(Step::AcquireStorefrontToken);
    }
    if crate::all_market_contexts(retailer_code) {
        steps.push(Step::DiscoverMarkets);
    }
    steps.extend([Step::ProductCatalogIdentifiers, Step::OfferDetails]);
    InitialFillProcess { steps }
}
