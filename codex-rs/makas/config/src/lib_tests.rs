use pretty_assertions::assert_eq;
use url::Url;

use super::destination_matches;

fn url(value: &str) -> Url {
    Url::parse(value).unwrap_or_else(|error| panic!("{value}: {error}"))
}

#[test]
fn https_gateway_allows_only_its_exact_origin() {
    let allowed = url("https://llm.corp.example/v1");
    let cases = [
        ("https://llm.corp.example/v1/responses", true),
        ("https://llm.corp.example/other/path", true),
        ("https://LLM.corp.example/v1", true),
        ("https://llm.corp.example:443/v1", true),
        ("wss://llm.corp.example/v1/responses", true),
        ("http://llm.corp.example/v1", false),
        ("ws://llm.corp.example/v1", false),
        ("https://llm.corp.example:8443/v1", false),
        ("https://llm.corp.example./v1", false),
        ("https://api.llm.corp.example/v1", false),
        ("https://corp.example/v1", false),
        ("https://llm.corp.example.evil.test/v1", false),
        ("https://llm.corp.example@evil.test/v1", false),
        ("https://user:secret@llm.corp.example/v1", false),
        ("https://api.openai.com/v1", false),
        ("https://chatgpt.com/backend-api/codex", false),
        ("https://203.0.113.10/v1", false),
        ("file:///etc/passwd", false),
    ];
    for (destination, expected) in cases {
        assert_eq!(
            destination_matches(&allowed, &url(destination)),
            expected,
            "{destination}"
        );
    }
}

#[test]
fn loopback_dev_gateway_keeps_port_and_scheme_exact() {
    let allowed = url("http://127.0.0.1:18080/v1");
    let cases = [
        ("http://127.0.0.1:18080/v1/responses", true),
        ("ws://127.0.0.1:18080/v1/responses", true),
        ("http://127.0.0.1:18081/v1", false),
        ("https://127.0.0.1:18080/v1", false),
        ("http://localhost:18080/v1", false),
    ];
    for (destination, expected) in cases {
        assert_eq!(
            destination_matches(&allowed, &url(destination)),
            expected,
            "{destination}"
        );
    }
}
