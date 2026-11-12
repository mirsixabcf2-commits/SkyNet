//! Integration test verifying 100% state persistence roundtrip across node restarts.

use skynet_cell::HardwareTier;
use skynet_dna::DEFAULT_INPUT_DIM;
use skynet_ledger::NanoSkn;
use skynet_node::{NodeConfig, NodeRuntime, StorageEngine};

struct SavedNodeSnapshot {
    pubkey: [u8; 32],
    cell_ids: Vec<u64>,
    balance: NanoSkn,
    blood: NanoSkn,
}

async fn run_initial_ticks_and_save(cfg: NodeConfig) -> SavedNodeSnapshot {
    let mut runtime = NodeRuntime::init(cfg).await.expect("node init must succeed");
    let stim = [0.25f32; DEFAULT_INPUT_DIM];
    let target = [0.5f32; 4];
    for _ in 0..5 {
        let _ = runtime.agent.run_biological_tick(&stim, &target).unwrap();
    }
    runtime.storage.save_all(&runtime.agent).expect("save_all must succeed");

    let snap = SavedNodeSnapshot {
        pubkey: runtime.agent.pubkey,
        cell_ids: runtime.agent.cells.iter().map(|c| c.cell_id).collect(),
        balance: runtime.agent.ledger.balance_of(&runtime.agent.pubkey),
        blood: runtime.agent.bank.blood_pool,
    };
    assert!(!snap.cell_ids.is_empty());
    assert!(snap.balance.0 > 0);
    snap
}

#[tokio::test]
async fn test_persistence_roundtrip_preserves_cells_and_skn() {
    let tmp_dir = std::env::temp_dir().join(format!("skynet_roundtrip_{}", rand::random::<u64>()));
    let cfg = NodeConfig {
        port: 19888,
        data_dir: tmp_dir.clone(),
        name: "Roundtrip-Agent".to_string(),
        tier: HardwareTier::StandardNode,
        seed_peers: Vec::new(),
        owner_wallet: None,
        genesis_path: None,
        tick_interval_ms: 1000,
        auto_flush_interval_sec: 30,
    };

    let snap = run_initial_ticks_and_save(cfg.clone()).await;

    let storage = StorageEngine::new(&tmp_dir).expect("storage init must succeed");
    assert!(storage.has_existing_state());

    let runtime2 = NodeRuntime::init(cfg).await.expect("node restart must succeed");
    assert_eq!(runtime2.agent.pubkey, snap.pubkey);
    assert_eq!(runtime2.agent.cells.len(), snap.cell_ids.len());
    assert_eq!(runtime2.agent.ledger.balance_of(&runtime2.agent.pubkey), snap.balance);
    assert_eq!(runtime2.agent.bank.blood_pool, snap.blood);

    let restored_cell_ids: Vec<u64> = runtime2.agent.cells.iter().map(|c| c.cell_id).collect();
    assert_eq!(restored_cell_ids, snap.cell_ids);
    for cell in &runtime2.agent.cells {
        assert!(cell.energy.0 > 0);
    }

    let _ = std::fs::remove_dir_all(tmp_dir);
}
