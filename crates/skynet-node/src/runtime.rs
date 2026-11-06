//! Asynchronous Tokio event loop, UDP impulse handling, biological pulse, and graceful shutdown.

use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use skynet_agent::{AgentError, AgentNode, SnowballParams};
use skynet_dna::DEFAULT_INPUT_DIM;
use skynet_ledger::{AgentMetabolicBank, NanoSkn, TxOutput, UtxoDagLedger};
use thiserror::Error;
use tokio::net::UdpSocket;

use crate::config::NodeConfig;
use crate::identity::{IdentityError, NodeIdentity};
use crate::storage::{StorageEngine, StorageError};

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("identity error: {0}")]
    Identity(#[from] IdentityError),
    #[error("storage error: {0}")]
    Storage(#[from] StorageError),
    #[error("agent error: {0}")]
    Agent(#[from] AgentError),
    #[error("socket I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// The core asynchronous daemon runtime executing the multicellular AI agent.
pub struct NodeRuntime {
    pub agent: AgentNode,
    pub config: NodeConfig,
    pub storage: StorageEngine,
    pub socket: Arc<UdpSocket>,
    is_running: Arc<AtomicBool>,
}

impl NodeRuntime {
    /// Initializes or restores the node from persisted storage.
    pub async fn init(config: NodeConfig) -> Result<Self, RuntimeError> {
        let storage = StorageEngine::new(&config.data_dir)?;
        let key_path = config.data_dir.join("node_identity.key");
        let identity = NodeIdentity::load_or_generate(&key_path)?;

        let agent = Self::spawn_or_restore(&config, &identity, &storage)?;
        let bind_addr = SocketAddr::from(([0, 0, 0, 0], config.port));
        let socket = Arc::new(UdpSocket::bind(bind_addr).await?);

        Ok(Self {
            agent,
            config,
            storage,
            socket,
            is_running: Arc::new(AtomicBool::new(true)),
        })
    }

    fn spawn_or_restore(
        config: &NodeConfig,
        identity: &NodeIdentity,
        storage: &StorageEngine,
    ) -> Result<AgentNode, RuntimeError> {
        let snowball = SnowballParams::default();
        if storage.has_existing_state() {
            if let (Some(ledger), Some(org)) = (storage.load_ledger()?, storage.load_organism()?) {
                let bank = AgentMetabolicBank::new(org.blood_pool);
                let opts = skynet_agent::RestoreOrganismOptions {
                    name: config.name.clone(),
                    seed: 1337,
                    signing_key: identity.signing_key.clone(),
                    tier: config.tier,
                    ledger,
                    cells: org.cells,
                    bank,
                    snowball_params: snowball,
                };
                let mut agent = AgentNode::restore_organism(opts)?;
                agent.total_voucher_nano_earned = org.total_voucher_nano_earned;
                let peers = storage.load_peers()?;
                for p in peers {
                    agent.peer_table.observe_peer(p);
                }
                return Ok(agent);
            }
        }
        let init_alloc = NanoSkn::from_skn(100_000).unwrap_or(NanoSkn::ZERO);
        let (ledger, _) = UtxoDagLedger::from_genesis(vec![
            TxOutput::direct(identity.pubkey, init_alloc)
        ]).map_err(AgentError::Ledger)?;

        let agent = AgentNode::from_signing_key(
            &config.name,
            1337,
            identity.signing_key.clone(),
            config.tier,
            ledger,
            snowball,
        )?;
        Ok(agent)
    }

    /// Runs the daemon loop until SIGINT/Ctrl+C is received.
    pub async fn run(&mut self) -> Result<(), RuntimeError> {
        println!("  [SkyNet-Node] Daemon active on UDP port {}", self.config.port);
        println!("  [SkyNet-Node] Public Key: {}", self.agent.pubkey.iter().map(|b| format!("{:02x}", b)).collect::<String>());

        let mut buf = [0u8; 2048];
        let mut tick_timer = tokio::time::interval(Duration::from_millis(self.config.tick_interval_ms));
        let mut flush_timer = tokio::time::interval(Duration::from_secs(self.config.auto_flush_interval_sec));

        while self.is_running.load(Ordering::Relaxed) {
            tokio::select! {
                recv_res = self.socket.recv_from(&mut buf) => {
                    let (len, src_addr) = recv_res?;
                    self.handle_incoming_packet(&buf[..len], src_addr).await?;
                }
                _ = tick_timer.tick() => {
                    self.perform_biological_pulse()?;
                }
                _ = flush_timer.tick() => {
                    self.storage.save_all(&self.agent)?;
                }
                _ = tokio::signal::ctrl_c() => {
                    println!("\n  [SkyNet-Node] SIGINT received. Initiating graceful shutdown...");
                    break;
                }
            }
        }
        self.graceful_shutdown()
    }

    async fn handle_incoming_packet(
        &mut self,
        packet_bytes: &[u8],
        src_addr: SocketAddr,
    ) -> Result<(), RuntimeError> {
        if let Ok(Some(reply)) = self.agent.handle_incoming_datagram(packet_bytes) {
            if let Ok(bytes) = reply.encode_datagram() {
                let _ = self.socket.send_to(&bytes, src_addr).await;
            }
        }
        Ok(())
    }

    fn perform_biological_pulse(&mut self) -> Result<(), RuntimeError> {
        let stimulus = [0.1f32; DEFAULT_INPUT_DIM];
        let target = [0.2f32; 4];
        let _ = self.agent.run_biological_tick(&stimulus, &target)?;
        let _ = self.agent.finalize_snowball_pulse()?;
        Ok(())
    }

    fn graceful_shutdown(&mut self) -> Result<(), RuntimeError> {
        self.is_running.store(false, Ordering::Relaxed);
        self.storage.save_all(&self.agent)?;
        println!("  [SkyNet-Node] State flushed to disk cleanly. Shutdown complete.");
        Ok(())
    }
}
