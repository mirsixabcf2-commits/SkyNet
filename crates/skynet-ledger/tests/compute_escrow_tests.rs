use skynet_ledger::compute_escrow::ComputeEscrowVault;
use skynet_ledger::{EscrowError, EscrowSplitConfig, NanoSkn};

#[test]
fn test_compute_escrow_happy_path() {
    let mut vault = ComputeEscrowVault::new();
    let task_id = [7u8; 32];
    let client = [1u8; 32];
    let worker = [2u8; 32];
    let bounty = NanoSkn::from_nano(1000);
    let bond = NanoSkn::from_nano(300);

    vault
        .lock_task_simple(task_id, client, worker, bounty, bond, 50)
        .expect("lock succeeds");

    let payout = vault
        .complete_task(task_id, &worker, [9u8; 32], 40)
        .expect("completion succeeds");
    assert_eq!(payout, NanoSkn::from_nano(1300));
}

#[test]
fn test_compute_escrow_delegated_auditor_quorum() {
    let mut vault = ComputeEscrowVault::new();
    let task_id = [15u8; 32];
    let client = [1u8; 32];
    let worker = [2u8; 32];
    let aud1 = [3u8; 32];
    let aud2 = [4u8; 32];

    let split = EscrowSplitConfig {
        worker_bounty: NanoSkn::from_nano(850),
        auditor_pool_bounty: NanoSkn::from_nano(150),
        required_auditors: 2,
    };
    let worker_bond = NanoSkn::from_nano(200);

    vault
        .lock_task_with_split(task_id, client, worker, split, worker_bond, 100)
        .expect("lock split succeeds");

    vault
        .register_auditor(task_id, aud1, NanoSkn::from_nano(50))
        .expect("aud1 registered");
    vault
        .register_auditor(task_id, aud2, NanoSkn::from_nano(50))
        .expect("aud2 registered");

    let attesting = vec![aud1, aud2];
    let breakdown = vault
        .complete_task_with_auditors(task_id, &worker, &attesting, [99u8; 32], 50)
        .expect("audited settlement succeeds");

    assert_eq!(breakdown.worker_payout, NanoSkn::from_nano(1050));
    assert_eq!(breakdown.auditor_payouts.len(), 2);
    assert_eq!(breakdown.auditor_payouts[0].1, NanoSkn::from_nano(125));
    assert_eq!(breakdown.auditor_payouts[1].1, NanoSkn::from_nano(125));
    assert_eq!(breakdown.total_disbursed, NanoSkn::from_nano(1300));
}

#[test]
fn test_compute_escrow_timeout_refund_with_slashing() {
    let mut vault = ComputeEscrowVault::new();
    let task_id = [8u8; 32];
    let client = [1u8; 32];
    let worker = [2u8; 32];
    let bounty = NanoSkn::from_nano(1000);
    let bond = NanoSkn::from_nano(300);

    vault
        .lock_task_simple(task_id, client, worker, bounty, bond, 50)
        .expect("lock succeeds");

    let (refund, burn) = vault
        .refund_task(task_id, &client, 51)
        .expect("refund succeeds");
    assert_eq!(refund, NanoSkn::from_nano(1150));
    assert_eq!(burn, NanoSkn::from_nano(150));
    assert_eq!(refund.0 + burn.0, 1300);
}

#[test]
fn test_duplicate_auditor_rejected() {
    let mut vault = ComputeEscrowVault::new();
    let task_id = [20u8; 32];
    let client = [1u8; 32];
    let worker = [2u8; 32];
    let aud1 = [3u8; 32];

    let split = EscrowSplitConfig {
        worker_bounty: NanoSkn::from_nano(800),
        auditor_pool_bounty: NanoSkn::from_nano(200),
        required_auditors: 2,
    };
    vault
        .lock_task_with_split(task_id, client, worker, split, NanoSkn::from_nano(100), 100)
        .unwrap();
    vault
        .register_auditor(task_id, aud1, NanoSkn::from_nano(50))
        .unwrap();

    let duplicate_attesting = vec![aud1, aud1];
    let err = vault
        .complete_task_with_auditors(task_id, &worker, &duplicate_attesting, [0u8; 32], 50)
        .unwrap_err();
    assert_eq!(err, EscrowError::DuplicateAuditor);
}

#[test]
fn test_self_auditing_forbidden() {
    let mut vault = ComputeEscrowVault::new();
    let task_id = [21u8; 32];
    let client = [1u8; 32];
    let worker = [2u8; 32];

    vault
        .lock_task_simple(
            task_id,
            client,
            worker,
            NanoSkn::from_nano(500),
            NanoSkn::from_nano(100),
            100,
        )
        .unwrap();

    // Worker attempts to register as auditor
    let err = vault
        .register_auditor(task_id, worker, NanoSkn::from_nano(50))
        .unwrap_err();
    assert_eq!(err, EscrowError::SelfAuditingForbidden);

    // Client attempts to register as auditor
    let err = vault
        .register_auditor(task_id, client, NanoSkn::from_nano(50))
        .unwrap_err();
    assert_eq!(err, EscrowError::SelfAuditingForbidden);
}

#[test]
fn test_auditor_bond_refunded_on_worker_timeout() {
    let mut vault = ComputeEscrowVault::new();
    let task_id = [22u8; 32];
    let client = [1u8; 32];
    let worker = [2u8; 32];
    let aud1 = [3u8; 32];

    let split = EscrowSplitConfig {
        worker_bounty: NanoSkn::from_nano(800),
        auditor_pool_bounty: NanoSkn::from_nano(200),
        required_auditors: 1,
    };
    vault
        .lock_task_with_split(task_id, client, worker, split, NanoSkn::from_nano(200), 50)
        .unwrap();
    vault
        .register_auditor(task_id, aud1, NanoSkn::from_nano(50))
        .unwrap();

    // Worker times out at epoch 51
    let (refund, burn) = vault.refund_task(task_id, &client, 51).unwrap();
    // Client receives total bounty (800 + 200) + 50% of worker bond (100) = 1100
    assert_eq!(refund, NanoSkn::from_nano(1100));
    // 50% of worker bond is burned = 100
    assert_eq!(burn, NanoSkn::from_nano(100));
}
