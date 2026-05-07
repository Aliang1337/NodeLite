use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::{Json, Router};
use clap::Parser;
use serde::Serialize;
use tokio::fs;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};
use ximonitor_proto::{ServerConfig, parse_server_config};

#[derive(Debug, Parser)]
#[command(name = "ximonitor-server")]
#[command(about = "XiMonitor central server")]
struct Cli {
    #[arg(long, default_value = "config/server.toml")]
    config: PathBuf,
}

#[derive(Clone)]
struct AppState {
    config: Arc<ServerConfig>,
}

#[derive(Debug, Serialize)]
struct BootstrapResponse {
    service: &'static str,
    status: &'static str,
    public_base_url: String,
    refresh_interval_secs: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    let cli = Cli::parse();
    let config = load_server_config(&cli.config).await?;
    let listen_addr = config.listen;
    let public_base_url = config.public_base_url.clone();
    let refresh_interval_secs = config.refresh_interval_secs;
    let state = AppState {
        config: Arc::new(config),
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/healthz", get(healthz))
        .route("/api/bootstrap", get(bootstrap))
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    let listener = TcpListener::bind(listen_addr)
        .await
        .with_context(|| format!("failed to bind server listener to {listen_addr}"))?;

    info!(
        listen = %listen_addr,
        public_base_url = %public_base_url,
        refresh_interval_secs,
        "ximonitor server listening",
    );

    axum::serve(listener, app)
        .await
        .context("server exited unexpectedly")
}

async fn load_server_config(path: &Path) -> Result<ServerConfig> {
    let content = fs::read_to_string(path)
        .await
        .with_context(|| format!("failed to read config file {}", path.display()))?;
    let config = parse_server_config(&content)
        .map_err(|error| anyhow!("failed to parse {}: {error}", path.display()))?;

    if let Some(parent) = config.snapshot_path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            warn!(
                snapshot_dir = %parent.display(),
                "snapshot directory does not exist yet; it will be created later",
            );
        }
    }
    if let Some(parent) = config.history_db_path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            warn!(
                history_dir = %parent.display(),
                "history directory does not exist yet; it will be created later",
            );
        }
    }

    Ok(config)
}

async fn index() -> Html<&'static str> {
    Html(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>XiMonitor</title>
    <style>
      :root {
        color-scheme: light;
        font-family: "Iowan Old Style", "Palatino Linotype", "Book Antiqua", serif;
        background: linear-gradient(135deg, #f3efe3 0%, #f9f7f1 55%, #e9f0f5 100%);
        color: #17212b;
      }
      body {
        margin: 0;
        min-height: 100vh;
        display: grid;
        place-items: center;
      }
      main {
        width: min(720px, calc(100vw - 32px));
        background: rgba(255, 255, 255, 0.82);
        border: 1px solid rgba(23, 33, 43, 0.08);
        border-radius: 24px;
        padding: 40px;
        box-shadow: 0 24px 80px rgba(23, 33, 43, 0.12);
        backdrop-filter: blur(20px);
      }
      h1 {
        font-size: clamp(2.5rem, 6vw, 4rem);
        line-height: 0.95;
        margin: 0 0 12px;
      }
      p {
        font-size: 1.1rem;
        line-height: 1.7;
        margin: 0;
      }
      code {
        font-family: "SFMono-Regular", "SF Mono", ui-monospace, monospace;
      }
    </style>
  </head>
  <body>
    <main>
      <h1>XiMonitor</h1>
      <p>The server bootstrap is running. Next commits will attach the node registry, WebSocket ingress, read-only APIs, and the dashboard.</p>
      <p style="margin-top: 18px;">Try <code>/healthz</code> or <code>/api/bootstrap</code> while the rest of the stack lands.</p>
    </main>
  </body>
</html>"#,
    )
}

async fn healthz() -> StatusCode {
    StatusCode::OK
}

async fn bootstrap(State(state): State<AppState>) -> impl IntoResponse {
    Json(BootstrapResponse {
        service: "ximonitor-server",
        status: "ok",
        public_base_url: state.config.public_base_url.clone(),
        refresh_interval_secs: state.config.refresh_interval_secs,
    })
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ximonitor_server=info,tower_http=info".into()),
        )
        .with_target(false)
        .compact()
        .init();
}
