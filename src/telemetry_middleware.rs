use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};
use std::time::Instant;
use tracing::{info, info_span, Instrument};

/// OpenTelemetry Semantic Conventionsに準拠したHTTPリクエストスパンを作成するミドルウェア。
/// `tracing_actix_web::TracingLogger`はコンソールログ用のスパンしか作らないため、
/// OTelエクスポート用にはこちらに置き換える（両方wrapすると二重計装になる）。
pub struct TelemetryMiddleware;

impl<S, B> Transform<S, ServiceRequest> for TelemetryMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = TelemetryMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(TelemetryMiddlewareService { service }))
    }
}

pub struct TelemetryMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for TelemetryMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let start = Instant::now();
        let method = req.method().to_string();
        let path = req.path().to_string();
        let route = req
            .match_pattern()
            .unwrap_or_else(|| path.clone());
        let connection_info = req.connection_info().clone();
        let server_address = connection_info.host().to_string();

        let span = info_span!(
            "http_request",
            http.request.method = %method,
            http.route = %route,
            server.address = %server_address,
            url.path = %path,
            url.scheme = "http",
            otel.kind = "server",
            http.response.status_code = tracing::field::Empty,
        );

        let fut = self.service.call(req);

        Box::pin(
            async move {
                let res = fut.await?;
                let status_code = res.status().as_u16();
                let duration_ms = start.elapsed().as_millis() as f64;

                tracing::Span::current().record("http.response.status_code", status_code);

                info!(
                    http.response.status_code = status_code,
                    duration_ms = duration_ms,
                    "{} {} -> {}",
                    method,
                    path,
                    status_code
                );

                Ok(res)
            }
            .instrument(span),
        )
    }
}
