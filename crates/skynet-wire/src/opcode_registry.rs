//! `opcode_registry`: Global catalog, category index, and metadata lookup for
//! 1,000+ standardized SkyNet compute task opcodes.

use crate::mcp_capsule::{HierarchicalOpcode, OpcodeDomain};
use serde::{Deserialize, Serialize};

pub use crate::opcode_domains::{DomainCategoryInfo, DOMAIN_CATEGORIES};

/// Standardized descriptor for a registered compute opcode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpcodeDescriptor {
    pub opcode: u16,
    pub domain: OpcodeDomain,
    pub task_code: u8,
    pub mcp_tool_name: String,
    pub title: &'static str,
    pub min_vram_mb: u32,
    pub min_ram_mb: u32,
    pub recommended_tier: u8,
    pub requires_tee: bool,
}

fn resolve_opcode_title(domain: OpcodeDomain, task_code: u8) -> &'static str {
    match (domain, task_code) {
        (OpcodeDomain::VisionAi, 0x01) => "Real-ESRGAN 4x Super Resolution",
        (OpcodeDomain::VisionAi, 0x02) => "FLUX.1 Schnell Text-to-Image",
        (OpcodeDomain::AudioSpeech, 0x01) => "OpenAI Whisper v3 Audio Transcription",
        (OpcodeDomain::AudioSpeech, 0x02) => "Demucs v4 4-Stem Audio Separation",
        (OpcodeDomain::LlmNlp, 0x01) => "Llama-3.3 8B Generative Inference",
        (OpcodeDomain::LlmNlp, 0x02) => "LoRA Adapter Fine-Tuning 70B",
        (OpcodeDomain::MicroTrainEvolution, 0x01) => "Opportunistic Multi-Step MLP Micro-Training",
        (OpcodeDomain::MicroTrainEvolution, 0x02) => "Opportunistic 16-bit LoRA Adapter Micro-Step",
        (OpcodeDomain::MicroTrainEvolution, 0x03) => "Guided Federated Plasmid Neuro-Evolution",
        (OpcodeDomain::Graphics3D, 0x01) => "Blender 4.2 Cycles Ray-Tracing Frame",
        (OpcodeDomain::BioScience, 0x01) => "AlphaFold3 Multi-Chain Protein Folding",
        (OpcodeDomain::HpcEngineering, 0x01) => "OpenFOAM Navier-Stokes Aerodynamic CFD",
        (OpcodeDomain::EarthSpace, 0x01) => "WRF Mesoscale Weather & Rainfall Forecast",
        (OpcodeDomain::DataEtl, 0x01) => "Polars CSV to Snappy Parquet Transformation",
        (OpcodeDomain::CryptographyZk, 0x01) => "Groth16 Zero-Knowledge SNARK Prover",
        (OpcodeDomain::FinanceQuant, 0x01) => "Monte Carlo 100M Path Exotic Option Pricing",
        (OpcodeDomain::RoboticsSwarm, 0x01) => "Fast-LIO 3D LiDAR Odometry & Mapping",
        (OpcodeDomain::MediaBroadcasting, 0x01) => "SVT-AV1 10-bit Video Transcoding Batch",
        (OpcodeDomain::EducationHeritage, 0x01) => "Ancient Han-Nom Document OCR & Translation",
        (OpcodeDomain::LegalLogistics, 0x01) => "Vehicle Routing Problem (VRP) Fleet Optimizer",
        _ => "Standardized Decentralized Compute Task",
    }
}

fn resolve_hardware_profile(domain: OpcodeDomain) -> (u32, u32, u8, bool) {
    match domain {
        OpcodeDomain::VisionAi | OpcodeDomain::Graphics3D => (8192, 16384, 2, false),
        OpcodeDomain::LlmNlp => (16384, 32768, 3, false),
        OpcodeDomain::MicroTrainEvolution => (2048, 8192, 1, false),
        OpcodeDomain::BioScience | OpcodeDomain::HpcEngineering => (0, 32768, 3, false),
        OpcodeDomain::CryptographyZk => (0, 65536, 3, false),
        OpcodeDomain::ConfidentialTee => (0, 16384, 2, true),
        _ => (0, 8192, 1, false),
    }
}

/// Resolve an opcode into its metadata descriptor and standard MCP URI.
pub fn lookup_opcode_descriptor(opcode: u16) -> OpcodeDescriptor {
    let parsed = HierarchicalOpcode::from_u16(opcode);
    let title = resolve_opcode_title(parsed.domain, parsed.task_code);
    let (min_vram_mb, min_ram_mb, recommended_tier, requires_tee) =
        resolve_hardware_profile(parsed.domain);

    OpcodeDescriptor {
        opcode,
        domain: parsed.domain,
        task_code: parsed.task_code,
        mcp_tool_name: parsed.canonical_mcp_tool_name().to_string(),
        title,
        min_vram_mb,
        min_ram_mb,
        recommended_tier,
        requires_tee,
    }
}

/// Calculate total capacity of standardized opcodes across all registered domains.
pub fn total_registered_capacity() -> usize {
    DOMAIN_CATEGORIES.iter().map(|d| d.capacity as usize).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_categories_exceed_1000_slots() {
        let total = total_registered_capacity();
        assert!(total >= 1000, "Capacity must be >= 1000, got {}", total);
    }

    #[test]
    fn test_lookup_opcode_descriptor_consistent() {
        let desc = lookup_opcode_descriptor(0x1001);
        assert_eq!(desc.domain, OpcodeDomain::VisionAi);
        assert_eq!(desc.task_code, 0x01);
        assert_eq!(desc.title, "Real-ESRGAN 4x Super Resolution");
        assert_eq!(desc.mcp_tool_name, "skynet://vision.upscale.real_esrgan");

        let media = lookup_opcode_descriptor(0xA001);
        assert_eq!(media.domain, OpcodeDomain::MediaBroadcasting);
        assert_eq!(media.title, "SVT-AV1 10-bit Video Transcoding Batch");
        assert_eq!(media.mcp_tool_name, "skynet://media.transcode.svt_av1");

        let edu = lookup_opcode_descriptor(0xB001);
        assert_eq!(edu.domain, OpcodeDomain::EducationHeritage);
        assert_eq!(edu.title, "Ancient Han-Nom Document OCR & Translation");
        assert_eq!(edu.mcp_tool_name, "skynet://edu.heritage.hannom_ocr");

        let legal = lookup_opcode_descriptor(0xC001);
        assert_eq!(legal.domain, OpcodeDomain::LegalLogistics);
        assert_eq!(legal.title, "Vehicle Routing Problem (VRP) Fleet Optimizer");
        assert_eq!(legal.mcp_tool_name, "skynet://legal.logistics.vrp_optimizer");
    }

    #[test]
    fn test_micro_train_evolution_domain_0x1300_registered() {
        let has_domain = DOMAIN_CATEGORIES
            .iter()
            .any(|c| c.base_code == 0x1300 && c.domain == OpcodeDomain::MicroTrainEvolution);
        assert!(has_domain);

        let mlp = lookup_opcode_descriptor(0x1301);
        assert_eq!(mlp.domain, OpcodeDomain::MicroTrainEvolution);
        assert_eq!(mlp.mcp_tool_name, "skynet://micro_train.mlp_step");

        let lora = lookup_opcode_descriptor(0x1302);
        assert_eq!(lora.domain, OpcodeDomain::MicroTrainEvolution);
        assert_eq!(lora.mcp_tool_name, "skynet://micro_train.lora_adapter");

        let plasmid = lookup_opcode_descriptor(0x1303);
        assert_eq!(plasmid.domain, OpcodeDomain::MicroTrainEvolution);
        assert_eq!(plasmid.mcp_tool_name, "skynet://evolution.plasmid_guided");
    }

    #[test]
    fn test_categories_base_codes_no_collision() {
        let mut bases = std::collections::HashSet::new();
        for cat in DOMAIN_CATEGORIES {
            assert!(
                bases.insert(cat.base_code),
                "Duplicate base code: {:#x}",
                cat.base_code
            );
        }
    }
}
