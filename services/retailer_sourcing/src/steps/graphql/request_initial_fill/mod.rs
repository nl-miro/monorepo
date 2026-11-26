//! Accepts a storefront's initial-fill request and publishes it as an event.

use crate::platform::bus::{Bus, Command, Event, Handle};
use retailer_data_extractor::load_storefront_config;
use retailer_sauce::prelude::StorefrontStrategy;
use serde::{Deserialize, Serialize};
use shared::retailer::RetailerCode;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct RequestInitialFill {
    pub(crate) saga_id: Uuid,
    pub(crate) retailer_code: RetailerCode,
}

impl Command for RequestInitialFill {
    const NAME: &'static str = "RequestInitialFill";
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct InitialFillRequested {
    saga_id: Uuid,
    retailer_code: RetailerCode,
    strategy: StorefrontStrategy,
}

impl InitialFillRequested {
    pub fn saga_id(&self) -> Uuid {
        self.saga_id
    }

    pub fn retailer_code(&self) -> RetailerCode {
        self.retailer_code
    }

    pub fn strategy(&self) -> StorefrontStrategy {
        self.strategy
    }
}

impl Event for InitialFillRequested {
    const NAME: &'static str = "InitialFillRequested";
}

struct Handler;

impl Handle for Handler {
    type Command = RequestInitialFill;
    type Event = InitialFillRequested;
    type Error = String;

    fn handle(&self, command: RequestInitialFill) -> Result<Vec<InitialFillRequested>, String> {
        let cfg = load_storefront_config(command.retailer_code).map_err(|err| err.to_string())?;

        let strategy = cfg.access.preferred;

        let cmd = InitialFillRequested {
            saga_id: command.saga_id,
            retailer_code: command.retailer_code,
            strategy,
        };

        Ok(vec![cmd])
    }
}

pub(crate) fn register(bus: &mut Bus) {
    bus.handle(Handler);
}
