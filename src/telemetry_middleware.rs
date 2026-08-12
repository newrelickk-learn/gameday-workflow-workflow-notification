use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures_util::future::LocalBoxFuture;
use rust_tracing_otel::TelemetryManager;
use std::future::{ready, Ready};
use std::sync::Arc;
use std::time::Instant;
use tracing::{info, info_span, Instrument};

/// OpenTelemetry Semantic Conventionsに準拠したHTTPリクエストスパン・メトリクスを
/// 記録するミドルウェア。
/// `tracing_actix_web::TracingLogger`はコンソールログ用のスパンしか作らないため、
/// OTelエクスポート用にはこちらに置き換える（両方wrapすると二重計装になる）。
///
/// スパンだけでなく`TelemetryManager`経由でメトリクス（http.server.request.duration等）
/// も記録する。New RelicのAPM UI（Summary/Transactionsページ）はスパンではなく
/// メトリクスを主なデータソースとして使うため、これが無いとAPMエンティティとして
/// 正しく認識されない（domain: APMではなくdomain: EXTになる）。
pub struct TelemetryMiddleware {
    telemetry: Arc<TelemetryManager>,
}

impl TelemetryMiddleware {
    pub fn new(telemetry: Arc<TelemetryManager>) -> Self {
        Self { telemetry }
    }
}

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
        ready(Ok(TelemetryMiddlewareService {
            service,
            telemetry: self.telemetry.clone(),
        }))
    }
}

pub struct TelemetryMiddlewareService<S> {
    service: S,
    telemetry: Arc<TelemetryManager>,
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
        let client_address = connection_info.realip_remote_addr().unwrap_or("").to_string();
        let network_protocol_version = match req.version() {
            actix_web::http::Version::HTTP_10 => "1.0",
            actix_web::http::Version::HTTP_11 => "1.1",
            actix_web::http::Version::HTTP_2 => "2",
            actix_web::http::Version::HTTP_3 => "3",
            _ => "",
        };
        let user_agent = req
            .headers()
            .get(actix_web::http::header::USER_AGENT)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        // スパン名はHTTP Semantic Conventions推奨の "{method} {route}" 形式にする
        // （otel.nameは動的なスパン名をtracing-opentelemetryに伝える予約フィールド名）
        let span_name = format!("{} {}", method, route);

        let span = info_span!(
            "http_request",
            otel.name = %span_name,
            otel.kind = "server",
            otel.status_code = tracing::field::Empty,
            http.request.method = %method,
            http.route = %route,
            http.response.status_code = tracing::field::Empty,
            network.protocol.version = %network_protocol_version,
            server.address = %server_address,
            client.address = %client_address,
            user_agent.original = %user_agent,
            url.path = %path,
            url.scheme = "http",
        );

        let telemetry = self.telemetry.clone();
        // http.server.active_requests (UpDownCounter): リクエスト開始で+1
        // （TelemetryManagerに委譲メソッドが無いため、metrics_provider()経由で呼ぶ）
        telemetry.metrics_provider().record_active_requests(&method, true);

        let fut = self.service.call(req);

        Box::pin(
            async move {
                let res = fut.await?;
                let status_code_u16 = res.status().as_u16();
                let status_code = status_code_u16 as i64;
                let duration_ms = start.elapsed().as_millis() as f64;

                let current_span = tracing::Span::current();
                // i64として渡すことで、New Relic側で文字列ではなく数値として扱われる
                // （u16はtracingのValue実装が無くDebugにフォールバックし文字列化されるため）
                current_span.record("http.response.status_code", status_code);
                // HTTP Semantic Conventions: サーバースパンは5xxの場合にErrorとする
                if status_code >= 500 {
                    current_span.record("otel.status_code", "ERROR");
                }

                // http.server.request.duration (Histogram) / http.server.requests (Counter)。
                // New RelicのAPM UIはこのメトリクスから apm.service.transaction.duration 等の
                // APM系メトリクスを合成する（スパンはサンプリングされるため補助的にしか使われない）。
                // status_code >= 400 の場合、ライブラリ側が自動でerror.type属性を付与する。
                telemetry.record_request_metrics(&route, &method, status_code_u16, duration_ms, None);
                // http.server.active_requests (UpDownCounter): リクエスト終了で-1
                telemetry.metrics_provider().record_active_requests(&method, false);

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
