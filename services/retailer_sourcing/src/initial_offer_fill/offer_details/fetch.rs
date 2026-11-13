use crate::initial_offer_fill::transport::Http;
use retailer_sauce::shopify::initial_fill::StorefrontToken;
#[cfg(test)]
use retailer_sauce::shopify::markets::MarketContext;
use shared::retailer::RetailerCode;
#[cfg(test)]
pub fn fetch_in_context<F: Http>(
    fetcher: &F,
    retailer: RetailerCode,
    token: Option<&StorefrontToken>,
    id: &str,
    context: Option<&MarketContext>,
) -> Result<String, String> {
    retailer_data_extractor::initial_fill_integration_for(retailer)
        .ok_or("initial fill is not configured")?
        .details(fetcher, token, context, id)
}

pub fn fetch_for_locale<F: Http>(
    fetcher: &F,
    retailer: RetailerCode,
    token: Option<&StorefrontToken>,
    id: &str,
    context: Option<&retailers_wip::initial_fill::RequestedContext>,
) -> Result<retailers_wip::initial_fill::FetchedDetails, String> {
    retailer_data_extractor::initial_fill_integration_for(retailer)
        .ok_or("initial fill is not configured")?
        .details_for_locale(fetcher, token, context, id)
}
