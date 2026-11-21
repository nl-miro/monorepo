//! Starts a Shopify GraphQL initial fill for each requested retailer.

use crate::platform::bus::command_envelope;
use crate::steps::graphql::initialize_retailer_fill::validate_retailer;
use crate::steps::prelude::InitializeRetailerFill;
use kernel::{
    CommandError,
    prelude::{CommandGateway, NewCommandMetadata},
};
use serde::Serialize;
use shared::retailer::RetailerCode;
use std::sync::Arc;
use tracing::error;
use utoipa::ToSchema;
use uuid::Uuid;

/// Failure while validating or queuing an initial offer fill.
#[derive(Debug, thiserror::Error)]
pub(crate) enum InitializationFailure {
    #[error("retailers must not be empty")]
    EmptyRetailers,
    #[error("unknown retailer code: {0}")]
    UnknownRetailer(String),
    #[error("duplicate retailer code: {0}")]
    DuplicateRetailer(String),
    #[error("{0}")]
    QueueFailed(String),
    #[error("unsupported retailer {0}: {1}")]
    UnsupportedRetailer(String, String),
}

#[derive(Serialize, ToSchema)]
pub(crate) struct FillReceipt {
    retailer_code: String,
    #[schema(value_type = String)]
    correlation_id: Uuid,
}

#[derive(Serialize, ToSchema)]
pub(crate) struct InitialFills {
    fills: Vec<FillReceipt>,
}

pub struct Service {
    gateway: Arc<CommandGateway>,
}

impl Service {
    pub fn new(gateway: Arc<CommandGateway>) -> Self {
        Self { gateway }
    }

    pub fn run(
        &self,
        retailer_codes: Vec<RetailerCode>,
    ) -> Result<InitialFills, InitializationFailure> {
        for retailer in &retailer_codes {
            validate_retailer(*retailer).map_err(|error| {
                InitializationFailure::UnsupportedRetailer(retailer.as_str(), error)
            })?;
        }
        let mut fills = Vec::new();
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
            fills.push(FillReceipt {
                retailer_code: retailer_code.as_str(),
                correlation_id: id,
            });
        }
        Ok(InitialFills { fills })
    }
}
