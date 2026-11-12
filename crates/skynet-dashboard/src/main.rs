//! Binary entrypoint for `skynet-dashboard` (`http://127.0.0.1:8888`).

use skynet_dashboard::{
    handle_connection, resolve_dashboard_port, DashboardAppState, DashboardError,
    DEFAULT_BIND_HOST,
};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), DashboardError> {
    let state = DashboardAppState::init().await?;
    let _ticker = state.spawn_background_ticker();

    let port = resolve_dashboard_port();
    let bind_http_addr = format!("{DEFAULT_BIND_HOST}:{port}");
    let listener = TcpListener::bind(&bind_http_addr).await?;
    println!("==================================================================");
    println!("  SKYNET REAL-TIME DASHBOARD SERVER RUNNING AT http://{bind_http_addr}");
    println!("  60FPS RFC 6455 WEBSOCKET STREAM AT ws://{bind_http_addr}/ws/telemetry");
    println!("==================================================================");

    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(handle_connection(stream, state.clone()));
    }
}
