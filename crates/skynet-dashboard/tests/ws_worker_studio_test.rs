//! In-process integration tests for `skynet-dashboard`: RFC 6455 WebSocket upgrade,
//! `60fps` (`16ms`) telemetry streaming, `< 100ms` Studio action reflection, and HTTP APIs.

use std::net::SocketAddr;
use std::time::{Duration, Instant};

use skynet_dashboard::{
    bind_and_serve, decode_ws_server_frame, DashboardSnapshot, WorkerStudioTelemetryDto,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

struct WsTestClient {
    stream: TcpStream,
    buf: Vec<u8>,
}

impl WsTestClient {
    async fn connect(addr: SocketAddr, ws_key: &str) -> (Self, String) {
        let mut stream = TcpStream::connect(addr).await.expect("connect ws");
        let req = format!(
            "GET /ws/telemetry HTTP/1.1\r\nHost: {addr}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: {ws_key}\r\nSec-WebSocket-Version: 13\r\n\r\n"
        );
        stream.write_all(req.as_bytes()).await.expect("write ws upgrade");
        let mut raw = Vec::with_capacity(4096);
        let mut chunk = [0u8; 2048];
        let split_idx = loop {
            let n = stream.read(&mut chunk).await.expect("read ws handshake");
            assert!(n > 0, "unexpected EOF during handshake");
            raw.extend_from_slice(&chunk[..n]);
            if let Some(pos) = raw.windows(4).position(|w| w == b"\r\n\r\n") {
                break pos + 4;
            }
        };
        let headers = String::from_utf8_lossy(&raw[..split_idx]).into_owned();
        let leftover = raw[split_idx..].to_vec();
        (Self { stream, buf: leftover }, headers)
    }

    async fn next_snapshot(&mut self) -> DashboardSnapshot {
        let mut chunk = [0u8; 8192];
        loop {
            if let Some((opcode, payload, used)) = decode_ws_server_frame(&self.buf) {
                self.buf.drain(..used);
                assert_eq!(opcode, 0x1, "expected text frame opcode 0x1");
                return serde_json::from_str::<DashboardSnapshot>(&payload).expect("valid snapshot JSON");
            }
            let n = tokio::time::timeout(Duration::from_millis(500), self.stream.read(&mut chunk))
                .await
                .expect("ws frame timeout")
                .expect("ws socket read");
            assert!(n > 0, "unexpected WS stream EOF");
            self.buf.extend_from_slice(&chunk[..n]);
        }
    }

    async fn wait_until<P>(&mut self, max_ms: u64, pred: P) -> (DashboardSnapshot, Duration)
    where
        P: Fn(&DashboardSnapshot) -> bool,
    {
        let start = Instant::now();
        let deadline = Duration::from_millis(max_ms);
        loop {
            let snap = self.next_snapshot().await;
            let elapsed = start.elapsed();
            if pred(&snap) {
                return (snap, elapsed);
            }
            assert!(elapsed < deadline, "condition not reflected within {max_ms}ms (elapsed {elapsed:?})");
        }
    }
}

async fn send_http_raw(addr: SocketAddr, raw_req: &str) -> String {
    let mut stream = TcpStream::connect(addr).await.expect("connect http");
    stream.write_all(raw_req.as_bytes()).await.expect("write http");
    let mut out = Vec::new();
    stream.read_to_end(&mut out).await.expect("read http");
    String::from_utf8_lossy(&out).into_owned()
}

async fn post_control(addr: SocketAddr, query: &str) {
    let req = format!(
        "POST /api/control?{query} HTTP/1.1\r\nHost: {addr}\r\nOrigin: http://127.0.0.1\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    );
    let resp = send_http_raw(addr, &req).await;
    assert!(resp.starts_with("HTTP/1.1 200 OK"), "unexpected control response: {resp}");
}

fn split_http_body(resp: &str) -> &str {
    resp.split_once("\r\n\r\n").map(|(_, b)| b).unwrap_or("")
}

#[tokio::test]
async fn ws_rfc6455_60fps_and_sub_100ms_studio_actions_reflect() {
    let (addr, server_task) = bind_and_serve("127.0.0.1:0").await.expect("bind dashboard");
    let (mut ws, headers) = WsTestClient::connect(addr, "dGhlIHNhbXBsZSBub25jZQ==").await;
    assert!(headers.starts_with("HTTP/1.1 101 Switching Protocols"));
    assert!(headers.contains("Sec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo="));

    let burst_start = Instant::now();
    for _ in 0..5 {
        let snap = ws.next_snapshot().await;
        assert_eq!(snap.agents.len(), 8);
        assert!(snap.supply.conservation_ok);
    }
    assert!(burst_start.elapsed() < Duration::from_millis(250), "60fps burst took too long");
    assert_all_four_studio_actions_sub_100ms(addr, &mut ws).await;
    server_task.abort();
}

async fn assert_all_four_studio_actions_sub_100ms(addr: SocketAddr, ws: &mut WsTestClient) {
    post_control(addr, "action=sim_game_launch").await;
    let (s_game, dt_game) = ws.wait_until(100, |s| s.worker_studio.governor_state == "PausedGaming" && s.worker_studio.active_game_detected).await;
    assert!(dt_game < Duration::from_millis(100) && s_game.worker_studio.preempt_latency_ms < 100.0);

    post_control(addr, "action=sim_thermal_spike").await;
    let (s_therm, dt_therm) = ws.wait_until(100, |s| s.worker_studio.governor_state == "PausedThermal" && s.worker_studio.temp_c >= 78).await;
    assert!(dt_therm < Duration::from_millis(100) && s_therm.worker_studio.temp_c == 82);

    post_control(addr, "action=sim_canary_fraud").await;
    let (s_fraud, dt_fraud) = ws.wait_until(100, |s| s.worker_studio.slashed_count >= 1 && s.worker_studio.pouw_score_bps == 0).await;
    assert!(dt_fraud < Duration::from_millis(100) && s_fraud.worker_studio.slashed_count == 1);

    post_control(addr, "action=sim_yield_sweep").await;
    let (s_sweep, dt_sweep) = ws.wait_until(100, |s| s.worker_studio.swept_to_cold_nano_skn > 0 && !s.yield_sweep_log.is_empty()).await;
    assert!(dt_sweep < Duration::from_millis(100) && s_sweep.worker_studio.swept_to_cold_nano_skn >= 50_000_000_000);
}

#[tokio::test]
async fn http_endpoints_assets_and_sec10_origin_guard_work() {
    let (addr, server_task) = bind_and_serve("127.0.0.1:0").await.expect("bind dashboard");

    let html = send_http_raw(addr, &format!("GET / HTTP/1.1\r\nHost: {addr}\r\n\r\n")).await;
    assert!(html.contains("200 OK") && html.contains("workerStudioPanel") && html.contains("/assets/worker_studio.js"));

    let js = send_http_raw(addr, &format!("GET /assets/worker_studio.js HTTP/1.1\r\nHost: {addr}\r\n\r\n")).await;
    assert!(js.contains("200 OK") && js.contains("/ws/telemetry") && js.contains("sim_yield_sweep"));

    let studio_resp = send_http_raw(addr, &format!("GET /api/worker-studio HTTP/1.1\r\nHost: {addr}\r\n\r\n")).await;
    let dto: WorkerStudioTelemetryDto = serde_json::from_str(split_http_body(&studio_resp)).expect("studio dto");
    assert_eq!(dto.governor_state, "DeepIdle");

    let presets = send_http_raw(addr, &format!("GET /api/presets HTTP/1.1\r\nHost: {addr}\r\n\r\n")).await;
    assert!(presets.contains("200 OK") && split_http_body(&presets).starts_with('['));

    let cross_site = send_http_raw(
        addr,
        &format!("POST /api/control?action=sim_game_launch HTTP/1.1\r\nHost: {addr}\r\nOrigin: https://evil.example\r\nSec-Fetch-Site: cross-site\r\n\r\n"),
    )
    .await;
    assert!(cross_site.starts_with("HTTP/1.1 403 Forbidden"));

    server_task.abort();
}
