//! Validates a GraphQL fill request without accessing saga storage.

use crate::platform::bus::{Command, Event, Handle};
use crate::v2::prelude::InitialFillContexts;
use retailer_data_extractor::get_storefront_config;
use serde::{Deserialize, Serialize};
use shared::retailer::RetailerCode;

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

pub(crate) fn validate_retailer(retailer: RetailerCode) -> Result<(), String> {
    let config =
        get_storefront_config(retailer).ok_or("retailer has no storefront configuration")?;
    config.initial_fill_contexts()?;
    Ok(())
}

pub(crate) struct Handler;
impl Handle for Handler {
    type Command = InitializeRetailerFill;
    type Event = RetailerFillInitialized;
    type Error = String;
    fn handle(&self, command: Self::Command) -> Result<Vec<Self::Event>, String> {
        validate_retailer(command.retailer_code)?;
        Ok(vec![RetailerFillInitialized {
            retailer_code: command.retailer_code,
        }])
    }
}

#[cfg(test)]
mod tests {
    use super::{Handler, InitializeRetailerFill, RetailerFillInitialized};
    use crate::platform::bus::Handle;
    use shared::retailer::RetailerCode;
    #[test]
    fn initialization_validates_without_a_database() {
        assert_eq!(
            Handler
                .handle(InitializeRetailerFill {
                    retailer_code: RetailerCode::AnkerAu
                })
                .unwrap(),
            vec![RetailerFillInitialized {
                retailer_code: RetailerCode::AnkerAu
            }]
        );
        assert!(
            Handler
                .handle(InitializeRetailerFill {
                    retailer_code: RetailerCode::AnkerKr
                })
                .is_err()
        );
    }
}
