use super::*;
use axum::{
    Json, Router,
    response::IntoResponse,
    routing::{any, post},
};
use outbox::io::OutboxEntryMetadata;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

pub(crate) struct TestServer {
    pub url: String,
    task: tokio::task::JoinHandle<()>,
}

impl Drop for TestServer {
    fn drop(&mut self) {
        self.task.abort();
    }
}

pub(crate) async fn serve(app: Router) -> TestServer {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let task = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    TestServer { url, task }
}

pub(crate) fn envelope() -> OutboundMessageEnvelope {
    OutboundMessageEnvelope {
        payload: serde_json::to_vec(&json!({
            "retailer_code":"AnkerAu", "title":"USB charger",
            "price":{"amount":9007199254740993_i64,"currency":"AUD"},
            "url":"https://shop.example/Charger?variant=2#offer"
        }))
        .unwrap(),
        metadata: OutboxEntryMetadata {
            message_id: Uuid::now_v7(),
            event_id: Uuid::now_v7(),
            correlation_id: Some(Uuid::now_v7()),
            causation_id: Some(Uuid::now_v7()),
            event_type: "OfferExported".into(),
            routing_key: DESTINATION.into(),
            source: Some("retailer_sourcing".into()),
            content_type: Some("application/json".into()),
        },
    }
}

async fn publish(
    publisher: OffersPublisher,
    message: OutboundMessageEnvelope,
) -> Result<(), OutboxError> {
    tokio::task::spawn_blocking(move || publisher.publish(message))
        .await
        .unwrap()
}

#[test]
fn configuration_accepts_only_origins_and_does_not_leak_input() {
    for origin in [
        "http://localhost",
        "https://offers.example/",
        "http://[::1]:3100",
    ] {
        let url = inbox_url(origin).unwrap();
        assert_eq!(url.path(), "/api/inbox");
    }
    for origin in [
        "",
        "not a URL",
        "ftp://example.com",
        "http://user:secret@example.com",
        "http://example.com/api",
        "http://example.com/?token=secret",
        "http://example.com/#secret",
        " http://example.com",
        "http://example.com\n",
    ] {
        assert_eq!(
            inbox_url(origin).unwrap_err().to_string(),
            err_configuration().to_string()
        );
    }
    let runtime = tokio::runtime::Runtime::new().unwrap();
    assert!(
        OffersPublisher::from_value(Err(VarError::NotPresent), runtime.handle().clone())
            .unwrap()
            .is_none()
    );
    assert!(OffersPublisher::from_value(Ok(String::new()), runtime.handle().clone()).is_err());
    #[cfg(unix)]
    {
        use std::{ffi::OsString, os::unix::ffi::OsStringExt};
        let invalid = VarError::NotUnicode(OsString::from_vec(vec![0xff]));
        assert!(OffersPublisher::from_value(Err(invalid), runtime.handle().clone()).is_err());
    }
}

#[test]
fn rejects_bad_destinations_and_payloads_before_http() {
    let mut message = envelope();
    message.metadata.routing_key = "customer_facing".into();
    assert!(matches!(submission(message), Err(OutboxError::Routing(_))));
    for payload in [b"null".as_slice(), b"[]", b"{}", b"\"text\"", b"{"] {
        let mut message = envelope();
        message.payload = payload.to_vec();
        assert!(submission(message).is_err());
    }
    let mut message = envelope();
    message.metadata.source = Some(" ".into());
    assert!(submission(message).is_err());
    let mut message = envelope();
    message.metadata.source = None;
    assert_eq!(
        submission(message).unwrap().source.as_deref(),
        Some("retailer_sourcing")
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn preserves_the_command_and_delivery_identity_across_retries() {
    let seen = Arc::new(Mutex::new(Vec::<Value>::new()));
    let captured = seen.clone();
    let app = Router::new().route(
        "/api/inbox",
        post(move |Json(body): Json<Value>| {
            let captured = captured.clone();
            async move {
                captured.lock().unwrap().push(body.clone());
                (
                    StatusCode::ACCEPTED,
                    Json(json!({"message_id":body["message_id"]})),
                )
            }
        }),
    );
    let server = serve(app).await;
    let publisher = OffersPublisher::new(&server.url, Handle::current()).unwrap();
    let message = envelope();
    publish(publisher.clone(), message.clone()).await.unwrap();
    publish(publisher, message.clone()).await.unwrap();
    let requests = seen.lock().unwrap();
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0], requests[1]);
    assert_eq!(
        requests[0],
        json!({
            "payload":serde_json::from_slice::<Value>(&message.payload).unwrap(),
            "routing_key":"ImportOfferDetails", "message_id":message.metadata.message_id,
            "correlation_id":message.metadata.correlation_id, "source":"retailer_sourcing"
        })
    );
    assert_ne!(message.metadata.message_id, message.metadata.event_id);
}

#[tokio::test(flavor = "multi_thread")]
async fn rejects_non_acceptance_without_redirects_or_automatic_retries() {
    for status in [200, 201, 204, 302, 400, 429, 500] {
        let calls = Arc::new(Mutex::new(0));
        let captured = calls.clone();
        let redirects = Arc::new(Mutex::new(0));
        let redirected = redirects.clone();
        let app = Router::new()
            .route(
                "/api/inbox",
                post(move || {
                    let captured = captured.clone();
                    async move {
                        *captured.lock().unwrap() += 1;
                        (
                            StatusCode::from_u16(status).unwrap(),
                            [("location", "/must-not-follow")],
                            "private response body",
                        )
                    }
                }),
            )
            .route(
                "/must-not-follow",
                any(move || {
                    let redirected = redirected.clone();
                    async move {
                        *redirected.lock().unwrap() += 1;
                        StatusCode::OK
                    }
                }),
            );
        let server = serve(app).await;
        let publisher = OffersPublisher::new(&server.url, Handle::current()).unwrap();
        let error = publish(publisher, envelope()).await.unwrap_err();
        assert_eq!(error.to_string(), err_rejected().to_string());
        assert_eq!(*calls.lock().unwrap(), 1);
        assert_eq!(*redirects.lock().unwrap(), 0);
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn rejects_invalid_oversized_and_mismatched_acknowledgements() {
    for body in [
        "{".to_owned(),
        "{}".into(),
        "{\"message_id\":\"wrong\"}".into(),
        json!({"message_id":Uuid::now_v7()}).to_string(),
        " ".repeat(MAX_ACK_BYTES + 1),
    ] {
        let app = Router::new().route(
            "/api/inbox",
            post(move || {
                let body = body.clone();
                async move { (StatusCode::ACCEPTED, body) }
            }),
        );
        let server = serve(app).await;
        let publisher = OffersPublisher::new(&server.url, Handle::current()).unwrap();
        assert_eq!(
            publish(publisher, envelope())
                .await
                .unwrap_err()
                .to_string(),
            err_acknowledgement().to_string()
        );
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn bounds_total_time_and_reports_connection_failure() {
    let server = serve(Router::new().route(
        "/api/inbox",
        post(|| async {
            tokio::time::sleep(Duration::from_secs(1)).await;
            StatusCode::ACCEPTED.into_response()
        }),
    ))
    .await;
    let publisher =
        OffersPublisher::with_timeout(&server.url, Handle::current(), Duration::from_millis(30))
            .unwrap();
    let outcome = tokio::time::timeout(Duration::from_millis(500), publish(publisher, envelope()))
        .await
        .expect("publisher deadline must fire before the delayed response");
    assert!(outcome.is_err());
    drop(server);
    // Release an ephemeral listener so the subsequent connection is refused.
    let unavailable = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("http://{}", unavailable.local_addr().unwrap());
    drop(unavailable);
    let publisher = OffersPublisher::new(&address, Handle::current()).unwrap();
    assert_eq!(
        publish(publisher, envelope())
            .await
            .unwrap_err()
            .to_string(),
        err_transport().to_string()
    );
}

async fn chunked_response(chunks: Vec<(Duration, Vec<u8>)>) -> TestServer {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let task = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut request = Vec::new();
        let mut buffer = [0_u8; 1024];
        while !request.windows(4).any(|bytes| bytes == b"\r\n\r\n") {
            let size = stream.read(&mut buffer).await.unwrap();
            if size == 0 {
                return;
            }
            request.extend_from_slice(&buffer[..size]);
        }
        if stream.write_all(b"HTTP/1.1 202 Accepted\r\nContent-Type: application/json\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n").await.is_err() { return; }
        for (delay, bytes) in chunks {
            tokio::time::sleep(delay).await;
            let mut chunk = format!("{:X}\r\n", bytes.len()).into_bytes();
            chunk.extend_from_slice(&bytes);
            chunk.extend_from_slice(b"\r\n");
            if stream.write_all(&chunk).await.is_err() {
                return;
            }
        }
        let _ = stream.write_all(b"0\r\n\r\n").await;
    });
    TestServer { url, task }
}

#[tokio::test(flavor = "multi_thread")]
async fn bounds_chunked_acknowledgements_without_content_length() {
    let message = envelope();
    let mut acknowledgement = json!({"message_id":message.metadata.message_id})
        .to_string()
        .into_bytes();
    acknowledgement.resize(MAX_ACK_BYTES + 1, b' ');
    let server = chunked_response(vec![(Duration::ZERO, acknowledgement)]).await;
    let publisher = OffersPublisher::new(&server.url, Handle::current()).unwrap();
    assert_eq!(
        publish(publisher, message).await.unwrap_err().to_string(),
        err_acknowledgement().to_string()
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn deadline_covers_a_slow_acknowledgement_body() {
    let message = envelope();
    let final_chunk = format!("\"{}\"}}", message.metadata.message_id).into_bytes();
    let server = chunked_response(vec![
        (Duration::ZERO, b"{\"message_id\":".to_vec()),
        (Duration::from_secs(1), final_chunk),
    ])
    .await;
    let publisher =
        OffersPublisher::with_timeout(&server.url, Handle::current(), Duration::from_millis(30))
            .unwrap();
    let outcome = tokio::time::timeout(Duration::from_millis(500), publish(publisher, message))
        .await
        .expect("the deadline must cover reading a streamed response body");
    assert!(outcome.is_err());
}
