//! `skynet-dashboard`: Real-time HTTP + RFC 6455 `60fps` WebSocket telemetry server
//! driving 8 live Multicellular `AgentNode`s and the Worker Earnings & Opportunistic Studio.

pub mod dto;
pub mod http_router;
pub mod live_world;
pub mod preprocess_api;
pub mod worker_studio_api;
pub mod world_actions;
pub mod ws_rfc6455;

pub use dto::{
    AgentDto, DashboardSnapshot, EfficiencyDto, HumanDto, ImpulseTrace, LiveNodeStatusDto,
    RolesDto, SupplyDto, WorkerStudioTelemetryDto,
};
pub use http_router::{
    bind_and_serve, handle_connection, resolve_dashboard_port, DashboardAppState,
    DEFAULT_BIND_HOST, DEFAULT_DASHBOARD_PORT,
};
pub use live_world::{DashboardError, LiveWorld};
pub use worker_studio_api::{
    load_worker_studio_state_file, StudioAction, WorkerStudioController, WORKER_STUDIO_STATE_FILE,
};
pub use world_actions::ControlCmd;
pub use ws_rfc6455::{
    build_ws_upgrade_response, compute_ws_accept_key, decode_ws_server_frame, encode_ws_text_frame,
    extract_ws_upgrade_key, stream_ws_telemetry_60fps, WS_FRAME_INTERVAL_MS,
};
