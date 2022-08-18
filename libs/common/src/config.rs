use anyhow::Result;
use figment::{
  providers::{Env, Format, Toml},
  Figment,
};
use once_cell::sync::Lazy;
use serde_derive::Deserialize;
use std::env;

/// server
pub const DEFAULT_HTTP_SERVER_URI: &str = "0.0.0.0:443";
pub const DEFAULT_HTTP_SERVER_API_KEY: &str = "ENKpqTvRw3ybXvMyWwf0DBIXeAS2JRTHtTsEq1jqoAyVjpI7l9iaaPlbnvAf0ReQ";
pub const DEFAULT_HTTP_SERVER_ENABLE_HTTPS: bool = true;
/// debug
pub const DEFAULT_LOG_LEVEL: &str = "ERROR";
pub const DEFAULT_LOG_FILE_LEVEL: &str = "ERROR";
pub const DEFAULT_LOG_FILE_PATH: &str = "./server-starter.log";
/// certificates
pub const DEFAULT_CONFIG_PATH_SSL: &str = "./config/ssl";
pub const DEFAULT_CERT_FILE_NAME_KEY: &str = "key.pem";
pub const DEFAULT_CERT_FILE_NAME_CERT: &str = "cert.pem";

/// The default `Config` instance
static CONFIG: Lazy<Config> = Lazy::new(|| Config::new().expect("Unable to retrieve config"));

/// Server config
#[derive(Debug, Deserialize)]
pub struct Server {
  pub uri: Option<String>,
  pub enable_https: Option<bool>,
  pub api_key: Option<String>,
}

/// Certificates config
#[derive(Debug, Deserialize)]
pub struct Certificate {
  pub config_path: Option<String>,
  pub file_name_cert: Option<String>,
  pub file_name_key: Option<String>,
}

/// Log config
#[derive(Debug, Deserialize)]
pub struct Log {
  pub log_level: Option<String>,
  pub log_file_level: Option<String>,
  pub log_file_path: Option<String>,
}

/// Application Config
#[derive(Debug, Deserialize)]
pub struct Config {
  /// the application's run mode (typically "development" or "production")
  pub run_mode: String,
  // server
  pub server: Server,
  // certificates
  pub certificate: Certificate,
  // logs
  pub log: Log,
}

impl Config {
  /// create a new Config by merging in various sources
  pub fn new() -> Result<Self> {
    let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".to_string());

    let mut config: Config = Figment::new()
      // load defaults
      .merge(Toml::file("config/default.toml"))
      // load local overrides
      .merge(Toml::file("config/local.toml"))
      // load run mode overrides
      .merge(Toml::file(format!("config/{}.toml", run_mode)))
      // load environment variables
      .merge(
        // support the nested structure of the config manually
        Env::raw()
          .map(|key| key.as_str().replace("SERVER_", "SERVER.").into())
          .map(|key| key.as_str().replace("CERTIFICATE_", "CERTIFICATE.").into())
          .map(|key| key.as_str().replace("LOG_", "LOG.").into())
      )
      // serialize and freeze
      .extract()?;

    // always use defaults if variables are not defined in config files or env variables
    // init log variables
    if config.log.log_file_path.is_none() {
      config.log.log_file_path = Some(DEFAULT_LOG_FILE_PATH.to_string())
    };
    if config.log.log_level.is_none() {
      config.log.log_level = Some(DEFAULT_LOG_LEVEL.to_string())
    };
    if config.log.log_file_level.is_none() {
      config.log.log_file_level = Some(DEFAULT_LOG_FILE_LEVEL.to_string())
    };
    // env vars: server
    if config.server.uri.is_none() {
      config.server.uri = Some(DEFAULT_HTTP_SERVER_URI.to_string())
    };
    if config.server.enable_https.is_none() {
      config.server.enable_https = Some(DEFAULT_HTTP_SERVER_ENABLE_HTTPS)
    };
    if config.server.api_key.is_none() {
      config.server.api_key = Some(DEFAULT_HTTP_SERVER_API_KEY.to_string())
    };
    // env vars: certificate
    if config.certificate.config_path.is_none() {
      config.certificate.config_path = Some(DEFAULT_CONFIG_PATH_SSL.to_string())
    };
    if config.certificate.file_name_key.is_none() {
      config.certificate.file_name_key = Some(DEFAULT_CERT_FILE_NAME_KEY.to_string())
    };
    if config.certificate.file_name_cert.is_none() {
      config.certificate.file_name_cert = Some(DEFAULT_CERT_FILE_NAME_CERT.to_string())
    };

    Ok(config)
  }

  /// Return true if the `run_mode` is "development"
  pub fn is_dev(&self) -> bool {
    self.run_mode == "development"
  }
}

/// Get the default static `Config`
pub fn get_config() -> &'static Config {
  &CONFIG
}
