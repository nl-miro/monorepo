//! Starts a Shopify GraphQL initial fill for each requested retailer.

use crate::platform::bus::command_envelope;
use crate::steps::prelude::InitializeRetailerFill;
use kernel::{
    CommandError,
    prelude::{CommandGateway, NewCommandMetadata},
};
use shared::retailer::RetailerCode;
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;

/// Failure while queuing an initial offer fill.
#[derive(Debug, thiserror::Error)]
pub(crate) enum InitializationFailure {
    #[error("{0}")]
    QueueFailed(String),
}

pub struct Service {
    gateway: Arc<CommandGateway>,
}

impl Service {
    pub fn new(gateway: Arc<CommandGateway>) -> Self {
        Self { gateway }
    }

    pub fn run(&self, retailer_codes: Vec<RetailerCode>) -> Result<(), InitializationFailure> {
        for retailer_code in retailer_codes {
            let command = InitializeRetailerFill { retailer_code };
            let id = Uuid::now_v7();
            let metadata = NewCommandMetadata {
                command_id: id,
                correlation_id: Some(id),
                causation_id: None,
                source: Some("api".into()),
            };

            let result = command_envelope(&command, metadata)
                .map_err(CommandError::conversion)
                .and_then(|envelope| self.gateway.dispatch(envelope));

            if let Err(error) = result {
                error!(command_id = %id, %error, "failed to queue initial fill");
                return Err(InitializationFailure::QueueFailed(error.to_string()));
            }
        }
        Ok(())
    }
}
