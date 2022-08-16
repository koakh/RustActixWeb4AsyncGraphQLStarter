use actix_web::{dev::ServiceRequest, get, guard, web, web::Data, App, Error as ActixError, HttpRequest, HttpResponse, HttpServer, Result};
use actix_web_httpauth::{
  extractors::{
    bearer::{BearerAuth, Config},
    AuthenticationError,
  },
  middleware::HttpAuthentication,
};
use async_graphql::{
  http::{playground_source, GraphQLPlaygroundConfig},
  Schema,
};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
use books::{BooksSchema, MutationRoot, QueryRoot, Storage, SubscriptionRoot};
use common::logger::init_log4rs;
use log::{error, info};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use serde::{Deserialize, Serialize};
use server::constants::{
  APP_NAME, DEFAULT_CERT_FILE_NAME_CERT, DEFAULT_CERT_FILE_NAME_KEY, DEFAULT_CONFIG_PATH_SSL, DEFAULT_HTTP_SERVER_API_KEY, DEFAULT_HTTP_SERVER_ENABLE_HTTPS, DEFAULT_HTTP_SERVER_URI,
  DEFAULT_LOGFILE_LEVEL, DEFAULT_LOG_FILE_PATH, DEFAULT_LOG_LEVEL, GRAPHQL_PATH, HTTP_SERVER_KEEP_ALIVE, PLAYGROUND_PATH,
};
use std::{env, time::Duration};

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageResponse {
  pub message: String,
}

async fn index(schema: web::Data<BooksSchema>, req: GraphQLRequest) -> GraphQLResponse {
  schema.execute(req.into_inner()).await.into()
}

async fn index_playground() -> Result<HttpResponse> {
  Ok(
    HttpResponse::Ok()
      .content_type("text/html; charset=utf-8")
      // leave subscription_endpoint("") will be wrapped in /graphql like all other protected endpoints
      .body(playground_source(GraphQLPlaygroundConfig::new(GRAPHQL_PATH).subscription_endpoint(GRAPHQL_PATH))),
  )
}

async fn index_ws(schema: web::Data<BooksSchema>, req: HttpRequest, payload: web::Payload) -> Result<HttpResponse> {
  GraphQLSubscription::new(Schema::clone(&*schema)).start(&req, payload)
}

#[get("/ping")]
async fn health_check(_: HttpRequest) -> Result<web::Json<MessageResponse>> {
  Ok(web::Json(MessageResponse { message: "pong".to_string() }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  // init defaults
  let default_log_file_path = std::env::var("LOG_FILE_PATH").unwrap_or_else(|_| DEFAULT_LOG_FILE_PATH.to_string());
  let default_log_level = std::env::var("LOG_LEVEL").unwrap_or_else(|_| DEFAULT_LOG_LEVEL.to_string());
  let default_logfile_level = std::env::var("LOGFILE_LEVEL").unwrap_or_else(|_| DEFAULT_LOGFILE_LEVEL.to_string());
  // init log4rs
  init_log4rs(&default_log_file_path, &default_log_level, &default_logfile_level).expect("can't initialize logger");
  // env vars
  let http_server_uri = env::var("HTTP_SERVER_URI").unwrap_or_else(|_| DEFAULT_HTTP_SERVER_URI.to_string());
  let config_path_ssl = env::var("CONFIG_PATH_SSL").unwrap_or_else(|_| DEFAULT_CONFIG_PATH_SSL.to_string());
  let http_server_enable_https = env::var("HTTP_SERVER_ENABLE_HTTPS").unwrap_or_else(|_| DEFAULT_HTTP_SERVER_ENABLE_HTTPS.to_string());
  let cert_file_name_key = env::var("CERT_FILE_NAME_KEY").unwrap_or_else(|_| DEFAULT_CERT_FILE_NAME_KEY.to_string());
  let cert_file_name_cert = env::var("CERT_FILE_NAME_CERT").unwrap_or_else(|_| DEFAULT_CERT_FILE_NAME_CERT.to_string());
  let http_server_api_key = env::var("HTTP_SERVER_API_KEY").unwrap_or_else(|_| DEFAULT_HTTP_SERVER_API_KEY.to_string());

  // authentication validator
  // required to implement ResponseError in src/app/errors.rs else we have a error
  // Err(AuthenticationError::from(config).into())
  async fn validator(req: ServiceRequest, credentials: BearerAuth) -> Result<ServiceRequest, ActixError> {
    let http_server_api_key = env::var("HTTP_SERVER_API_KEY").unwrap_or_else(|_| DEFAULT_HTTP_SERVER_API_KEY.to_string());
    if credentials.token() == http_server_api_key {
      Ok(req)
    } else {
      let config = req.app_data::<Config>().cloned().unwrap_or_default().scope("urn:example:channel=HBO&urn:example:rating=G,PG-13");
      error!("{}", "invalid authorization api key".to_string());
      // uncomment to debug api key token
      // error!("  currentKey: '{}', credentials token: '{}'", http_server_api_key, credentials.token());
      Err(AuthenticationError::from(config).into())
      // with this output unauthorized in response, we keep with silence message and error code 401
      // use actix_web::{error::ErrorUnauthorized};
      // Err(ErrorUnauthorized("unauthorized"))
    }
  }

  // init graphql schema
  let schema = Schema::build(QueryRoot, MutationRoot, SubscriptionRoot).data(Storage::default()).finish();

  let http_server = HttpServer::new(move || {
    App::new()
      .app_data(Data::new(schema.clone()))
      .service(health_check)
      .service(web::resource(PLAYGROUND_PATH).guard(guard::Get()).to(index_playground))
      .service(
        web::scope(GRAPHQL_PATH)
          // bellow routes are protected with validator
          .wrap(HttpAuthentication::bearer(validator))
          // don't add "/" slash else we need to use /graphql/
          .service(web::resource("").guard(guard::Post()).to(index))
          .service(web::resource("").guard(guard::Get()).guard(guard::Header("upgrade", "websocket")).to(index_ws)),
      )
  })
  .keep_alive(Duration::from_secs(HTTP_SERVER_KEEP_ALIVE));

  if http_server_enable_https.eq("true") {
    info!(
      "start {} graphql server at: '{}', apiKey: '{}...', certificates '{}', '{}'",
      APP_NAME,
      http_server_uri,
      &http_server_api_key[..10],
      cert_file_name_key,
      cert_file_name_cert
    );
    // prepare ssl builder
    let cert_file_name_key = format!("{}/{}", config_path_ssl, cert_file_name_key);
    let cert_file_name_cert = format!("{}/{}", config_path_ssl, cert_file_name_cert);
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder.set_private_key_file(cert_file_name_key.clone(), SslFiletype::PEM).unwrap();
    builder.set_certificate_chain_file(cert_file_name_cert.clone()).unwrap();
    // start server
    http_server.bind_openssl(http_server_uri, builder)?.run().await
  } else {
    info!("start {} graphql server at: '{}', apiKey: '{}...'", APP_NAME, http_server_uri, &http_server_api_key[..10]);
    // start server
    http_server.bind(http_server_uri)?.run().await
  }
}
