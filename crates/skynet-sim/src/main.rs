//! `skynet-sim`: Live multi-Agent Chaos UDP simulation entrypoint verifying
//! both classic multicellular consensus and opportunistic edge micro-training (`006`).

mod classic_sim;
mod opportunistic_sim;
mod sim_metrics;

use sim_metrics::{
    print_classic_report, print_opportunistic_report, verify_classic_invariants,
    verify_opportunistic_invariants, SimError,
};

#[tokio::main]
async fn main() -> Result<(), SimError> {
    println!("==========================================================================");
    println!("  SKYNET BIO-CELLULAR AI BLOCKCHAIN — CHAOS UDP ECOSYSTEM SIMULATION");
    println!("==========================================================================\n");

    let classic = classic_sim::run_classic_simulation().await?;
    verify_classic_invariants(&classic)?;
    print_classic_report(&classic);

    let opportunistic = opportunistic_sim::run_opportunistic_simulation().await?;
    verify_opportunistic_invariants(&opportunistic)?;
    print_opportunistic_report(&opportunistic);

    println!("    => KIEM TOAN TOAN HE THONG: HOP LE TUYET DOI (0 nanoSKN sai so!)\n");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn chaos_simulations_preserve_all_invariants() {
        let classic = classic_sim::run_classic_simulation().await.expect("classic sim");
        verify_classic_invariants(&classic).expect("classic invariants");

        let opp = opportunistic_sim::run_opportunistic_simulation().await.expect("opportunistic sim");
        verify_opportunistic_invariants(&opp).expect("opportunistic invariants");
    }
}
