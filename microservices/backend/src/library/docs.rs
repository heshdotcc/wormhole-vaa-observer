use std::sync::Arc;
use aide::{
    axum::{
        routing::{get, get_with},
        ApiRouter, IntoApiResponse,
    },
    OperationOutput,
    openapi::{OpenApi, Tag},
    scalar::Scalar,
    swagger::Swagger,
    transform::TransformOpenApi,
};
use axum::{
    response::IntoResponse,
    Extension,
    Json,
    http::StatusCode,
};
use crate::{
    state::AppState,
    library::errors::AppError,
};
use uuid::Uuid;
use super::config::get_config;

impl OperationOutput for AppError {
    type Inner = Self;
}

pub fn docs_routes(state: Arc<AppState>) -> ApiRouter {
    let config = get_config();
    aide::generate::infer_responses(true);

    let router: ApiRouter = ApiRouter::new()
        .api_route_with(
            "/",
            get_with(
                Scalar::new("/docs/private/api.json")
                    .with_title(&config.api_title)
                    .axum_handler(),
                |op| op.description("This documentation page."),
            ),
            |p| p.security_requirement("ApiKey"),
        )
        .api_route_with(
            "/swagger",
            get_with(
                Swagger::new("/docs/private/api.json")
                    .with_title(&config.api_title)
                    .axum_handler(),
                |op| op.description("This documentation page."),
            ),
            |p| p.security_requirement("ApiKey"),
        )
        .route("/private/api.json", get(serve_docs))
        .with_state(state);

    aide::generate::infer_responses(false);

    router // implicit return
}

pub fn configure_api_docs(api: TransformOpenApi) -> TransformOpenApi {
  let config = get_config();
  
  api.title(&config.api_title)
      .summary("wormhole-vaa-observer")
      // .description(include_str!("README.md"))
      .tag(Tag {
          name: "wormhole".into(),
          description: Some("Wormhole VAA Endpoints".into()),
          ..Default::default()
      })
      .tag(Tag {
          name: "health".into(),
          description: Some("Health-check Endpoints".into()),
          ..Default::default()
      })
      .security_scheme(
          "ApiKey",
          aide::openapi::SecurityScheme::ApiKey {
              location: aide::openapi::ApiKeyLocation::Header,
              name: "X-Auth-Key".into(),
              description: Some("A key that is ignored.".into()),
              extensions: Default::default(),
          },
      )
      .default_response_with::<AppError, _>(|res| {
          res.example(AppError {
              error: "some error happened".to_string(),
              error_details: None,
              error_id: Uuid::nil(),
              status: StatusCode::IM_A_TEAPOT,
          })
      })
}

async fn serve_docs(Extension(api): Extension<Arc<OpenApi>>) -> impl IntoApiResponse {
    Json(api).into_response()
}