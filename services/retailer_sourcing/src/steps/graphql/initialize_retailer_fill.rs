//! Persists the initial saga snapshot before reporting a retailer fill initialized.

use crate::platform::bus::{Command, Event, HandleCorrelated};
use crate::saga::saga_repository::SagaRepository;
use crate::saga::shopify_graphql_initial_fill::ShopifyGraphQlInitialFillSaga;
use serde::{Deserialize, Serialize};
use shared::retailer::RetailerCode;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeRetailerFill {
    pub retailer_code: RetailerCode,
}
impl Command for InitializeRetailerFill {
    const NAME: &'static str = "InitializeRetailerFill";
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetailerFillInitialized {
    pub retailer_code: RetailerCode,
}
impl Event for RetailerFillInitialized {
    const NAME: &'static str = "RetailerFillInitialized";
}

pub(crate) struct Handler(pub(crate) SagaRepository);

impl HandleCorrelated for Handler {
    type Command = InitializeRetailerFill;
    type Event = RetailerFillInitialized;

    fn handle(
        &self,
        command: InitializeRetailerFill,
        correlation: Uuid,
    ) -> Result<RetailerFillInitialized, String> {
        let retailer = command.retailer_code;
        self.0
            .initialize(correlation, retailer, || {
                ShopifyGraphQlInitialFillSaga::snapshot(retailer)
            })
            .map_err(|error| error.to_string())?;
        Ok(RetailerFillInitialized {
            retailer_code: retailer,
        })
    }
}
