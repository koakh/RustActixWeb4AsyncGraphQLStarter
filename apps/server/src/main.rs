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
use common::{config::get_config, logger::init_log4rs};
use log::{error, info};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use serde::{Deserialize, Serialize};
use server::constants::{APP_NAME, GRAPHQL_PATH, HTTP_SERVER_KEEP_ALIVE, PLAYGROUND_PATH};
use std::{time::Duration};

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
  // env vars: log
  let config = get_config();
  // // init local log variables
  let log_level = config.log.log_level.as_ref().unwrap().to_owned();
  let log_file_level = config.log.log_file_level.as_ref().unwrap().to_owned();
  let log_file_path = config.log.log_file_path.as_ref().unwrap().to_owned();
  let server_uri = config.server.uri.as_ref().unwrap().to_owned();
  let server_enable_https = config.server.enable_https.as_ref().unwrap().to_owned();
  let server_api_key = config.server.api_key.as_ref().unwrap().to_owned();
  let certificate_config_path = config.certificate.config_path.as_ref().unwrap().to_owned();
  let certificate_file_name_key = config.certificate.file_name_key.as_ref().unwrap().to_owned();
  let certificate_file_name_cert = config.certificate.file_name_cert.as_ref().unwrap().to_owned();
  // init log4rs
  init_log4rs(&log_file_path, &log_level, &log_file_level).expect("can't initialize log4rs logger");

  // authentication validator
  // required to implement ResponseError in src/app/errors.rs else we have a error
  // Err(AuthenticationError::from(config).into())
  async fn validator(req: ServiceRequest, credentials: BearerAuth) -> Result<ServiceRequest, ActixError> {
    let config = get_config();
    let http_server_api_key = config.server.api_key.as_ref().unwrap().to_owned();
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

  if server_enable_https {
    info!(
      "start {} graphql server at: '{}', apiKey: '{}...', certificates '{}', '{}'",
      APP_NAME,
      server_uri,
      &server_api_key[..10],
      certificate_file_name_key,
      certificate_file_name_cert,
    );
    // prepare ssl builder
    let certificate_file_path_key = format!("{}/{}", certificate_config_path, certificate_file_name_key);
    let certificate_file_path_cert = format!("{}/{}", certificate_config_path, certificate_file_name_cert);
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder.set_private_key_file(certificate_file_path_key.clone(), SslFiletype::PEM).unwrap();
    builder.set_certificate_chain_file(certificate_file_path_cert.clone()).unwrap();
    // start server
    http_server.bind_openssl(server_uri, builder)?.run().await
  } else {
    info!("start {} graphql server at: '{}', apiKey: '{}...'", APP_NAME, &server_uri, &server_api_key[..10]);
    // start server
    http_server.bind(&server_uri)?.run().await
  }
}
