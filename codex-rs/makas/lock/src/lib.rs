//! The Makas provider lock (K-07, K-17, K-27).
//!
//! `codex-core` calls [`apply`] after it has resolved every config source, so `config.toml`,
//! `-c`, profiles, `--oss`, requirements, and per-thread overrides all end up here.

use std::collections::HashMap;

use codex_makas_config::LOCKED;
use codex_makas_config::llm_base_url;
use codex_model_provider_info::ModelProviderInfo;
use codex_model_provider_info::WireApi;
use codex_model_provider_info::create_oss_provider_with_base_url;

/// The id of the only model provider that exists in a locked build.
pub const MAKAS_PROVIDER_ID: &str = "makas";

const MAKAS_PROVIDER_NAME: &str = "Makas";

/// Builds the only provider Makas can use.
///
/// The name must differ from OpenAI's so that `is_openai()` stays false; otherwise Codex would
/// route compaction and other calls to backend routes the gateway does not serve.
pub fn locked_provider() -> ModelProviderInfo {
    ModelProviderInfo {
        name: MAKAS_PROVIDER_NAME.to_string(),
        // Keeps the API key login flow, which stores the personal gateway key (K-21).
        requires_openai_auth: true,
        ..create_oss_provider_with_base_url(llm_base_url(), WireApi::Responses)
    }
}

/// Replaces the resolved providers and selection with the locked provider.
pub fn apply(
    providers: HashMap<String, ModelProviderInfo>,
    provider_id: String,
) -> (HashMap<String, ModelProviderInfo>, String) {
    if !LOCKED {
        return (providers, provider_id);
    }
    (
        HashMap::from([(MAKAS_PROVIDER_ID.to_string(), locked_provider())]),
        MAKAS_PROVIDER_ID.to_string(),
    )
}
