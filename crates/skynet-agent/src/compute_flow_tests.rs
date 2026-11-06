use super::*;
use rand::rngs::OsRng;

#[test]
fn test_end_to_end_compute_flow_happy_path() {
    let mut coordinator = ComputeFlowCoordinator::new();
    let mut csprng = OsRng;
    let client_key = SigningKey::generate(&mut csprng);
    let worker_key = SigningKey::generate(&mut csprng);

    let task_id = [11u8; 32];
    let task_input = CandleTaskInput {
        task_id,
        dim_in: 2,
        dim_hidden: 3,
        dim_out: 1,
        weights: vec![0.05f32; 13],
        input: vec![0.5f32, -0.5f32],
    };

    let bounty = NanoSkn::from_nano(1000);
    let bond = NanoSkn::from_nano(200);

    let manifest = ComputeFlowCoordinator::prepare_task(
        &client_key,
        PrepareTaskParams {
            task_id,
            opcode: 0x1101,
            required_tier: 1,
            bounty,
            min_bond: bond,
            auditor_bounty: NanoSkn::ZERO,
            required_auditors: 0,
            timeout_epoch: 100,
            input: &task_input,
        },
    )
    .expect("prepare succeeds");

    let bid = ComputeFlowCoordinator::create_bid(&worker_key, &manifest, 1, bond)
        .expect("bid succeeds");

    coordinator
        .assign_task(
            &client_key,
            AssignTaskParams {
                task_id,
                worker_pubkey: bid.worker_pubkey,
                bounty,
                bond,
                timeout_epoch: manifest.timeout_epoch,
            },
        )
        .expect("assign succeeds");

    let (output, claim) = coordinator
        .execute_and_claim(&worker_key, &task_input)
        .expect("execute succeeds");

    assert!(claim.execution_ticks <= manifest.max_ticks);

    let payout = coordinator
        .verify_and_settle(task_id, &claim, &output, 50)
        .expect("settlement succeeds");

    assert_eq!(payout, NanoSkn::from_nano(1200));
}

#[test]
fn test_end_to_end_compute_flow_timeout_and_slashing() {
    let mut coordinator = ComputeFlowCoordinator::new();
    let mut csprng = OsRng;
    let client_key = SigningKey::generate(&mut csprng);
    let worker_key = SigningKey::generate(&mut csprng);

    let task_id = [12u8; 32];
    let bounty = NanoSkn::from_nano(800);
    let bond = NanoSkn::from_nano(200);

    coordinator
        .assign_task(
            &client_key,
            AssignTaskParams {
                task_id,
                worker_pubkey: worker_key.verifying_key().to_bytes(),
                bounty,
                bond,
                timeout_epoch: 100,
            },
        )
        .expect("assign succeeds");

    let (refund, burn) = coordinator
        .handle_timeout_refund(task_id, &client_key.verifying_key().to_bytes(), 101)
        .expect("refund succeeds");

    assert_eq!(refund, NanoSkn::from_nano(900)); // 800 + 100
    assert_eq!(burn, NanoSkn::from_nano(100));
    assert_eq!(refund.0 + burn.0, 1000);
}

#[test]
fn test_create_bid_rejects_underqualified_or_underbonded() {
    let mut csprng = OsRng;
    let client_key = SigningKey::generate(&mut csprng);
    let worker_key = SigningKey::generate(&mut csprng);

    let task_input = CandleTaskInput {
        task_id: [13u8; 32],
        dim_in: 2,
        dim_hidden: 3,
        dim_out: 1,
        weights: vec![0.05f32; 13],
        input: vec![0.5f32, -0.5f32],
    };

    let manifest = ComputeFlowCoordinator::prepare_task(
        &client_key,
        PrepareTaskParams {
            task_id: [13u8; 32],
            opcode: 0x1101,
            required_tier: 2,
            bounty: NanoSkn::from_nano(1000),
            min_bond: NanoSkn::from_nano(300),
            auditor_bounty: NanoSkn::ZERO,
            required_auditors: 0,
            timeout_epoch: 100,
            input: &task_input,
        },
    )
    .unwrap();

    // Tier 1 worker rejected for Tier 2 task
    let err = ComputeFlowCoordinator::create_bid(&worker_key, &manifest, 1, NanoSkn::from_nano(300))
        .unwrap_err();
    assert!(matches!(
        err,
        ComputeFlowError::TierTooLow {
            actual: 1,
            required: 2
        }
    ));

    // Insufficient bond rejected
    let err = ComputeFlowCoordinator::create_bid(&worker_key, &manifest, 2, NanoSkn::from_nano(200))
        .unwrap_err();
    assert!(matches!(err, ComputeFlowError::InsufficientBond));
}
