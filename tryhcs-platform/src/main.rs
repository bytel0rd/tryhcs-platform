use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use tryhcs_commons_be::{env::EnvConfig, file_upload::get_upload_client, redis::RedisCache};
use tryhcs_compliance_be::{
    app::ComplianceApp, endpoints::compliance_router, integrations::youverify::YouverifyApi,
    repo::ComplianceDB,
};
use tryhcs_customers_be::{app::CustomersApp, db_repo::CustomerDB, endpoint::customers_router};

use eyre::Context;
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt().init();

    let env: EnvConfig = envy::from_env::<EnvConfig>().wrap_err("loaded config files")?;

    info!("Connecting to databases");
    let customer_db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&env.database_url)
        .await
        .wrap_err("Failed to connect to db")?;

    let customer_db = Arc::new(CustomerDB {
        customer_db: customer_db_pool.clone(),
    });
    info!("Connected to customer database");

    let compliance_db = Arc::new(ComplianceDB {
        customer_db: customer_db_pool.clone(),
    });
    info!("Connected to compliance database");

    let redis_client = Arc::new(RedisCache(redis::Client::open(env.redis_url.as_str())?));
    info!("connected to redis");

    let youverify_api: Arc<YouverifyApi> = Arc::new(YouverifyApi {
        env: Arc::new(env.clone()),
    });
    let s3_client = get_upload_client(&env).await?;

    // Create Arc<CustomerApp> and Arc<ComplianceApp>
    let customer_app: Arc<CustomersApp> = Arc::new(CustomersApp {
        db_pool: customer_db,
        s3_client: s3_client.clone(),
        env: env.clone(),
        redis: redis_client.clone(),
    });
    let compliance_app = Arc::new(ComplianceApp {
        compliance: youverify_api,
        env: env.clone(),
        redis: redis_client.clone(),
        compliance_repo: compliance_db,
    });

    let app_router = app_router(customer_app.clone(), compliance_app.clone()).await?;
    info!("Mounted app routes");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4080")
        .await
        .wrap_err("failed to bind to port")?;

    info!("Listening on 0.0.0.0:4080");
    axum::serve(listener, app_router)
        .await
        .wrap_err("Unable to serve router")?;

    Ok(())
}

async fn app_router(
    customer_app: Arc<CustomersApp>,
    compliance_app: Arc<ComplianceApp>,
) -> eyre::Result<Router> {
    let cors_layer = CorsLayer::new()
        .allow_methods(Any)
        .allow_origin(Any)
        .allow_headers(Any);

    let router = Router::new()
        .route("/", get(health_info))
        .nest("/workspace/v1", customers_router(customer_app.clone()))
        .nest("/compliance", compliance_router(compliance_app.clone()))
        .layer(ServiceBuilder::new().layer(cors_layer));

    Ok(router)
}

async fn health_info() -> impl IntoResponse {
    "Alive"
}
