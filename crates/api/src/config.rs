use std::env::{self, VarError};
use std::fmt::Display;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;

/// Which environment the app runs in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppEnv {
    Development,
    Production,
}

impl FromStr for AppEnv {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "development" | "dev" => Ok(Self::Development),
            "production" | "prod" => Ok(Self::Production),
            other => Err(format!(
                "unknown value '{other}' (expected development or production)"
            )),
        }
    }
}

/// All runtime settings, loaded once at startup.
#[derive(Clone)]
pub struct Config {
    pub app_env: AppEnv,
    pub host: IpAddr,
    pub port: u16,
    #[expect(dead_code, reason = "used from Step 7 (database pool)")]
    pub database_url: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("missing required environment variable {0}")]
    Missing(&'static str),
    #[error("invalid value for {name}: {reason}")]
    Invalid { name: &'static str, reason: String },
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            app_env: optional("APP_ENV", AppEnv::Development)?,
            host: optional("APP_HOST", IpAddr::V4(Ipv4Addr::UNSPECIFIED))?,
            port: optional("APP_PORT", 3000)?,
            database_url: required("DATABASE_URL")?,
        })
    }

    pub fn addr(&self) -> SocketAddr {
        SocketAddr::new(self.host, self.port)
    }
}

/// Reads a variable that must be present and non-empty.
fn required(name: &'static str) -> Result<String, ConfigError> {
    match env::var(name) {
        Ok(v) if !v.trim().is_empty() => Ok(v),
        Ok(_) | Err(VarError::NotPresent) => Err(ConfigError::Missing(name)),
        Err(VarError::NotUnicode(_)) => Err(ConfigError::Invalid {
            name,
            reason: "not valid UTF-8".into(),
        }),
    }
}

/// Reads and parses a variable, falling back to `default` when absent.
fn optional<T>(name: &'static str, default: T) -> Result<T, ConfigError>
where
    T: FromStr,
    T::Err: Display,
{
    match env::var(name) {
        Ok(v) => v.trim().parse().map_err(|e: T::Err| ConfigError::Invalid {
            name,
            reason: e.to_string(),
        }),
        Err(VarError::NotPresent) => Ok(default),
        Err(VarError::NotUnicode(_)) => Err(ConfigError::Invalid {
            name,
            reason: "not valid UTF-8".into(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_app_env() {
        assert_eq!("production".parse::<AppEnv>(), Ok(AppEnv::Production));
        assert_eq!("DEV".parse::<AppEnv>(), Ok(AppEnv::Development));
        assert!("staging".parse::<AppEnv>().is_err());
    }
}
