# configloader-rs... loads your config.

configloader-rs serializes the environment into your struct through proc macros and struct "annotations".

Why? It makes it easy to not repeat yourself. The struct already has naming conventions we want (*most of the time*)
out of the environment to get the correct details.

Imagine you have the following:

```rust
// You import this beautiful lib.
use configloader::{ConfigError, ConfigLoader};

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

	// Value should be Default::default() :)
    #[skip]
    password: String,
}

// and then!


#[test]
fn loads_double_nested_config_from_env() {
    let _guard = ENV_LOCK.lock().unwrap();

    unsafe {
        std::env::set_var("APP_NAME", "boring-oidc");
        std::env::set_var("PORT", "8080");
        std::env::set_var("HOST", "localhost");
        std::env::set_var("DB_PORT", "5432");
        std::env::set_var("USERNAME", "admin");
    }

    let config = AppConfig::load().unwrap();

	// And now you can just check the cool stuff
    assert_eq!(config.app_name, "boring-oidc");
    assert_eq!(config.port, 8080);
    assert_eq!(config.database.host, "localhost");
    assert_eq!(config.database.db_port, 5432);
    assert_eq!(config.database.credentials.username, "admin");
    assert_eq!(config.database.credentials.password, "");
	assert_eq!(config.def_val, "wow cool val");
    assert_eq!(config.u_def_val, 123);
}
```

