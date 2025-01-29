use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use axum::{response::IntoResponse, routing::get, Router};
use tower_http::trace::{TraceLayer};
use tower_http::classify::{SharedClassifier, ServerErrorsAsFailures};
use metrics::{counter, gauge, histogram};


pub fn setup_metrics_recorder() -> PrometheusHandle {
    PrometheusBuilder::new()
        .add_global_label("service", env!("CARGO_PKG_NAME"))
        .install_recorder()
        .expect("failed to install Prometheus recorder")
}

/// System resource metrics
pub fn record_system_metrics() {
  if let Ok(mem_info) = sys_info::mem_info() {
      gauge!("memory_total_bytes", mem_info.total as f64 * 1024.0);
      gauge!("memory_free_bytes", mem_info.free as f64 * 1024.0);
      gauge!("memory_avail_bytes", mem_info.avail as f64 * 1024.0);
  }
  
  if let Ok(cpu) = sys_info::loadavg() {
      gauge!("cpu_load_1m", cpu.one);
      gauge!("cpu_load_5m", cpu.five);
      gauge!("cpu_load_15m", cpu.fifteen);
  }
}

/// Metrics for REST API endpoints
pub fn record_api_metrics(endpoint: &str, status: u16, duration_ms: f64) {
  counter!("http_requests_total", 1, "endpoint" => endpoint.to_string(), "status" => status.to_string());
  histogram!("http_request_duration_ms", duration_ms, "endpoint" => endpoint.to_string());
}

/// Metrics for gRPC raw VAAs processing metrics
pub fn record_vaa_metrics(total: usize, unique: usize, duplicates: usize, gaps: usize) {
    counter!("vaa_processed_total", total as u64);
    counter!("vaa_unique_total", unique as u64);
    counter!("vaa_duplicates_total", duplicates as u64);
    counter!("vaa_sequence_gaps_total", gaps as u64);
}

/// Handler for the /metrics endpoint for Prometheus metrics
pub async fn metrics_handler(metrics: PrometheusHandle) -> impl IntoResponse {
    metrics.render()
}

/// Middleware layer for HTTP request instrumentation
pub fn create_metrics_layer() -> TraceLayer<SharedClassifier<ServerErrorsAsFailures>> {
    TraceLayer::new_for_http()
}

/// Inject metrics routes to the router
pub fn add_metrics_routes(router: Router, prometheus_handle: PrometheusHandle) -> Router {
    router.route("/metrics", get(move || metrics_handler(prometheus_handle.clone())))
}