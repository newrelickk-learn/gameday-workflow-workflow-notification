use actix_web::web;

use crate::api::handlers;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .service(
                web::scope("/workflows")
                    .route(
                        "/start",
                        web::post().to(handlers::workflow::start_workflow),
                    )
                    .route(
                        "/validate-approval",
                        web::post().to(handlers::workflow::validate_approval),
                    )
                    .route(
                        "/approve",
                        web::post().to(handlers::workflow::approve_workflow),
                    )
                    .route(
                        "/definition",
                        web::get().to(handlers::workflow::get_workflow_definition),
                    ),
            )
            .service(
                web::scope("/notifications")
                    .route(
                        "/send",
                        web::post().to(handlers::notification::send_notification),
                    )
                    .route(
                        "/history",
                        web::get().to(handlers::notification::get_notification_history),
                    ),
            )
            .route("/health", web::get().to(handlers::health::health_check)),
    );
}

