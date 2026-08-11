mod api;
mod domain;
mod infrastructure;
mod services;
mod telemetry_middleware;

use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use sea_orm::{Database, DatabaseConnection};
use std::sync::Arc;

use api::routes;
use infrastructure::db::WorkflowRepository;
use rust_tracing_otel::TelemetryManager;
use services::workflow_service::WorkflowServiceImpl;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 環境変数の読み込み
    dotenv().ok();

    // テレメトリ（tracing + OpenTelemetry）の初期化
    // この呼び出しがグローバルなtracing subscriberを設定するため、
    // 他にtracing_subscriber::fmt().init()等を呼んではいけない（二重初期化になる）
    let telemetry = Arc::new(
        TelemetryManager::from_env().expect("Failed to create telemetry manager"),
    );
    telemetry
        .init()
        .await
        .expect("Failed to initialize telemetry");

    // データベース接続の作成
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let db: DatabaseConnection = Database::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    tracing::info!("Connected to database");

    // リポジトリとサービスの作成
    let notification_repository = WorkflowRepository::new(db.clone());
    let notification_service_for_workflow = services::notification_service::NotificationServiceImpl::new(notification_repository);
    let notification_service_box: Box<dyn services::notification_service::NotificationService> = Box::new(notification_service_for_workflow);
    
    let workflow_repository = WorkflowRepository::new(db.clone());
    let workflow_service = WorkflowServiceImpl::new(workflow_repository, notification_service_box);
    
    // 通知サービス用のリポジトリ（app_data用）
    let notification_repository_for_app = WorkflowRepository::new(db);
    let notification_service_for_app = services::notification_service::NotificationServiceImpl::new(notification_repository_for_app);

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8003".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid number");

    tracing::info!("Starting server on port {}", port);

    // サービスをAppのデータとして登録
    let workflow_service_data = web::Data::new(workflow_service);
    let notification_service_data = web::Data::new(notification_service_for_app);

    let result = HttpServer::new(move || {
        App::new()
            .app_data(workflow_service_data.clone())
            .app_data(notification_service_data.clone())
            .wrap(telemetry_middleware::TelemetryMiddleware)
            .configure(routes::configure)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await;

    if let Err(e) = telemetry.shutdown().await {
        tracing::error!("Failed to shutdown telemetry: {}", e);
    }

    result
}

