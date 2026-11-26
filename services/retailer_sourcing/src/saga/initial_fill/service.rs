//! Application service for initial-fill sagas.

use crate::platform::bus::Dispatcher;
use crate::steps::prelude::RequestInitialFill;
use shared::retailer::RetailerCodes;
use uuid::Uuid;

pub(crate) struct Service {
    command_gateway: Dispatcher,
}

impl Service {
    pub(crate) fn new(command_gateway: Dispatcher) -> Self {
        Self { command_gateway }
    }

    pub(crate) async fn run(&self, retailer_codes: Vec<String>) {
        let retailer_codes: RetailerCodes = retailer_codes.try_into().unwrap();

        for retailer_code in retailer_codes.0 {
            let cmd = RequestInitialFill {
                saga_id: Uuid::now_v7(),
                retailer_code,
            };

            let _ = self.command_gateway.dispatch(&cmd, "api/v1/initial-fill");
        }
    }
}
