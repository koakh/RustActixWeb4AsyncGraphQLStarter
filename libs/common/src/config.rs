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
  pub http_server_uri: Option<String>,
  pub http_server_enable_https: Option<bool>,
  pub http_server_api_key: Option<String>,
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

// pub struct Config {
//   pub http_server_uri: String,
//   pub http_server_enable_https: bool,
//   pub http_server_api_key: String,
//   pub certs_config_path: String,
//   pub certs_file_name_cert: String,
//   pub certs_file_name_key: String,
//   pub log_file_level: String,
//   pub log_file_path: String,
//   pub log_level: String,
// }

// /// Database pool config
// #[derive(Debug, Deserialize)]
// pub struct DbPool {
//   /// Database pool min
//   pub min: Option<i16>,
//   /// Database pool max
//   pub max: Option<i16>,
// }

// /// Database config
// #[derive(Debug, Deserialize)]
// pub struct Database {
//   /// Database hostname/IP
//   pub hostname: String,
//   /// Database username
//   pub username: String,
//   /// Database password
//   pub password: String,
//   /// Database name
//   pub name: String,
//   /// Database port
//   pub port: u16,
//   /// Full database url
//   pub url: String,
//   /// Database debug logging
//   pub debug: bool,
//   /// Database pool config
//   pub pool: DbPool,
// }

// /// Redis config
// #[derive(Debug, Deserialize)]
// pub struct Redis {
//   /// Redis url
//   pub url: String,
// }

// /// Auth client config
// #[derive(Debug, Deserialize)]
// pub struct AuthClient {
//   /// OAuth2 client id
//   pub id: Option<String>,
//   /// OAuth2 client secret
//   pub secret: Option<String>,
// }

// /// Auth test user config
// #[derive(Debug, Deserialize)]
// pub struct AuthTestUser {
//   /// Test user username
//   pub username: Option<String>,
//   /// Test user password
//   pub password: Option<String>,
// }

// /// Auth test config
// #[derive(Debug, Deserialize)]
// pub struct AuthTest {
//   /// Auth test user config
//   pub user: AuthTestUser,
//   /// Auth alt test user config
//   pub alt: AuthTestUser,
// }

// /// Auth config
// #[derive(Debug, Deserialize)]
// pub struct Auth {
//   /// OAuth2 url
//   pub url: String,
//   /// OAuth2 audience
//   pub audience: String,
//   /// Auth client config
//   pub client: AuthClient,
//   /// Auth test config
//   pub test: AuthTest,
// }

/// Application Config
#[derive(Debug, Deserialize)]
pub struct Config {
  /// the application's run mode (typically "development" or "production")
  pub run_mode: String,
  // TODO:
  /// the port to bind to
  pub port: u16,
  // server
  pub server: Server,
  // certificates
  pub certificate: Certificate,
  // logs
  pub log: Log,
}

impl Config {
  /// Create a new Config by merging in various sources
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
          // TODO: add env variables here like SERVER_, CERTIFICATE_, LOG_
          // split the Database variables
          .map(|key| key.as_str().replace("DATABASE_POOL_", "DATABASE.POOL.").into())
          .map(|key| key.as_str().replace("DATABASE_", "DATABASE.").into())
          // split the Redis variables
          .map(|key| key.as_str().replace("REDIS_", "REDIS.").into())
          // split the Auth variables
          .map(|key| key.as_str().replace("AUTH_TEST_USER_", "AUTH.TEST.USER.").into())
          .map(|key| key.as_str().replace("AUTH_TEST_ALT_", "AUTH.TEST.ALT.").into())
          .map(|key| key.as_str().replace("AUTH_CLIENT_", "AUTH.CLIENT.").into())
          .map(|key| key.as_str().replace("AUTH_", "AUTH.").into()),
      )
      // serialize and freeze
      .extract()?;

    // always use defaults if variables are not defined in config files or env variables
    if config.server.http_server_uri.is_none() {
      config.server.http_server_uri = Some(DEFAULT_HTTP_SERVER_URI.to_string());
    }

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
    if config.server.http_server_uri.is_none() {
      config.server.http_server_uri = Some(DEFAULT_HTTP_SERVER_URI.to_string())
    };
    if config.server.http_server_enable_https.is_none() {
      config.server.http_server_enable_https = Some(DEFAULT_HTTP_SERVER_ENABLE_HTTPS)
    };
    if config.server.http_server_api_key.is_none() {
      config.server.http_server_api_key = Some(DEFAULT_HTTP_SERVER_API_KEY.to_string())
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
