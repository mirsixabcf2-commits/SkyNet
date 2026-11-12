//! Asynchronous Tokio event loop, P2P rendezvous/epidermal filtering, biological pulse,
//! and graceful shutdown for `skynet-node`.

use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use skynet_agent::{AgentError, AgentNode, SnowballParams, SEED_KEEPALIVE_INTERVAL_SEC};
use skynet_dna::DEFAULT_INPUT_DIM;
use skynet_ledger::AgentMetabolicBank;
use skynet_wire::WireError;
use thiserror::Error;
use tokio::net::UdpSocket;
use tokio::sync::Notify;

use crate::config::NodeConfig;
use crate::genesis_io::{load_genesis_file, GenesisIoError};
use crate::identity::{IdentityError, NodeIdentity};
use crate::runtime_p2p::{bootstrap_node_ledger, RuntimeP2pDriver, LAN_BEACON_INTERVAL_MS};
use crate::storage::{StorageEngine, StorageError};

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("identity error: {0}")]
    Identity(#[from] IdentityError),
    #[error("storage error: {0}")]
    Storage(#[from] StorageError),
    #[error("genesis I/O error: {0}")]
    Genesis(#[from] GenesisIoError),
    #[error("agent error: {0}")]
    Agent(#[from] AgentError),
    #[error("wire error: {0}")]
    Wire(#[from] WireError),
    #[error("socket I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Core asynchronous daemon runtime executing the multicellular AI agent & P2P driver.
pub struct NodeRuntime {
    pub agent: AgentNode,
    pub config: NodeConfig,
    pub storage: StorageEngine,
    pub socket: Arc<UdpSocket>,
    pub p2p: RuntimeP2pDriver,
    is_running: Arc<AtomicBool>,
    shutdown_notify: Arc<Notify>,
}

type BootstrappedAgent = (AgentNode, Option<[u8; 32]>);

impl NodeRuntime {
    /// Initializes or restores the node from persisted storage or `--genesis <PATH>`.
    pub async fn init(mut config: NodeConfig) -> Result<Self, RuntimeError> {
        let storage = StorageEngine::new(&config.data_dir)?;
        let key_path = config.data_dir.join("node_identity.key");
        let identity = NodeIdentity::load_or_generate(&key_path)?;

        let (mut agent, genesis_hash) = Self::spawn_or_restore(&config, &identity, &storage)?;
        let bind_addr = SocketAddr::from(([0, 0, 0, 0], config.port));
        let socket = UdpSocket::bind(bind_addr).await?;
        let local_port = socket.local_addr()?.port();
        config.port = local_port;
        let p2p = RuntimeP2pDriver::new(genesis_hash, local_port, &mut agent);

        Ok(Self {
            agent,
            config,
            storage,
            socket: Arc::new(socket),
            p2p,
            is_running: Arc::new(AtomicBool::new(true)),
            shutdown_notify: Arc::new(Notify::new()),
        })
    }

    pub fn shutdown(&self) {
        self.is_running.store(false, Ordering::Relaxed);
        self.shutdown_notify.notify_waiters();
    }

    fn spawn_or_restore(
        config: &NodeConfig,
        identity: &NodeIdentity,
        storage: &StorageEngine,
    ) -> Result<BootstrappedAgent, RuntimeError> {
        if storage.has_existing_state() {
            if let Some(restored) = Self::try_restore_existing(config, identity, storage)? {
                return Ok(restored);
            }
        }
        let (ledger, genesis_hash) =
            bootstrap_node_ledger(config.genesis_path.as_deref(), identity.pubkey)?;
        let agent = AgentNode::from_signing_key(
            &config.name,
            1337,
            identity.signing_key.clone(),
            config.tier,
            ledger,
            SnowballParams::default(),
        )?;
        Ok((agent, genesis_hash))
    }

    fn try_restore_existing(
        config: &NodeConfig,
        identity: &NodeIdentity,
        storage: &StorageEngine,
    ) -> Result<Option<BootstrappedAgent>, RuntimeError> {
        let (Some(ledger), Some(org)) = (storage.load_ledger()?, storage.load_organism()?) else {
            return Ok(None);
        };
        let genesis_hash = match &config.genesis_path {
            Some(p) => Some(load_genesis_file(p)?.1),
            None => None,
        };
        let opts = skynet_agent::RestoreOrganismOptions {
            name: config.name.clone(),
            seed: 1337,
            signing_key: identity.signing_key.clone(),
            tier: config.tier,
            ledger,
            cells: org.cells,
            bank: AgentMetabolicBank::new(org.blood_pool),
            snowball_params: SnowballParams::default(),
        };
        let mut agent = AgentNode::restore_organism(opts)?;
        agent.total_voucher_nano_earned = org.total_voucher_nano_earned;
        for p in storage.load_peers()? {
            agent.peer_table.observe_peer(p);
        }
        Ok(Some((agent, genesis_hash)))
    }

    /// Runs the daemon loop until `shutdown()` or SIGINT/Ctrl+C is received.
    pub async fn run(&mut self) -> Result<(), RuntimeError> {
        let _ = self.p2p.send_seed_keepalives(&self.socket, &mut self.agent, &self.config.seed_peers).await?;
        let _ = self.p2p.broadcast_lan_beacon(&self.socket, &mut self.agent).await?;

        let mut buf = [0u8; 2048];
        let mut tick_timer = tokio::time::interval(Duration::from_millis(self.config.tick_interval_ms));
        let mut flush_timer = tokio::time::interval(Duration::from_secs(self.config.auto_flush_interval_sec));
        let mut seed_timer = tokio::time::interval(Duration::from_secs(SEED_KEEPALIVE_INTERVAL_SEC));
        let mut lan_timer = tokio::time::interval(Duration::from_millis(LAN_BEACON_INTERVAL_MS));

        while self.is_running.load(Ordering::Relaxed) {
            tokio::select! {
                recv_res = self.socket.recv_from(&mut buf) => {
                    let (len, src_addr) = recv_res?;
                    let _ = self.p2p.process_datagram(&self.socket, &mut self.agent, (&buf[..len], src_addr)).await?;
                }
                _ = tick_timer.tick() => self.perform_biological_pulse()?,
                _ = seed_timer.tick() => {
                    let _ = self.p2p.send_seed_keepalives(&self.socket, &mut self.agent, &self.config.seed_peers).await?;
                }
                _ = lan_timer.tick() => {
                    let _ = self.p2p.broadcast_lan_beacon(&self.socket, &mut self.agent).await?;
                }
                _ = flush_timer.tick() => self.storage.save_all(&self.agent)?,
                _ = self.shutdown_notify.notified() => break,
                _ = tokio::signal::ctrl_c() => break,
            }
        }
        self.graceful_shutdown()
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
        Ok(())
    }
}
