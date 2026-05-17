# configloader-rs... loads your config.

configloader-rs serializes the environment into your struct through proc macros and struct "annotations".

Why? It makes it easy to not repeat yourself. The struct already has naming conventions we want (*most of the time*)
out of the environment to get the correct details.

Imagine you have the following:

```rust
use configloader::ConfigLoader;

#[derive(Debug, PartialEq, ConfigLoader)]
#[prefix("APP")]
struct AppConfig {
    name: String,
    port: u16,

    // ENV wins when it exists, otherwise this kicks in.
    #[default("false")]
    debug: bool,

    // Default::default(), don't go looking in ENV for this.
    #[skip]
    runtime_only_secret: String,

    #[nested]
    database: DatabaseConfig,

    #[nested]
    cache: CacheConfig,
}

#[derive(Debug, PartialEq, ConfigLoader)]
struct DatabaseConfig {
    url: String,
    max_connections: u32,

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
    host: String,

    #[default("6379")]
    port: u16,
}

fn main() -> Result<(), configloader::ConfigError> {
    let config = AppConfig::load()?;

    assert_eq!(config.name, "boring-oidc");
    assert_eq!(config.port, 8080);
    assert_eq!(config.debug, false);
    assert_eq!(config.database.url, "postgres://localhost/app");
    assert_eq!(config.database.max_connections, 10);
    assert_eq!(config.database.credentials.username, "admin");
    assert_eq!(config.cache.host, "localhost");
    assert_eq!(config.cache.port, 6379);
    assert_eq!(config.runtime_only_secret, "");

    Ok(())
}
```

And then your env looks like this:

```sh
APP_NAME=boring-oidc
APP_PORT=8080
APP_DATABASE_URL=postgres://localhost/app
APP_DATABASE_MAX_CONNECTIONS=10
APP_DATABASE_CREDENTIALS_USERNAME=admin
APP_DATABASE_CREDENTIALS_PASSWORD=not-for-your-readme-probably
APP_CACHE_HOST=localhost
```

Names are just the field names uppercased. Nested structs get the parent field name slapped in front:

```text
database.url -> DATABASE_URL
database.credentials.username -> DATABASE_CREDENTIALS_USERNAME
```

If you add `#[prefix("APP")]` at the top, everything starts with `APP_`.

Defaults are fallbacks, not hardcoded overrides. So this:

```rust
#[default("6379")]
port: u16,
```

means `APP_CACHE_PORT` wins if it exists, otherwise `6379` is used.

If you don't want a top level prefix, don't add one:

```rust
#[derive(ConfigLoader)]
struct AppConfig {
    port: u16,
}
```

That one reads `PORT`.
