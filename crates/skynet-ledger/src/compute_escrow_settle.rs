//! `compute_escrow_settle`: Settlement calculations and bond reconciliation for compute task escrows.

use std::collections::HashSet;

use crate::compute_escrow_types::{AuditorDeposit, EscrowError, EscrowPayoutBreakdown};
use crate::NanoSkn;

/// Calculates itemized audited settlement with deduplication and bond protection.
pub fn calculate_audited_payout(
    bounty: NanoSkn,
    bond: NanoSkn,
    auditor_pool: NanoSkn,
    registered: &[AuditorDeposit],
    attesting: &[[u8; 32]],
) -> Result<EscrowPayoutBreakdown, EscrowError> {
    let mut seen = HashSet::with_capacity(attesting.len());
    for &pk in attesting {
        if !seen.insert(pk) {
            return Err(EscrowError::DuplicateAuditor);
        }
    }

    let worker_payout = bounty.checked_add(bond)?;
    let num_attesting = attesting.len() as u128;
    let reward_share = NanoSkn(auditor_pool.0.checked_div(num_attesting).unwrap_or(0));
    let mut total_auditor = NanoSkn::ZERO;
    let mut auditor_payouts = Vec::with_capacity(registered.len());

    for &pubkey in attesting {
        let deposit = registered
            .iter()
            .find(|d| d.auditor_pubkey == pubkey)
            .ok_or(EscrowError::UnregisteredAuditor)?;
        let payout = deposit.bond.checked_add(reward_share)?;
        total_auditor = total_auditor.checked_add(payout)?;
        auditor_payouts.push((pubkey, payout));
    }

    // Return bonds to registered auditors who did not participate in attestation
    for dep in registered {
        if !seen.contains(&dep.auditor_pubkey) {
            total_auditor = total_auditor.checked_add(dep.bond)?;
            auditor_payouts.push((dep.auditor_pubkey, dep.bond));
        }
    }

    let total_disbursed = worker_payout.checked_add(total_auditor)?;
    Ok(EscrowPayoutBreakdown {
        worker_payout,
        auditor_payouts,
        total_disbursed,
    })
}
