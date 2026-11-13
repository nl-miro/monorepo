//! Inventory-backed request contexts; observed artifact contexts retain their existing format.
use crate::retailers::{Market, Retailer};
use retailer_sauce::initial_fill::{CatalogBatch, Http, InitialFillIntegration};
use retailer_sauce::shopify::{initial_fill::StorefrontToken, markets::MarketContext};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestedContext {
    pub country: Option<String>,
    pub language: String,
    /// Historical queued contexts retain their expected currency.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    /// Remember Shopify's default across batches without sending it as a request country.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_country: Option<String>,
}
impl From<&MarketContext> for RequestedContext {
    fn from(context: &MarketContext) -> Self {
        Self {
            country: Some(context.country.clone()),
            language: context.language.clone(),
            currency: Some(context.currency.clone()),
            observed_country: None,
        }
    }
}
impl RequestedContext {
    /// Validate response evidence and retain the default country for subsequent requests.
    pub fn observe(&mut self, response: &Value) -> Result<(), String> {
        let country = response
            .pointer("/extensions/context/country")
            .and_then(Value::as_str)
            .filter(|s| s.len() == 2 && s.bytes().all(|c| c.is_ascii_uppercase()))
            .ok_or("Storefront omitted valid country evidence")?;
        let language = response
            .pointer("/extensions/context/language")
            .and_then(Value::as_str)
            .ok_or("Storefront omitted language evidence")?;
        if language != self.language
            || self.country.as_deref().is_some_and(|c| c != country)
            || self
                .observed_country
                .as_deref()
                .is_some_and(|c| c != country)
        {
            return Err("Storefront response changed the requested or observed context".into());
        }
        if self.country.is_none() {
            self.observed_country = Some(country.to_owned());
        }
        Ok(())
    }
}
impl Retailer {
    /// Snapshot the main-table locales in declaration order, collapsing request aliases.
    pub fn initial_fill_contexts(&self) -> Result<Vec<RequestedContext>, String> {
        let mut contexts = Vec::new();
        for locale in self.supported_locales() {
            let country = match locale.country_region {
                Some(Market::Country(country)) => Some(country.as_str().to_owned()),
                Some(Market::Region(_)) => return Err("GraphQL country cannot be a region".into()),
                None => None,
            };
            let context = RequestedContext {
                country,
                language: locale.language.as_str().to_owned(),
                currency: None,
                observed_country: None,
            };
            if !contexts.contains(&context) {
                contexts.push(context);
            }
        }
        if contexts.is_empty() {
            return Err("retailer has no supported locales".into());
        }
        Ok(contexts)
    }
}
pub struct LocaleCatalogBatch {
    pub batch: CatalogBatch,
    pub context: Option<RequestedContext>,
}
pub struct FetchedDetails {
    pub body: String,
    pub context: Option<MarketContext>,
}
/// Local extension of the published integration interface for optional-country requests.
pub trait LocaleInitialFillIntegration: InitialFillIntegration {
    fn catalog_for_locale(
        &self,
        http: &dyn Http,
        token: Option<&StorefrontToken>,
        context: Option<&RequestedContext>,
        after: Option<String>,
    ) -> Result<LocaleCatalogBatch, String>;
    fn details_for_locale(
        &self,
        http: &dyn Http,
        token: Option<&StorefrontToken>,
        context: Option<&RequestedContext>,
        id: &str,
    ) -> Result<FetchedDetails, String>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::retailers::{self, RetailerCode};
    use serde_json::json;

    #[test]
    fn inventory_contexts_exclude_advertised_entries_and_preserve_countryless_languages() {
        let contexts = |code| {
            retailers::get_retailer(code)
                .unwrap()
                .initial_fill_contexts()
                .unwrap()
        };
        let ca = contexts(RetailerCode::AnkerCa);
        assert_eq!(
            ca.iter()
                .map(|c| (c.country.as_deref(), c.language.as_str()))
                .collect::<Vec<_>>(),
            [(Some("CA"), "EN"), (Some("CA"), "FR")]
        );
        assert_eq!(
            contexts(RetailerCode::AnkerEu)[0].country.as_deref(),
            Some("AT")
        );
        assert_eq!(contexts(RetailerCode::AnkerDe).len(), 1);
        let eu = contexts(RetailerCode::MinisForumEu);
        assert_eq!(
            eu.iter()
                .map(|c| (c.country.as_deref(), c.language.as_str()))
                .collect::<Vec<_>>(),
            [(None, "EN")]
        );
        let main = contexts(RetailerCode::UgreenCom);
        assert_eq!(main.len(), 2);
        assert!(main.iter().all(|c| c.currency.is_none()));
        let mut empty = retailers::get_retailer(RetailerCode::AnkerAu).unwrap();
        empty.supported_locales.clear();
        assert!(empty.initial_fill_contexts().is_err());
        let mut region = retailers::get_retailer(RetailerCode::AnkerAu).unwrap();
        region.supported_locales[0].country_region = Market::from_code("EU");
        assert!(region.initial_fill_contexts().is_err());
    }

    #[test]
    fn historical_contexts_retain_currency_and_new_contexts_validate_observation() {
        let old = json!({"country":"CA","language":"FR","currency":"CAD"});
        let context: RequestedContext = serde_json::from_value(old.clone()).unwrap();
        assert_eq!(serde_json::to_value(context).unwrap(), old);
        let mut context: RequestedContext =
            serde_json::from_value(json!({"country":null,"language":"DE"})).unwrap();
        assert!(context.observe(&json!({})).is_err());
        assert!(
            context
                .observe(&json!({"extensions":{"context":{"country":"DE","language":"EN"}}}))
                .is_err()
        );
        context
            .observe(&json!({"extensions":{"context":{"country":"DE","language":"DE"}}}))
            .unwrap();
        assert_eq!(context.country, None);
        assert_eq!(context.observed_country.as_deref(), Some("DE"));
        assert!(
            context
                .observe(&json!({"extensions":{"context":{"country":"AT","language":"DE"}}}))
                .is_err()
        );
    }
}
