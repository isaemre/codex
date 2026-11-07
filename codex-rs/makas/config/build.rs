//! Embeds the only LLM address into the binary at compile time (K-07).
//!
//! Every input is read here and nowhere else, so a shipped binary has no runtime switch.

use std::env;
use std::net::IpAddr;

use url::Host;
use url::Url;

const BASE_URL_VAR: &str = "MAKAS_LLM_BASE_URL";
const UNLOCK_VAR: &str = "MAKAS_UNLOCK_FOR_UPSTREAM_TESTS";
const DEV_DEFAULT_BASE_URL: &str = "https://aigateway.mycompany.com";

fn main() {
    println!("cargo:rerun-if-env-changed={BASE_URL_VAR}");
    println!("cargo:rerun-if-env-changed={UNLOCK_VAR}");
    println!("cargo:rustc-check-cfg=cfg(makas_unlocked)");

    let is_release = env::var("PROFILE").is_ok_and(|profile| profile == "release");

    if env::var_os(UNLOCK_VAR).is_some() {
        if is_release {
            fail(&format!(
                "{UNLOCK_VAR} must never be set for a release build"
            ));
        }
        println!("cargo:rustc-cfg=makas_unlocked");
    }

    let base_url = match env::var(BASE_URL_VAR) {
        Ok(value) => value,
        Err(_) if is_release => fail(&format!("{BASE_URL_VAR} is required for a release build")),
        Err(_) => DEV_DEFAULT_BASE_URL.to_string(),
    };
    if let Err(message) = validate_base_url(&base_url, is_release) {
        fail(&format!("{BASE_URL_VAR}: {message}"));
    }
    println!(
        "cargo:rustc-env=MAKAS_COMPILED_LLM_BASE_URL={}",
        base_url.trim_end_matches('/')
    );
}

fn validate_base_url(base_url: &str, is_release: bool) -> Result<(), String> {
    let url = Url::parse(base_url).map_err(|error| format!("invalid URL: {error}"))?;
    match url.scheme() {
        "https" => {}
        "http" if !is_release && is_loopback(&url) => {}
        "http" => return Err("plain http is only allowed for loopback in dev builds".to_string()),
        scheme => return Err(format!("unsupported scheme `{scheme}`")),
    }
    if url.host().is_none() {
        return Err("a host is required".to_string());
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err("credentials are not allowed in the URL".to_string());
    }
    if url.query().is_some() || url.fragment().is_some() {
        return Err("query and fragment are not allowed in the URL".to_string());
    }
    Ok(())
}

fn is_loopback(url: &Url) -> bool {
    match url.host() {
        Some(Host::Domain(domain)) => domain == "localhost",
        Some(Host::Ipv4(ip)) => IpAddr::V4(ip).is_loopback(),
        Some(Host::Ipv6(ip)) => IpAddr::V6(ip).is_loopback(),
        None => false,
    }
}

fn fail(message: &str) -> ! {
    // `cargo::error` needs a newer MSRV contract than build scripts here rely on; a panic stops
    // the build on every Cargo version.
    panic!("makas: {message}");
}
