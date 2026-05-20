use configloader::{ConfigError, ConfigLoader};
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[derive(Debug, PartialEq, ConfigLoader)]
struct DefaultsConfig {
    required: String,

    #[default("fallback")]
    default_string: String,

    #[default("8080")]
    default_port: u16,

    #[default("true")]
    default_bool: bool,

    #[skip]
    skipped_string: String,

    #[skip]
    skipped_number: u32,
}

#[test]
fn loads_defaults_and_skipped_values_when_env_is_absent() {
    let _guard = ENV_LOCK.lock().unwrap();

    unsafe {
        std::env::set_var("DEFAULTS_CONFIG_REQUIRED", "present");
        std::env::remove_var("DEFAULTS_CONFIG_DEFAULT_STRING");
        std::env::remove_var("DEFAULTS_CONFIG_DEFAULT_PORT");
        std::env::remove_var("DEFAULTS_CONFIG_DEFAULT_BOOL");
        std::env::set_var("DEFAULTS_CONFIG_SKIPPED_STRING", "ignored");
        std::env::set_var("DEFAULTS_CONFIG_SKIPPED_NUMBER", "123");
    }

    let config = DefaultsConfig::load().unwrap();

    assert_eq!(config.required, "present");
    assert_eq!(config.default_string, "fallback");
    assert_eq!(config.default_port, 8080);
    assert!(config.default_bool);
    assert_eq!(config.skipped_string, "");
    assert_eq!(config.skipped_number, 0);
}

#[test]
fn env_values_override_defaults_when_present() {
    let _guard = ENV_LOCK.lock().unwrap();

    unsafe {
        std::env::set_var("DEFAULTS_CONFIG_REQUIRED", "present");
        std::env::set_var("DEFAULTS_CONFIG_DEFAULT_STRING", "from-env");
        std::env::set_var("DEFAULTS_CONFIG_DEFAULT_PORT", "9000");
        std::env::set_var("DEFAULTS_CONFIG_DEFAULT_BOOL", "false");
    }

    let config = DefaultsConfig::load().unwrap();

    assert_eq!(config.default_string, "from-env");
    assert_eq!(config.default_port, 9000);
    assert!(!config.default_bool);
}

#[test]
fn invalid_env_values_are_not_masked_by_defaults() {
    let _guard = ENV_LOCK.lock().unwrap();

    unsafe {
        std::env::set_var("DEFAULTS_CONFIG_REQUIRED", "present");
        std::env::remove_var("DEFAULTS_CONFIG_DEFAULT_STRING");
        std::env::set_var("DEFAULTS_CONFIG_DEFAULT_PORT", "not-a-port");
        std::env::remove_var("DEFAULTS_CONFIG_DEFAULT_BOOL");
    }

    let err = DefaultsConfig::load().unwrap_err();

    assert!(matches!(
        err,
        ConfigError::InvalidVar { name, message }
            if name == "DEFAULTS_CONFIG_DEFAULT_PORT" && !message.is_empty()
    ));
}
