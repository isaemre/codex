#![allow(clippy::expect_used)]
//! Codex's own transports must refuse everything but the compiled-in address (K-27).
//!
//! The denied requests fail before proxy resolution, DNS, or any socket work, so these tests do
//! not need network access. A broken lock shows up as a different error or a real connection.

use codex_http_client::ClientRouteClass;
use codex_http_client::HttpClient;
use codex_http_client::HttpClientBuilder;
use codex_http_client::HttpClientFactory;
use codex_http_client::HttpError;
use codex_http_client::HttpTransport;
use codex_http_client::NetworkPolicy;
use codex_http_client::NetworkPolicyDenied;
use codex_http_client::OutboundProxyPolicy;
use codex_http_client::Request;
use codex_http_client::ReqwestTransport;
use codex_http_client::RouteAwareClientPool;
use codex_http_client::TransportError;
use codex_makas_config::LOCKED;
use codex_makas_config::llm_base_url;
use codex_websocket_client::WebSocketConnector;
use http::Method;
use pretty_assertions::assert_eq;
use tokio_tungstenite::tungstenite::Error as WebSocketError;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;
use url::Url;

const DENIED_URLS: [&str; 4] = [
    "https://api.openai.com/v1/responses",
    "https://chatgpt.com/backend-api/codex/responses",
    "https://attacker.example/v1/responses",
    "http://127.0.0.1:11434/v1/responses",
];

fn factory() -> HttpClientFactory {
    HttpClientFactory::new(OutboundProxyPolicy::ReqwestDefault)
}

fn direct_client() -> HttpClient {
    HttpClientBuilder::new()
        .build_direct()
        .expect("build direct client")
}

fn routed_client() -> HttpClient {
    RouteAwareClientPool::new(factory(), ClientRouteClass::Api).into_client()
}

#[test]
fn network_policy_allows_only_the_llm_address() {
    if !LOCKED {
        return;
    }
    let policy = NetworkPolicy::unmanaged();
    for url in DENIED_URLS {
        let url = Url::parse(url).expect("parse URL");
        assert_eq!(
            policy.acquire(&url).err(),
            Some(NetworkPolicyDenied::Destination),
            "{url}"
        );
    }
    let llm = Url::parse(&format!("{}/responses", llm_base_url())).expect("parse LLM URL");
    assert!(policy.acquire(&llm).is_ok());
}

#[test]
fn sdks_without_destination_checks_are_disabled() {
    if !LOCKED {
        return;
    }
    assert_eq!(
        NetworkPolicy::unmanaged()
            .acquire_for_unsupported_sdk()
            .err(),
        Some(NetworkPolicyDenied::UnsupportedTransport)
    );
}

#[tokio::test]
async fn http_clients_refuse_other_destinations() {
    if !LOCKED {
        return;
    }
    for client in [direct_client(), routed_client()] {
        for url in DENIED_URLS {
            let error = client.post(url).send().await.expect_err("request denied");
            assert!(
                matches!(error, HttpError::Policy(NetworkPolicyDenied::Destination)),
                "{url}: {error:?}"
            );
        }
    }
}

#[tokio::test]
async fn transport_refuses_other_destinations() {
    if !LOCKED {
        return;
    }
    let transport = ReqwestTransport::from_http_client(direct_client());
    for url in DENIED_URLS {
        let error = transport
            .execute(Request::new(Method::POST, url.to_string()))
            .await
            .expect_err("request denied");
        assert!(
            matches!(
                error,
                TransportError::Policy(NetworkPolicyDenied::Destination)
            ),
            "{url}: {error:?}"
        );
    }
}

#[tokio::test]
async fn websocket_connector_refuses_other_destinations() {
    if !LOCKED {
        return;
    }
    let connector = WebSocketConnector::new(&factory()).expect("build connector");
    let request = "wss://api.openai.com/v1/responses"
        .into_client_request()
        .expect("build request");
    let error = connector
        .connect(request, WebSocketConfig::default())
        .await
        .expect_err("connection denied");
    assert!(
        matches!(&error, WebSocketError::Io(io) if io.kind() == std::io::ErrorKind::PermissionDenied),
        "{error:?}"
    );
}
