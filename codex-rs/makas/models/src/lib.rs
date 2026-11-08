//! Lists the models the company gateway serves, for the onboarding model step (K-30).
//!
//! The request goes through `codex-http-client`, so the destination lock (K-27) applies to it.
//! The gateway answers in the OpenAI `/models` shape (`{"data":[{"id":...}]}`), which the
//! upstream `codex-api` models client cannot decode, so the parsing lives here.

use std::time::Duration;

use codex_http_client::ClientRouteClass;
use codex_http_client::HttpClientFactory;
use codex_http_client::RouteAwareClientPool;
use codex_makas_config::llm_base_url;
use serde_json::Value;
use thiserror::Error;

const MODELS_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ModelsError {
    /// The gateway rejected the key (HTTP 401 or 403).
    #[error("the gateway rejected the API key")]
    Unauthorized,
    /// The gateway could not be reached, or the destination lock refused the request.
    #[error("could not reach the gateway")]
    Unreachable,
    /// The gateway answered with an error status or a body this crate cannot read.
    #[error("the gateway returned an unexpected response")]
    BadResponse,
}

/// Fetches the model ids from the compiled gateway address, authenticated with `api_key`.
///
/// `factory` must come from the running config so proxy routing matches the rest of the app.
pub async fn fetch_model_ids(
    factory: HttpClientFactory,
    api_key: &str,
) -> Result<Vec<String>, ModelsError> {
    let client = RouteAwareClientPool::with_connect_timeout(
        factory,
        ClientRouteClass::Other,
        MODELS_TIMEOUT,
    );
    let response = client
        .get(models_url(llm_base_url()))
        .bearer_auth(api_key)
        .timeout(MODELS_TIMEOUT)
        .send()
        .await
        .map_err(|_| ModelsError::Unreachable)?;

    check_status(response.status().as_u16())?;
    let body = response
        .text()
        .await
        .map_err(|_| ModelsError::BadResponse)?;
    parse_model_ids(&body)
}

fn models_url(base: &str) -> String {
    format!("{}/models", base.trim_end_matches('/'))
}

fn check_status(status: u16) -> Result<(), ModelsError> {
    match status {
        200..=299 => Ok(()),
        401 | 403 => Err(ModelsError::Unauthorized),
        _ => Err(ModelsError::BadResponse),
    }
}

/// Reads the `id` of each entry in `data`, keeping the gateway's order and dropping duplicates.
///
/// A body without a `data` array is treated as an unexpected response, not as an empty list,
/// so a misconfigured gateway is not mistaken for one with no models.
fn parse_model_ids(body: &str) -> Result<Vec<String>, ModelsError> {
    let value: Value = serde_json::from_str(body).map_err(|_| ModelsError::BadResponse)?;
    let data = value
        .get("data")
        .and_then(Value::as_array)
        .ok_or(ModelsError::BadResponse)?;

    let mut ids: Vec<String> = Vec::new();
    for entry in data {
        let Some(id) = entry.get("id").and_then(Value::as_str) else {
            continue;
        };
        let id = id.trim();
        if !id.is_empty() && !ids.iter().any(|existing| existing == id) {
            ids.push(id.to_string());
        }
    }
    Ok(ids)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn models_url_joins_without_double_slash() {
        assert_eq!(
            models_url("https://gateway.example.com/v1/"),
            "https://gateway.example.com/v1/models"
        );
        assert_eq!(
            models_url("https://gateway.example.com"),
            "https://gateway.example.com/models"
        );
    }

    #[test]
    fn parse_reads_ids_in_order() {
        let body = r#"{"object":"list","data":[{"id":"qwen-coder"},{"id":"gpt-oss-20b"}]}"#;
        assert_eq!(
            parse_model_ids(body),
            Ok(vec!["qwen-coder".to_string(), "gpt-oss-20b".to_string()])
        );
    }

    #[test]
    fn parse_drops_duplicates_blank_and_missing_ids() {
        let body = r#"{"data":[{"id":"a"},{"id":"a"},{"id":"  "},{"name":"x"},{"id":" b "}]}"#;
        assert_eq!(
            parse_model_ids(body),
            Ok(vec!["a".to_string(), "b".to_string()])
        );
    }

    #[test]
    fn parse_accepts_empty_data_array() {
        assert_eq!(parse_model_ids(r#"{"data":[]}"#), Ok(Vec::new()));
    }

    #[test]
    fn parse_rejects_non_json_and_missing_data() {
        assert_eq!(parse_model_ids("<html>"), Err(ModelsError::BadResponse));
        assert_eq!(
            parse_model_ids(r#"{"models":[{"slug":"x"}]}"#),
            Err(ModelsError::BadResponse)
        );
    }

    #[test]
    fn status_classification() {
        assert_eq!(check_status(200), Ok(()));
        assert_eq!(check_status(204), Ok(()));
        assert_eq!(check_status(401), Err(ModelsError::Unauthorized));
        assert_eq!(check_status(403), Err(ModelsError::Unauthorized));
        assert_eq!(check_status(404), Err(ModelsError::BadResponse));
        assert_eq!(check_status(500), Err(ModelsError::BadResponse));
    }
}
