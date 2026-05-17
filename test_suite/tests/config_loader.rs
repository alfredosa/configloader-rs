use configloader::{ConfigError, ConfigLoader};
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[derive(Debug, PartialEq, ConfigLoader)]
struct AppConfig {
    app_name: String,
    port: u16,

    #[default("wow cool val")]
    def_val: String,

    #[default("123")]
    u_def_val: u32,

    #[nested]
    database: DatabaseConfig,
}

#[derive(Debug, PartialEq, ConfigLoader)]
struct DatabaseConfig {
    host: String,
    db_port: u16,

    #[nested]
    credentials: CredentialsConfig,
}

#[derive(Debug, PartialEq, ConfigLoader)]
struct CredentialsConfig {
    username: String,

    #[skip]
    password: String,
}

#[derive(Debug, PartialEq, ConfigLoader)]
#[prefix("APP")]
struct PrefixedAppConfig {
    app_name: String,

    #[nested]
    database: DatabaseConfig,
}

#[test]
fn loads_double_nested_config_from_env() {
    let _guard = ENV_LOCK.lock().unwrap();

    unsafe {
        std::env::set_var("APP_NAME", "boring-oidc");
        std::env::set_var("PORT", "8080");
        std::env::set_var("DATABASE_HOST", "localhost");
        std::env::set_var("DATABASE_DB_PORT", "5432");
        std::env::set_var("DATABASE_CREDENTIALS_USERNAME", "admin");
    }

    let config = AppConfig::load().unwrap();

    assert_eq!(config.app_name, "boring-oidc");
    assert_eq!(config.port, 8080);
    assert_eq!(config.database.host, "localhost");
    assert_eq!(config.database.db_port, 5432);
    assert_eq!(config.database.credentials.username, "admin");
    assert_eq!(config.database.credentials.password, "");
}

#[test]
fn reports_all_missing_env_vars_across_nested_config() {
    let _guard = ENV_LOCK.lock().unwrap();

    unsafe {
        std::env::remove_var("APP_NAME");
        std::env::remove_var("PORT");
        std::env::remove_var("DATABASE_HOST");
        std::env::remove_var("DATABASE_DB_PORT");
        std::env::remove_var("DATABASE_CREDENTIALS_USERNAME");
    }

    let err = AppConfig::load().unwrap_err();

    assert_eq!(
        err,
        ConfigError::MissingVars(vec![
            "APP_NAME".to_string(),
            "PORT".to_string(),
            "DATABASE_HOST".to_string(),
            "DATABASE_DB_PORT".to_string(),
            "DATABASE_CREDENTIALS_USERNAME".to_string()
        ])
    );
}

// This tests actually shows that the api is broken because it
// might be good to get the list of invalid too?
// need to be careful here because a password
// is actually not something we want to display.
#[test]
fn reports_invalid_env_var_after_required_vars_are_present() {
    let _guard = ENV_LOCK.lock().unwrap();

    unsafe {
        std::env::set_var("APP_NAME", "boring-oidc");
        std::env::set_var("PORT", "not-a-port");
        std::env::set_var("DATABASE_HOST", "localhost");
        std::env::set_var("DATABASE_DB_PORT", "5432");
        std::env::set_var("DATABASE_CREDENTIALS_USERNAME", "admin");
    }

    let err = AppConfig::load().unwrap_err();

    assert!(matches!(err, ConfigError::InvalidVar { name, .. } if name == "PORT"));
}

#[test]
fn defaults_just_work() {
    let _guard = ENV_LOCK.lock().unwrap();

    unsafe {
        std::env::set_var("APP_NAME", "boring-oidc");
        std::env::set_var("PORT", "8080");
        std::env::set_var("DATABASE_HOST", "localhost");
        std::env::set_var("DATABASE_DB_PORT", "5432");
        std::env::set_var("DATABASE_CREDENTIALS_USERNAME", "admin");
    }

    let config = AppConfig::load().unwrap();

    assert_eq!(config.def_val, "wow cool val");
    assert_eq!(config.u_def_val, 123);
}

#[test]
fn loads_top_level_config_with_declared_prefix() {
    let _guard = ENV_LOCK.lock().unwrap();

    unsafe {
        std::env::set_var("APP_APP_NAME", "boring-oidc");
        std::env::set_var("APP_DATABASE_HOST", "localhost");
        std::env::set_var("APP_DATABASE_DB_PORT", "5432");
        std::env::set_var("APP_DATABASE_CREDENTIALS_USERNAME", "admin");
    }

    let config = PrefixedAppConfig::load().unwrap();

    assert_eq!(config.app_name, "boring-oidc");
    assert_eq!(config.database.host, "localhost");
    assert_eq!(config.database.db_port, 5432);
    assert_eq!(config.database.credentials.username, "admin");
}
