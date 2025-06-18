use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

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
    ),
    info(
        title = "Jiangtokoto Server API",
        description = "表情包服务器API文档",
        version = "1.0.0",
        contact(
            name = "API Support",
            email = "support@example.com"
        )
    ),
    servers(
        (url = "https://api.jiangtokoto.cn", description = "生产服务器")
    )
)]
pub struct ApiDoc;

pub fn swagger_ui() -> SwaggerUi {
    SwaggerUi::new("/swagger-ui")
        .url("/api-docs/openapi.json", ApiDoc::openapi())
} 