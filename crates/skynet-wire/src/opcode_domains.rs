//! `opcode_domains`: Domain category catalog for SkyNet's 1,000+ standardized compute opcodes.

use crate::mcp_capsule::OpcodeDomain;
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

/// Catalog of standardized compute domains spanning the 1,000+ opcode spectrum.
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
        domain: OpcodeDomain::MicroTrainEvolution,
        name: "Opportunistic Edge Micro-Training & Neuro-Evolution",
        description: "Multi-step MLP micro-batches, LoRA adapters, federated Plasmid evolution",
        base_code: 0x1300,
        capacity: 80,
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
