use configloader::{ConfigError, ConfigLoader};
use std::str::FromStr;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[derive(Debug, PartialEq)]
struct ApiKey(String);

#[derive(Debug, PartialEq)]
struct CommaSeparatedList(Vec<String>);

#[derive(Debug, PartialEq)]
struct ParseApiKeyError;

#[derive(Debug, PartialEq)]
struct ParseCommaSeparatedListError;

impl std::fmt::Display for ParseApiKeyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "api key must start with key_")
    }
}

impl std::fmt::Display for ParseCommaSeparatedListError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "list cannot be empty")
    }
}

impl FromStr for ApiKey {
    type Err = ParseApiKeyError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.starts_with("key_") {
            Ok(Self(value.to_string()))
        } else {
            Err(ParseApiKeyError)
        }
    }
}

impl FromStr for CommaSeparatedList {
    type Err = ParseCommaSeparatedListError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let values = value
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .collect::<Vec<_>>();

        if values.is_empty() {
            Err(ParseCommaSeparatedListError)
        } else {
            Ok(Self(values))
        }
    }
}

#[derive(Debug, PartialEq, ConfigLoader)]
struct CustomTypeConfig {
    api_key: ApiKey,
    allowed_origins: CommaSeparatedList,
}

#[test]
fn loads_custom_from_str_types() {
    let _guard = ENV_LOCK.lock().unwrap();

    unsafe {
        std::env::set_var("API_KEY", "key_123");
        std::env::set_var("ALLOWED_ORIGINS", "https://a.example, https://b.example");
    }

    let config = CustomTypeConfig::load().unwrap();

    assert_eq!(config.api_key, ApiKey("key_123".to_string()));
    assert_eq!(
        config.allowed_origins,
        CommaSeparatedList(vec![
            "https://a.example".to_string(),
            "https://b.example".to_string(),
        ])
    );
}

#[test]
fn reports_custom_from_str_error_messages() {
    let _guard = ENV_LOCK.lock().unwrap();

    unsafe {
        std::env::set_var("API_KEY", "invalid");
        std::env::set_var("ALLOWED_ORIGINS", "https://a.example");
    }

    let err = CustomTypeConfig::load().unwrap_err();

    assert_eq!(
        err,
        ConfigError::InvalidVar {
            name: "API_KEY".to_string(),
            message: "api key must start with key_".to_string(),
        }
    );
}
