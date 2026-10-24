//! Temporary HTTP outbox transport. Only durable inbox acceptance completes a row.

use std::{env::VarError, io, time::Duration};

use inbox_api::io::{SubmitInboxRequest, SubmitInboxResponse};
use outbox::io::{OutboundMessageEnvelope, OutboxError, OutboxPublisherPort};
use reqwest::{Client, StatusCode, Url};
use tokio::runtime::Handle;
use uuid::Uuid;

const TIMEOUT: Duration = Duration::from_secs(10);
const MAX_ACK_BYTES: usize = 64 * 1024;
const DESTINATION: &str = "offers";
const COMMAND: &str = "ImportOfferDetails";

#[derive(Clone)]
pub struct OffersPublisher {
    client: Client,
    inbox_url: Url,
    runtime: Handle,
    timeout: Duration,
}

impl OffersPublisher {
    pub fn from_env(runtime: Handle) -> io::Result<Option<Self>> {
        Self::from_value(std::env::var("OFFERS_URL"), runtime)
    }

    fn from_value(value: Result<String, VarError>, runtime: Handle) -> io::Result<Option<Self>> {
        match value {
            Err(VarError::NotPresent) => Ok(None),
            Ok(origin) => Self::new(&origin, runtime).map(Some),
            Err(_) => Err(err_configuration()),
        }
    }

    pub fn new(origin: &str, runtime: Handle) -> io::Result<Self> {
        Self::with_timeout(origin, runtime, TIMEOUT)
    }

    fn with_timeout(origin: &str, runtime: Handle, timeout: Duration) -> io::Result<Self> {
        let inbox_url = inbox_url(origin)?;
        let client = Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .retry(reqwest::retry::never())
            .timeout(timeout)
            .build()
            .map_err(|_| err_client_configuration())?;
        Ok(Self {
            client,
            inbox_url,
            runtime,
            timeout,
        })
    }

    async fn deliver(
        &self,
        request: SubmitInboxRequest,
        expected: Uuid,
    ) -> Result<(), OutboxError> {
        let body = serde_json::to_vec(&request).map_err(|_| err_payload())?;
        let mut response = self
            .client
            .post(self.inbox_url.clone())
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(body)
            .send()
            .await
            .map_err(|_| err_transport())?;
        if response.status() != StatusCode::ACCEPTED {
            return Err(err_rejected());
        }
        if response
            .content_length()
            .is_some_and(|length| length > MAX_ACK_BYTES as u64)
        {
            return Err(err_acknowledgement());
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(|_| err_transport())? {
            if chunk.len() > MAX_ACK_BYTES - bytes.len() {
                return Err(err_acknowledgement());
            }
            bytes.extend_from_slice(&chunk);
        }
        let response: SubmitInboxResponse =
            serde_json::from_slice(&bytes).map_err(|_| err_acknowledgement())?;
        if Uuid::parse_str(&response.message_id).ok() != Some(expected) {
            return Err(err_acknowledgement());
        }
        Ok(())
    }
}

impl OutboxPublisherPort for OffersPublisher {
    /// Called on Mulac's blocking worker; the application owns the async runtime.
    fn publish(&self, envelope: OutboundMessageEnvelope) -> Result<(), OutboxError> {
        let expected = envelope.metadata.message_id;
        let request = submission(envelope)?;
        self.runtime.block_on(async {
            tokio::time::timeout(self.timeout, self.deliver(request, expected))
                .await
                .map_err(|_| err_timeout())?
        })
    }
}

fn submission(envelope: OutboundMessageEnvelope) -> Result<SubmitInboxRequest, OutboxError> {
    if envelope.metadata.routing_key != DESTINATION {
        return Err(err_destination());
    }
    let payload: serde_json::Value =
        serde_json::from_slice(&envelope.payload).map_err(|_| err_payload())?;
    if !payload.is_object() {
        return Err(err_payload());
    }
    let request = SubmitInboxRequest {
        payload: Some(payload),
        routing_key: COMMAND.into(),
        message_id: Some(envelope.metadata.message_id.to_string()),
        correlation_id: envelope.metadata.correlation_id.map(|id| id.to_string()),
        source: Some(
            envelope
                .metadata
                .source
                .unwrap_or_else(|| "retailer_sourcing".into()),
        ),
    };
    request.validate().map_err(|_| err_payload())?;
    Ok(request)
}

fn inbox_url(origin: &str) -> io::Result<Url> {
    let mut url = Url::parse(origin).map_err(|_| err_configuration())?;
    if origin.trim() != origin
        || origin.bytes().any(|byte| byte.is_ascii_control())
        || !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.path() != "/"
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(err_configuration());
    }
    url.set_path("/api/inbox");
    Ok(url)
}

fn err_configuration() -> io::Error {
    io::Error::other(
        "OFFERS_URL must be an HTTP(S) origin without credentials, path, query or fragment",
    )
}
fn err_client_configuration() -> io::Error {
    io::Error::other("offers HTTP client could not be configured")
}
fn err_payload() -> OutboxError {
    OutboxError::Conversion("outbox message must contain a valid offers inbox submission".into())
}
fn err_destination() -> OutboxError {
    OutboxError::Routing("outbox message destination must be offers".into())
}
fn err_transport() -> OutboxError {
    OutboxError::Transport("offers inbox request failed".into())
}
fn err_timeout() -> OutboxError {
    OutboxError::Transport("offers inbox request timed out".into())
}
fn err_rejected() -> OutboxError {
    OutboxError::Transport("offers inbox did not return durable acceptance".into())
}
fn err_acknowledgement() -> OutboxError {
    OutboxError::Transport("offers inbox acknowledgement is invalid".into())
}

#[cfg(test)]
pub(crate) mod tests;
