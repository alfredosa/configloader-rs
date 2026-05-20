use configloader::{ConfigError, ConfigLoader};
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[derive(Debug, PartialEq, ConfigLoader)]
struct EnvOverrideConfig {
    #[env("SERVICE_NAME")]
    name: String,

    #[env("HTTP_PORT")]
    port: u16,
}

#[derive(Debug, PartialEq, ConfigLoader)]
#[prefix("APP")]
struct PrefixedEnvOverrideConfig {
    #[env("SERVICE_NAME")]
    name: String,

    #[env("HTTP_PORT")]
    port: u16,
}

#[derive(Debug, PartialEq, ConfigLoader)]
struct NestedEnvOverrideConfig {
    #[nested]
    #[env("DB")]
    database: DatabaseConfig,
}

#[derive(Debug, PartialEq, ConfigLoader)]
struct DatabaseConfig {
    #[env("HOSTNAME")]
    host: String,

    #[env("PORT")]
    port: u16,
}

#[derive(Debug, PartialEq, ConfigLoader)]
struct DefaultedEnvOverrideConfig {
    #[env("HTTP_PORT")]
    #[default("8080")]
    port: u16,
}

#[test]
fn loads_fields_from_explicit_env_names() {
    let _guard = ENV_LOCK.lock().unwrap();

    unsafe {
        std::env::set_var("ENV_OVERRIDE_CONFIG_SERVICE_NAME", "accounts");
        std::env::set_var("ENV_OVERRIDE_CONFIG_HTTP_PORT", "8080");
    }

    let config = EnvOverrideConfig::load().unwrap();

    assert_eq!(config.name, "accounts");
    assert_eq!(config.port, 8080);
}

#[test]
fn explicit_env_names_are_still_prefixed() {
    let _guard = ENV_LOCK.lock().unwrap();

    unsafe {
        std::env::set_var("APP_SERVICE_NAME", "accounts");
        std::env::set_var("APP_HTTP_PORT", "8080");
    }

    let config = PrefixedEnvOverrideConfig::load().unwrap();

    assert_eq!(config.name, "accounts");
    assert_eq!(config.port, 8080);
}

#[test]
fn explicit_env_names_work_for_nested_prefixes_and_nested_fields() {
    let _guard = ENV_LOCK.lock().unwrap();

    unsafe {
        std::env::set_var("NESTED_ENV_OVERRIDE_CONFIG_DB_HOSTNAME", "localhost");
        std::env::set_var("NESTED_ENV_OVERRIDE_CONFIG_DB_PORT", "5432");
    }

    let config = NestedEnvOverrideConfig::load().unwrap();

    assert_eq!(config.database.host, "localhost");
    assert_eq!(config.database.port, 5432);
}

#[test]
fn explicit_env_names_work_with_defaults() {
    let _guard = ENV_LOCK.lock().unwrap();

    unsafe {
        std::env::remove_var("DEFAULTED_ENV_OVERRIDE_CONFIG_HTTP_PORT");
    }

    let config = DefaultedEnvOverrideConfig::load().unwrap();

    assert_eq!(config.port, 8080);

    unsafe {
        std::env::set_var("DEFAULTED_ENV_OVERRIDE_CONFIG_HTTP_PORT", "9000");
    }

    let config = DefaultedEnvOverrideConfig::load().unwrap();

    assert_eq!(config.port, 9000);
}

#[test]
fn reports_missing_explicit_env_names() {
    let _guard = ENV_LOCK.lock().unwrap();

    unsafe {
        std::env::remove_var("ENV_OVERRIDE_CONFIG_SERVICE_NAME");
        std::env::remove_var("ENV_OVERRIDE_CONFIG_HTTP_PORT");
    }

    let err = EnvOverrideConfig::load().unwrap_err();

    assert_eq!(
        err,
        ConfigError::MissingVars(vec!["ENV_OVERRIDE_CONFIG_SERVICE_NAME".to_string(), "ENV_OVERRIDE_CONFIG_HTTP_PORT".to_string()])
    );
}

#[test]
fn reports_invalid_explicit_env_name() {
    let _guard = ENV_LOCK.lock().unwrap();

    unsafe {
        std::env::set_var("ENV_OVERRIDE_CONFIG_SERVICE_NAME", "accounts");
        std::env::set_var("ENV_OVERRIDE_CONFIG_HTTP_PORT", "not-a-port");
    }

    let err = EnvOverrideConfig::load().unwrap_err();

    assert!(matches!(
        err,
        ConfigError::InvalidVar { name, message }
            if name == "ENV_OVERRIDE_CONFIG_HTTP_PORT" && !message.is_empty()
    ));
}
