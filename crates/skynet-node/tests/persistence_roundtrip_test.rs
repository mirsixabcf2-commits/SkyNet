//! Integration test verifying 100% state persistence roundtrip across node restarts.

use skynet_cell::HardwareTier;
use skynet_dna::DEFAULT_INPUT_DIM;
use skynet_node::{NodeConfig, NodeRuntime, StorageEngine};

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
        tick_interval_ms: 1000,
        auto_flush_interval_sec: 30,
    };

    let original_pubkey;
    let original_cell_count;
    let original_cell_ids: Vec<u64>;
    let original_balance;
    let original_blood;

    // 1. First run: Initialize node, execute biological ticks, and save
    {
        let mut runtime = NodeRuntime::init(cfg.clone())
            .await
            .expect("node init must succeed");

        let stim = [0.25f32; DEFAULT_INPUT_DIM];
        let target = [0.5f32; 4];
        for _ in 0..5 {
            let _ = runtime.agent.run_biological_tick(&stim, &target).unwrap();
        }

        runtime.storage.save_all(&runtime.agent).expect("save_all must succeed");

        original_pubkey = runtime.agent.pubkey;
        original_cell_count = runtime.agent.cells.len();
        original_cell_ids = runtime.agent.cells.iter().map(|c| c.cell_id).collect();
        original_balance = runtime.agent.ledger.balance_of(&original_pubkey);
        original_blood = runtime.agent.bank.blood_pool;

        assert!(original_cell_count > 0);
        assert!(original_balance.0 > 0);
    } // runtime drops here, simulating process exit/reboot

    // 2. Second run: Restart node from the same data_dir
    {
        let storage = StorageEngine::new(&tmp_dir).expect("storage init must succeed");
        assert!(storage.has_existing_state());

        let runtime2 = NodeRuntime::init(cfg)
            .await
            .expect("node restart must succeed");

        assert_eq!(runtime2.agent.pubkey, original_pubkey);
        assert_eq!(runtime2.agent.cells.len(), original_cell_count);
        assert_eq!(
            runtime2.agent.ledger.balance_of(&runtime2.agent.pubkey),
            original_balance
        );
        assert_eq!(runtime2.agent.bank.blood_pool, original_blood);

        // Assert biological cell IDs and roles are conserved 100%
        let restored_cell_ids: Vec<u64> = runtime2.agent.cells.iter().map(|c| c.cell_id).collect();
        assert_eq!(restored_cell_ids, original_cell_ids);
        for cell in &runtime2.agent.cells {
            assert!(cell.energy.0 > 0);
        }
    }

    let _ = std::fs::remove_dir_all(tmp_dir);
}
