use configloader::{ConfigError, ConfigLoader};
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[derive(Debug, PartialEq, ConfigLoader)]
struct ErrorConfig {
    port: u16,
    enabled: bool,
    letter: char,
}

#[derive(Debug, PartialEq, ConfigLoader)]
struct NestedErrorConfig {
    name: String,

    #[nested]
    service: ErrorConfig,
}

#[test]
fn reports_invalid_var_name_and_parse_message() {
    let _guard = ENV_LOCK.lock().unwrap();

    unsafe {
        std::env::set_var("ERROR_CONFIG_PORT", "not-a-port");
        std::env::set_var("ERROR_CONFIG_ENABLED", "true");
        std::env::set_var("ERROR_CONFIG_LETTER", "z");
    }

    let err = ErrorConfig::load().unwrap_err();

    match err {
        ConfigError::InvalidVar { name, message } => {
            assert_eq!(name, "ERROR_CONFIG_PORT");
            assert!(!message.is_empty());
        }
        other => panic!("expected invalid var error, got {other:?}"),
    }
}

#[test]
fn reports_invalid_var_with_nested_prefix() {
    let _guard = ENV_LOCK.lock().unwrap();

    unsafe {
        std::env::set_var("NESTED_ERROR_CONFIG_NAME", "api");
        std::env::set_var("NESTED_ERROR_CONFIG_SERVICE_PORT", "70000");
        std::env::set_var("NESTED_ERROR_CONFIG_SERVICE_ENABLED", "true");
        std::env::set_var("NESTED_ERROR_CONFIG_SERVICE_LETTER", "z");
    }

    let err = NestedErrorConfig::load().unwrap_err();

    assert!(matches!(
        err,
        ConfigError::InvalidVar { name, message }
            if name == "NESTED_ERROR_CONFIG_SERVICE_PORT" && !message.is_empty()
    ));
}

#[test]
fn reports_all_missing_vars_before_parsing_present_invalid_values() {
    let _guard = ENV_LOCK.lock().unwrap();

    unsafe {
        std::env::set_var("ERROR_CONFIG_PORT", "not-a-port");
        std::env::remove_var("ERROR_CONFIG_ENABLED");
        std::env::remove_var("ERROR_CONFIG_LETTER");
    }

    let err = ErrorConfig::load().unwrap_err();

    assert_eq!(
        err,
        ConfigError::MissingVars(vec!["ERROR_CONFIG_ENABLED".to_string(), "ERROR_CONFIG_LETTER".to_string()])
    );
}
