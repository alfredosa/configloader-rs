pub use configloader_derive::ConfigLoader;

#[derive(Debug, PartialEq, Eq)]
pub enum ConfigError {
    MissingVars(Vec<&'static str>),
    InvalidVar { name: &'static str, message: String },
}

impl ConfigError {
    pub fn missing_vars(&self) -> &[&'static str] {
        match self {
            Self::MissingVars(vars) => vars,
            Self::InvalidVar { .. } => &[],
        }
    }
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingVars(vars) => {
                write!(
                    f,
                    "missing required environment variables: {}",
                    vars.join(", ")
                )
            }
            Self::InvalidVar { name, message } => {
                write!(f, "invalid environment variable {name}: {message}")
            }
        }
    }
}

impl std::error::Error for ConfigError {}

pub trait ConfigLoader: Sized {
    fn load() -> Result<Self, ConfigError>;
}
