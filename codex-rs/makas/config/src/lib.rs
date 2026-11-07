//! Compile-time Makas settings: the LLM address and the destination check built on it (K-07).
//!
//! This is a leaf crate because `codex-http-client` depends on it. It reads no environment
//! variable and no config at runtime; the values below are fixed when the binary is built.

use std::sync::LazyLock;

use url::Url;

/// False only in test builds made with `MAKAS_UNLOCK_FOR_UPSTREAM_TESTS`, which `build.rs`
/// refuses for release builds (K-28).
pub const LOCKED: bool = !cfg!(makas_unlocked);

const LLM_BASE_URL: &str = env!("MAKAS_COMPILED_LLM_BASE_URL");

static LLM_URL: LazyLock<Option<Url>> = LazyLock::new(|| Url::parse(LLM_BASE_URL).ok());

/// The only LLM address this binary can talk to.
pub fn llm_base_url() -> &'static str {
    LLM_BASE_URL
}

/// Whether Makas may open a connection to `destination`.
///
/// Fails closed: if the compiled address cannot be parsed, nothing is allowed.
pub fn is_destination_allowed(destination: &Url) -> bool {
    if !LOCKED {
        return true;
    }
    LLM_URL
        .as_ref()
        .is_some_and(|allowed| destination_matches(allowed, destination))
}

/// Compares origins exactly. There is no suffix, wildcard, or trailing-dot normalization.
fn destination_matches(allowed: &Url, destination: &Url) -> bool {
    let scheme_matches = matches!(
        (allowed.scheme(), destination.scheme()),
        ("https", "https" | "wss") | ("http", "http" | "ws")
    );
    scheme_matches
        && destination.username().is_empty()
        && destination.password().is_none()
        && destination.host_str().is_some()
        && destination.host_str() == allowed.host_str()
        && destination.port_or_known_default() == allowed.port_or_known_default()
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
