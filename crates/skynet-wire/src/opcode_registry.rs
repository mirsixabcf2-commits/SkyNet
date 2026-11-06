//! `opcode_registry`: Global catalog, category index, and metadata lookup for
//! 1,000+ standardized SkyNet compute task opcodes.

use crate::mcp_capsule::{HierarchicalOpcode, OpcodeDomain};
use serde::{Deserialize, Serialize};

/// High-level domain metadata category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainCategoryInfo {
    pub domain: OpcodeDomain,
    pub name: &'static str,
    pub description: &'static str,
    pub base_code: u16,
    pub capacity: u16,
}

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

/// Catalog of all 12 major compute domains spanning the 1,000+ opcode spectrum.
pub const DOMAIN_CATEGORIES: &[DomainCategoryInfo] = &[
    DomainCategoryInfo {
        domain: OpcodeDomain::VisionAi,
        name: "AI Computer Vision & Generative Media",
        description: "Super-resolution, diffusion, object detection, segmentation, OCR",
        base_code: 0x1000,
        capacity: 150,
    },
    DomainCategoryInfo {
        domain: OpcodeDomain::AudioSpeech,
        name: "Audio, Speech & Acoustic Intelligence",
        description: "Transcription, voice cloning, stem separation, sound event detection",
        base_code: 0x1100,
        capacity: 100,
    },
    DomainCategoryInfo {
        domain: OpcodeDomain::LlmNlp,
        name: "Large Language Models & NLP",
        description: "Inference, LoRA fine-tuning, embeddings, reasoning, agentic execution",
        base_code: 0x1200,
        capacity: 100,
    },
    DomainCategoryInfo {
        domain: OpcodeDomain::Graphics3D,
        name: "3D Graphics, Spatial Computing & VFX",
        description: "Blender Cycles, Gaussian Splatting, NeRF, fluids, hair, particle dynamics",
        base_code: 0x2000,
        capacity: 100,
    },
    DomainCategoryInfo {
        domain: OpcodeDomain::BioScience,
        name: "Biomedical, Genomics & Life Sciences",
        description: "AlphaFold protein folding, AutoDock molecular docking, single-cell RNA",
        base_code: 0x3000,
        capacity: 100,
    },
    DomainCategoryInfo {
        domain: OpcodeDomain::HpcEngineering,
        name: "Engineering Simulation & CAE/CFD",
        description: "OpenFOAM CFD, CalculiX FEA, SPICE circuits, thermal dissipation",
        base_code: 0x4000,
        capacity: 100,
    },
    DomainCategoryInfo {
        domain: OpcodeDomain::EarthSpace,
        name: "Earth, Climate, Geophysics & Aerospace",
        description: "WRF weather modeling, 3D seismic inversion, SAR satellite, N-body",
        base_code: 0x5000,
        capacity: 80,
    },
    DomainCategoryInfo {
        domain: OpcodeDomain::DataEtl,
        name: "Big Data Pipelines, Vector Search & Graphs",
        description: "Polars/DuckDB Parquet, MinHash deduplication, HNSW index build",
        base_code: 0x6000,
        capacity: 80,
    },
    DomainCategoryInfo {
        domain: OpcodeDomain::CryptographyZk,
        name: "Cryptography, Zero-Knowledge & Privacy",
        description: "Groth16/Plonky2 ZK-provers, FHE evaluation, MPC, smart contract fuzzing",
        base_code: 0x7000,
        capacity: 80,
    },
    DomainCategoryInfo {
        domain: OpcodeDomain::FinanceQuant,
        name: "Quantitative Finance, Risk & DeFi",
        description: "Monte Carlo option pricing, CoW-Swap batch solvers, AML transaction graphs",
        base_code: 0x8000,
        capacity: 80,
    },
    DomainCategoryInfo {
        domain: OpcodeDomain::RoboticsSwarm,
        name: "Autonomous Robotics, Drones & Smart Grids",
        description: "LiDAR 3D SLAM, drone swarm 4D trajectories, VPP load balancing",
        base_code: 0x9000,
        capacity: 80,
    },
    DomainCategoryInfo {
        domain: OpcodeDomain::MediaBroadcasting,
        name: "Media & Broadcasting Codecs",
        description: "AV1/HEVC video transcoding, multi-track audio mastering, spatial audio",
        base_code: 0xA000,
        capacity: 80,
    },
    DomainCategoryInfo {
        domain: OpcodeDomain::EducationHeritage,
        name: "Education, Translation & Cultural Heritage",
        description: "Ancient Han-Nom OCR, rare dialect translation, heritage digitizing",
        base_code: 0xB000,
        capacity: 60,
    },
    DomainCategoryInfo {
        domain: OpcodeDomain::LegalLogistics,
        name: "Legal, Logistics & Autonomous Supply Chain",
        description: "Smart contract audit, VRP fleet optimization, customs bill verification",
        base_code: 0xC000,
        capacity: 60,
    },
];

fn resolve_opcode_title(domain: OpcodeDomain, task_code: u8) -> &'static str {
    match (domain, task_code) {
        (OpcodeDomain::VisionAi, 0x01) => "Real-ESRGAN 4x Super Resolution",
        (OpcodeDomain::VisionAi, 0x02) => "FLUX.1 Schnell Text-to-Image",
        (OpcodeDomain::AudioSpeech, 0x01) => "OpenAI Whisper v3 Audio Transcription",
        (OpcodeDomain::AudioSpeech, 0x02) => "Demucs v4 4-Stem Audio Separation",
        (OpcodeDomain::LlmNlp, 0x01) => "Llama-3.3 8B Generative Inference",
        (OpcodeDomain::LlmNlp, 0x02) => "LoRA Adapter Fine-Tuning 70B",
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
    fn test_categories_base_codes_no_collision() {
        let mut bases = std::collections::HashSet::new();
        for cat in DOMAIN_CATEGORIES {
            assert!(bases.insert(cat.base_code), "Duplicate base code: {:#x}", cat.base_code);
        }
    }
}
