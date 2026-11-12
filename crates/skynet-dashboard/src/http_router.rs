//! `http_router`: Async HTTP + RFC 6455 `60fps` WebSocket server router for `skynet-dashboard`.

use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::dto::DashboardSnapshot;
use crate::live_world::{DashboardError, LiveWorld};
use crate::preprocess_api;
use crate::worker_studio_api::{StudioAction, WorkerStudioController};
use crate::world_actions::ControlCmd;
use crate::ws_rfc6455::{extract_ws_upgrade_key, stream_ws_telemetry_60fps};

pub const DEFAULT_BIND_HOST: &str = "127.0.0.1";
pub const DEFAULT_DASHBOARD_PORT: u16 = 8888;
const WORLD_TICK_INTERVAL_MS: u64 = 400;
const DASHBOARD_HTML: &str = include_str!("../assets/dashboard.html");
const WORKER_STUDIO_JS: &str = include_str!("../assets/worker_studio.js");

pub fn resolve_dashboard_port() -> u16 {
    for key in ["SKYNET_PORT", "PORT"] {
        if let Ok(val) = std::env::var(key) {
            if let Ok(port) = val.parse::<u16>() {
                return port;
            }
        }
    }
    DEFAULT_DASHBOARD_PORT
}

#[derive(Clone)]
pub struct DashboardAppState {
    pub world: Arc<Mutex<LiveWorld>>,
    pub studio: Arc<RwLock<WorkerStudioController>>,
    pub cached_snapshot: Arc<RwLock<DashboardSnapshot>>,
}

impl DashboardAppState {
    pub async fn init() -> Result<Self, DashboardError> {
        let world = LiveWorld::init().await?;
        let studio = WorkerStudioController::new();
        let initial = world.snapshot_with_studio(&studio.telemetry_dto(), &studio.sweep_log_vec());
        Ok(Self {
            world: Arc::new(Mutex::new(world)),
            studio: Arc::new(RwLock::new(studio)),
            cached_snapshot: Arc::new(RwLock::new(initial)),
        })
    }

    pub fn spawn_background_ticker(&self) -> JoinHandle<()> {
        let bg = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(WORLD_TICK_INTERVAL_MS));
            loop {
                interval.tick().await;
                let mut w = bg.world.lock().await;
                w.step().await;
                let (dto, logs) = {
                    let mut st = bg.studio.write().unwrap_or_else(|p| p.into_inner());
                    st.tick_opportunistic();
                    (st.telemetry_dto(), st.sweep_log_vec())
                };
                let snap = w.snapshot_with_studio(&dto, &logs);
                *bg.cached_snapshot.write().unwrap_or_else(|p| p.into_inner()) = snap;
            }
        })
    }

    pub fn latest_snapshot_json(&self) -> String {
        let snap = self.cached_snapshot.read().unwrap_or_else(|p| p.into_inner());
        serde_json::to_string(&*snap).unwrap_or_else(|_| "{}".to_string())
    }

    pub fn worker_studio_json(&self) -> String {
        let mut st = self.studio.write().unwrap_or_else(|p| p.into_inner());
        st.refresh_from_disk(None);
        st.telemetry_json()
    }

    pub async fn apply_http_control(&self, path: &str) {
        let (action, val) = parse_control_query(path);
        if let Some(studio_act) = StudioAction::parse(action) {
            let (msg, dto, logs) = {
                let mut st = self.studio.write().unwrap_or_else(|p| p.into_inner());
                let m = st.apply_action(studio_act);
                (m, st.telemetry_dto(), st.sweep_log_vec())
            };
            {
                let mut snap = self.cached_snapshot.write().unwrap_or_else(|p| p.into_inner());
                snap.worker_studio = dto;
                snap.yield_sweep_log = logs;
                snap.events.insert(0, msg.clone());
                snap.events.truncate(20);
            }
            let bg_world = Arc::clone(&self.world);
            tokio::spawn(async move { bg_world.lock().await.push_event(msg); });
            return;
        }
        let mut w = self.world.lock().await;
        apply_world_control_action(&mut w, action, val);
        let (dto, logs) = {
            let st = self.studio.read().unwrap_or_else(|p| p.into_inner());
            (st.telemetry_dto(), st.sweep_log_vec())
        };
        *self.cached_snapshot.write().unwrap_or_else(|p| p.into_inner()) = w.snapshot_with_studio(&dto, &logs);
    }
}

/// Binds an ephemeral or fixed TCP address and spawns both ticker and HTTP/WS accept loops.
pub async fn bind_and_serve(bind_addr: &str) -> Result<(SocketAddr, JoinHandle<()>), DashboardError> {
    let state = DashboardAppState::init().await?;
    let listener = TcpListener::bind(bind_addr).await?;
    let local_addr = listener.local_addr()?;
    let ticker = state.spawn_background_ticker();
    let server = tokio::spawn(async move {
        let _keep_ticker = ticker;
        while let Ok((stream, _)) = listener.accept().await {
            tokio::spawn(handle_connection(stream, state.clone()));
        }
    });
    Ok((local_addr, server))
}

pub async fn handle_connection(mut stream: TcpStream, state: DashboardAppState) {
    let mut buf = [0u8; 8192];
    let Ok(n) = stream.read(&mut buf).await else { return };
    if n == 0 { return; }
    let req = String::from_utf8_lossy(&buf[..n]);
    let mut parts = req.lines().next().unwrap_or("").split_whitespace();
    let method = parts.next().unwrap_or("GET");
    let path = parts.next().unwrap_or("/");
    let origin = extract_header(&req, "Origin");
    let sec_fetch_site = extract_header(&req, "Sec-Fetch-Site");

    if is_cross_site_blocked(method, path, origin, sec_fetch_site) {
        let _ = stream.write_all(b"HTTP/1.1 403 Forbidden\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\nForbidden: Cross-site control rejected").await;
        return;
    }
    if method == "GET" && path.starts_with("/ws/telemetry") {
        if let Some(ws_key) = extract_ws_upgrade_key(&req) {
            let _ = stream_ws_telemetry_60fps(stream, &ws_key, move || state.latest_snapshot_json()).await;
            return;
        }
    }
    let cors = if is_allowed_origin(origin) && !origin.is_empty() { origin } else { "http://127.0.0.1:8888" };
    let (status, content_type, body) = route_http_request(&state, method, path, &req).await;
    let resp = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: {cors}\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nVary: Origin\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(resp.as_bytes()).await;
}

async fn route_http_request(state: &DashboardAppState, method: &str, path: &str, req: &str) -> (&'static str, &'static str, String) {
    match (method, path) {
        ("OPTIONS", _) => ("204 No Content", "text/plain", String::new()),
        (_, p) if p == "/" || p.starts_with("/index.html") => ("200 OK", "text/html; charset=utf-8", DASHBOARD_HTML.to_string()),
        (_, p) if p.starts_with("/assets/worker_studio.js") => ("200 OK", "application/javascript; charset=utf-8", WORKER_STUDIO_JS.to_string()),
        (_, p) if p.starts_with("/api/worker-studio") => ("200 OK", "application/json; charset=utf-8", state.worker_studio_json()),
        (_, p) if p.starts_with("/api/state") => ("200 OK", "application/json; charset=utf-8", state.latest_snapshot_json()),
        ("POST", p) if p.starts_with("/api/control") => {
            state.apply_http_control(p).await;
            ("200 OK", "application/json; charset=utf-8", "{\"ok\":true}".to_string())
        }
        (_, p) if p.starts_with("/api/presets") => ("200 OK", "application/json; charset=utf-8", preprocess_api::handle_presets_request()),
        ("POST", p) if p.starts_with("/api/preprocess") => {
            let res = preprocess_api::handle_preprocess_request(preprocess_api::extract_http_body(req));
            ("200 OK", "application/json; charset=utf-8", res)
        }
        _ => ("404 Not Found", "text/plain", "Not Found".to_string()),
    }
}

fn parse_control_query(path: &str) -> (&str, &str) {
    let query = path.split('?').nth(1).unwrap_or("");
    let (mut action, mut val) = ("", "");
    for kv in query.split('&') {
        let Some((k, v)) = kv.split_once('=') else { continue };
        if k == "action" { action = v; }
        if k == "value" { val = v; }
    }
    (action, val)
}

fn apply_world_control_action(w: &mut LiveWorld, action: &str, val: &str) {
    match action {
        "loss" => if let Ok(r) = val.parse::<f64>() { w.apply_control(ControlCmd::SetLoss(r)); },
        "flood_ips" => w.apply_control(ControlCmd::FloodIps),
        "cluster_mitosis" => w.apply_control(ControlCmd::ClusterMitosis),
        "double_spend" => w.apply_control(ControlCmd::DoubleSpend),
        "reclaim_iot" => w.apply_control(ControlCmd::ReclaimIot),
        _ => {}
    }
}

fn is_cross_site_blocked(method: &str, path: &str, origin: &str, sec_fetch_site: &str) -> bool {
    let protected = (method == "POST" && path.starts_with("/api/control")) || path.starts_with("/ws/telemetry");
    protected && (sec_fetch_site.eq_ignore_ascii_case("cross-site") || (!origin.is_empty() && !is_allowed_origin(origin)))
}

fn is_allowed_origin(origin: &str) -> bool {
    origin.is_empty() || ["http://127.0.0.1", "http://localhost", "https://127.0.0.1", "https://localhost"].iter().any(|p| origin.starts_with(p))
}

fn extract_header<'a>(req: &'a str, header_name: &str) -> &'a str {
    req.lines().skip(1).find_map(|l| l.split_once(':').and_then(|(k, v)| k.trim().eq_ignore_ascii_case(header_name).then_some(v.trim()))).unwrap_or("")
}
