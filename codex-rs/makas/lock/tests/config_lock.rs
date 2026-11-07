#![allow(clippy::expect_used)]
//! Every config source must resolve to the single compiled-in provider (K-07, K-27).

use codex_config::LoaderOverrides;
use codex_core::config::Config;
use codex_core::config::ConfigBuilder;
use codex_core::config::ConfigOverrides;
use codex_makas_config::LOCKED;
use codex_makas_config::llm_base_url;
use codex_makas_lock::MAKAS_PROVIDER_ID;
use pretty_assertions::assert_eq;
use tempfile::TempDir;
use toml::Value as TomlValue;

const ATTACKER_PROVIDER: &str = r#"
[model_providers.corp]
name = "corp"
base_url = "https://attacker.example/v1"
"#;

#[derive(Default)]
struct Sources {
    config_toml: String,
    requirements_toml: Option<String>,
    cli_overrides: Vec<(String, TomlValue)>,
    harness_overrides: ConfigOverrides,
}

async fn load(sources: Sources) -> Config {
    let home = TempDir::new().expect("create temporary home");
    std::fs::write(home.path().join("config.toml"), sources.config_toml)
        .expect("write config.toml");

    let mut loader_overrides = LoaderOverrides::without_managed_config_for_tests();
    if let Some(requirements) = sources.requirements_toml {
        let path = home.path().join("requirements.toml");
        std::fs::write(&path, requirements).expect("write requirements.toml");
        loader_overrides.system_requirements_path = Some(path);
    }

    ConfigBuilder::default()
        .codex_home(home.path().to_path_buf())
        .fallback_cwd(Some(home.path().to_path_buf()))
        .loader_overrides(loader_overrides)
        .cli_overrides(sources.cli_overrides)
        .harness_overrides(sources.harness_overrides)
        .build()
        .await
        .expect("load config")
}

fn assert_locked(config: &Config) {
    assert_eq!(config.model_provider_id, MAKAS_PROVIDER_ID);
    assert_eq!(
        config.model_provider.base_url.as_deref(),
        Some(llm_base_url())
    );
    assert_eq!(
        config.model_providers.keys().collect::<Vec<_>>(),
        vec![MAKAS_PROVIDER_ID]
    );
    assert!(!config.model_provider.is_openai());
    assert!(config.model_provider.requires_openai_auth);
    assert!(!config.model_provider.supports_websockets);
}

#[tokio::test]
async fn empty_config_uses_the_locked_provider() {
    if LOCKED {
        assert_locked(&load(Sources::default()).await);
    }
}

#[tokio::test]
async fn config_toml_cannot_select_another_provider() {
    if !LOCKED {
        return;
    }
    let config = load(Sources {
        config_toml: format!(
            r#"
model_provider = "corp"
openai_base_url = "https://attacker.example/v1"
{ATTACKER_PROVIDER}
"#
        ),
        ..Default::default()
    })
    .await;
    assert_locked(&config);
}

#[tokio::test]
async fn config_toml_cannot_redirect_the_openai_provider() {
    if !LOCKED {
        return;
    }
    let config = load(Sources {
        config_toml: r#"
model_provider = "openai"
openai_base_url = "https://attacker.example/v1"
"#
        .to_string(),
        ..Default::default()
    })
    .await;
    assert_locked(&config);
}

#[tokio::test]
async fn legacy_profile_cannot_smuggle_another_provider() {
    if !LOCKED {
        return;
    }
    // Upstream rejects the legacy `profile` key outright, so the attacker profile never loads.
    let home = TempDir::new().expect("create temporary home");
    std::fs::write(
        home.path().join("config.toml"),
        format!(
            r#"
profile = "attacker"

[profiles.attacker]
model_provider = "corp"
{ATTACKER_PROVIDER}
"#
        ),
    )
    .expect("write config.toml");
    let result = ConfigBuilder::default()
        .codex_home(home.path().to_path_buf())
        .fallback_cwd(Some(home.path().to_path_buf()))
        .loader_overrides(LoaderOverrides::without_managed_config_for_tests())
        .build()
        .await;
    assert!(result.is_err(), "legacy profile key must not load");
}

#[tokio::test]
async fn cli_overrides_cannot_select_another_provider() {
    if !LOCKED {
        return;
    }
    let attacker: toml::Table = toml::from_str(ATTACKER_PROVIDER).expect("parse provider");
    let config = load(Sources {
        cli_overrides: vec![
            ("model_provider".to_string(), "corp".into()),
            (
                "model_providers".to_string(),
                attacker["model_providers"].clone(),
            ),
        ],
        ..Default::default()
    })
    .await;
    assert_locked(&config);
}

#[tokio::test]
async fn local_provider_flag_cannot_select_ollama() {
    if !LOCKED {
        return;
    }
    let config = load(Sources {
        harness_overrides: ConfigOverrides {
            model_provider: Some("ollama".to_string()),
            ..Default::default()
        },
        ..Default::default()
    })
    .await;
    assert_locked(&config);
}

#[tokio::test]
async fn requirements_cannot_select_another_provider() {
    if !LOCKED {
        return;
    }
    let config = load(Sources {
        requirements_toml: Some(format!(
            r#"
model_provider = "corp"
{ATTACKER_PROVIDER}
"#
        )),
        ..Default::default()
    })
    .await;
    assert_locked(&config);
}
