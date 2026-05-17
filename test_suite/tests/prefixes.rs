use configloader::{ConfigError, ConfigLoader};
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[derive(Debug, PartialEq, ConfigLoader)]
struct RootConfig {
    service_name: String,

    #[nested]
    database: DatabaseConfig,

    #[nested]
    cache: CacheConfig,
}

#[derive(Debug, PartialEq, ConfigLoader)]
struct DatabaseConfig {
    host: String,
    port: u16,

    #[nested]
    credentials: CredentialsConfig,
}

#[derive(Debug, PartialEq, ConfigLoader)]
struct CredentialsConfig {
    username: String,
    password: String,
}

#[derive(Debug, PartialEq, ConfigLoader)]
struct CacheConfig {
    endpoint: String,
    ttl_seconds: u64,
}

#[derive(Debug, PartialEq, ConfigLoader)]
#[prefix("SERVICE")]
struct DeclaredPrefixConfig {
    name: String,

    #[nested]
    database: DatabaseConfig,
}

#[test]
fn nests_prefixes_by_field_name_for_multiple_branches() {
    let _guard = ENV_LOCK.lock().unwrap();

    unsafe {
        std::env::set_var("SERVICE_NAME", "accounts");
        std::env::set_var("DATABASE_HOST", "db.internal");
        std::env::set_var("DATABASE_PORT", "5432");
        std::env::set_var("DATABASE_CREDENTIALS_USERNAME", "admin");
        std::env::set_var("DATABASE_CREDENTIALS_PASSWORD", "secret");
        std::env::set_var("CACHE_ENDPOINT", "redis://cache.internal:6379");
        std::env::set_var("CACHE_TTL_SECONDS", "30");
    }

    let config = RootConfig::load().unwrap();

    assert_eq!(config.service_name, "accounts");
    assert_eq!(config.database.host, "db.internal");
    assert_eq!(config.database.port, 5432);
    assert_eq!(config.database.credentials.username, "admin");
    assert_eq!(config.database.credentials.password, "secret");
    assert_eq!(config.cache.endpoint, "redis://cache.internal:6379");
    assert_eq!(config.cache.ttl_seconds, 30);
}

#[test]
fn uses_declared_prefix_for_top_level_load() {
    let _guard = ENV_LOCK.lock().unwrap();

    unsafe {
        std::env::set_var("SERVICE_NAME", "accounts");
        std::env::set_var("SERVICE_DATABASE_HOST", "db.internal");
        std::env::set_var("SERVICE_DATABASE_PORT", "5432");
        std::env::set_var("SERVICE_DATABASE_CREDENTIALS_USERNAME", "admin");
        std::env::set_var("SERVICE_DATABASE_CREDENTIALS_PASSWORD", "secret");
    }

    let config = DeclaredPrefixConfig::load().unwrap();

    assert_eq!(config.name, "accounts");
    assert_eq!(config.database.host, "db.internal");
    assert_eq!(config.database.port, 5432);
    assert_eq!(config.database.credentials.username, "admin");
    assert_eq!(config.database.credentials.password, "secret");
}

#[test]
fn explicit_load_with_prefix_overrides_declared_prefix() {
    let _guard = ENV_LOCK.lock().unwrap();

    unsafe {
        std::env::set_var("OVERRIDE_NAME", "accounts");
        std::env::set_var("OVERRIDE_DATABASE_HOST", "db.internal");
        std::env::set_var("OVERRIDE_DATABASE_PORT", "5432");
        std::env::set_var("OVERRIDE_DATABASE_CREDENTIALS_USERNAME", "admin");
        std::env::set_var("OVERRIDE_DATABASE_CREDENTIALS_PASSWORD", "secret");
    }

    let config = DeclaredPrefixConfig::load_with_prefix("OVERRIDE").unwrap();

    assert_eq!(config.name, "accounts");
    assert_eq!(config.database.host, "db.internal");
    assert_eq!(config.database.port, 5432);
    assert_eq!(config.database.credentials.username, "admin");
    assert_eq!(config.database.credentials.password, "secret");
}

#[test]
fn reports_missing_nested_vars_with_full_prefix_chain() {
    let _guard = ENV_LOCK.lock().unwrap();

    unsafe {
        std::env::remove_var("MISSING_SERVICE_NAME");
        std::env::remove_var("MISSING_DATABASE_HOST");
        std::env::remove_var("MISSING_DATABASE_PORT");
        std::env::remove_var("MISSING_DATABASE_CREDENTIALS_USERNAME");
        std::env::remove_var("MISSING_DATABASE_CREDENTIALS_PASSWORD");
        std::env::remove_var("MISSING_CACHE_ENDPOINT");
        std::env::remove_var("MISSING_CACHE_TTL_SECONDS");
    }

    let err = RootConfig::load_with_prefix("MISSING").unwrap_err();

    assert_eq!(
        err,
        ConfigError::MissingVars(vec![
            "MISSING_SERVICE_NAME".to_string(),
            "MISSING_DATABASE_HOST".to_string(),
            "MISSING_DATABASE_PORT".to_string(),
            "MISSING_DATABASE_CREDENTIALS_USERNAME".to_string(),
            "MISSING_DATABASE_CREDENTIALS_PASSWORD".to_string(),
            "MISSING_CACHE_ENDPOINT".to_string(),
            "MISSING_CACHE_TTL_SECONDS".to_string(),
        ])
    );
}
