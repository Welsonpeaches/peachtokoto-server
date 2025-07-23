use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use crate::config::SwaggerConfig;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::meme::random_meme,
        crate::handlers::meme::list_memes,
        crate::handlers::meme::get_meme_by_id,
        crate::handlers::meme::get_meme_count,
        crate::handlers::meme::health_check,
        crate::handlers::statistics::get_statistics
    ),
    components(
        schemas(
            crate::handlers::meme::RandomMemeQuery,
            crate::handlers::meme::GetMemeQuery,
            crate::handlers::meme::MemeListItem,
            crate::handlers::meme::MemeCount,
            crate::handlers::statistics::Statistics
        )
    ),
    tags(
        (name = "memes", description = "表情包相关API"),
        (name = "statistics", description = "统计信息API")
    )
)]
pub struct ApiDoc;

pub fn create_openapi_spec(config: &SwaggerConfig) -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    
    // 更新 info 部分
    openapi.info.title = config.title.clone();
    openapi.info.description = Some(config.description.clone());
    openapi.info.version = config.version.clone();
    
    // 更新联系信息
    openapi.info.contact = Some(utoipa::openapi::ContactBuilder::new()
        .name(Some(config.contact_name.clone()))
        .email(Some(config.contact_email.clone()))
        .build());
    
    // 更新服务器信息
    openapi.servers = Some(vec![
        utoipa::openapi::ServerBuilder::new()
            .url(config.server_url.clone())
            .description(Some(config.server_description.clone()))
            .build()
    ]);
    
    openapi
}

pub fn create_swagger_ui(config: SwaggerConfig) -> SwaggerUi {
    let openapi_spec = create_openapi_spec(&config);
    SwaggerUi::new(config.endpoint)
        .url("/api-docs/openapi.json", openapi_spec)
}