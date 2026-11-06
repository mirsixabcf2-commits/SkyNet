//! `mcp_capsule`: Model Context Protocol (MCP) standardized tool abstractions,
//! hierarchical opcodes, and zero-touch worker capsule manifests.

use serde::{Deserialize, Serialize};

/// High-level functional domain classified by the high-order byte of a 16-bit opcode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum OpcodeDomain {
    /// Deep learning vision, super-resolution, detection, segmentation (0x10)
    VisionAi = 0x10,
    /// Audio, speech recognition, vocal stem separation, denoising (0x11)
    AudioSpeech = 0x11,
    /// Large language models, text generation, LoRA fine-tuning (0x12)
    LlmNlp = 0x12,
    /// 3D graphics rendering, Gaussian Splatting, NeRF, ray-tracing (0x20)
    Graphics3D = 0x20,
    /// Video transcoding, codec conversion, media pipelines (0x21)
    MediaTranscode = 0x21,
    /// Biology, molecular dynamics, protein folding, drug docking (0x30)
    BioScience = 0x30,
    /// Engineering simulation, OpenFOAM CFD, finite element FEA (0x40 / legacy 0x31)
    HpcEngineering = 0x40,
    /// Quantitative finance, Monte Carlo, astrophysics N-Body (legacy 0x32)
    QuantPhysics = 0x32,
    /// Earth, Climate, Geophysics, Weather & Aerospace (0x50)
    EarthSpace = 0x50,
    /// Big data pipelines, ETL, Parquet conversion, deduplication (0x60)
    DataEtl = 0x60,
    /// Vector indexing, HNSW graph building, knowledge graphs (0x61)
    KnowledgeVector = 0x61,
    /// Cryptography, ZK-SNARK, ZK-STARK, FHE, MPC (0x70)
    CryptographyZk = 0x70,
    /// Confidential computing, Hardware TEE (SGX / SEV-SNP) (0x71)
    ConfidentialTee = 0x71,
    /// Quantitative Finance, Risk, AML & DeFi Solvers (0x80)
    FinanceQuant = 0x80,
    /// Autonomous robotics, drone swarm trajectory, LiDAR SLAM (0x90)
    RoboticsSwarm = 0x90,
    /// Media broadcasting, codecs, audio mastering (0xA0)
    MediaBroadcasting = 0xA0,
    /// Education, specialized translation, ancient heritage (0xB0)
    EducationHeritage = 0xB0,
    /// Legal contracts, taxation, logistics, supply chain (0xC0)
    LegalLogistics = 0xC0,
    /// Custom / Experimental extensions (0xFF)
    Custom = 0xFF,
}

impl OpcodeDomain {
    pub fn from_high_byte(byte: u8) -> Self {
        match byte {
            0x10 => Self::VisionAi,
            0x11 => Self::AudioSpeech,
            0x12 => Self::LlmNlp,
            0x20 => Self::Graphics3D,
            0x21 => Self::MediaTranscode,
            0x30 => Self::BioScience,
            0x31 | 0x40 => Self::HpcEngineering,
            0x32 => Self::QuantPhysics,
            0x50 => Self::EarthSpace,
            0x60 => Self::DataEtl,
            0x61 => Self::KnowledgeVector,
            0x70 => Self::CryptographyZk,
            0x71 => Self::ConfidentialTee,
            0x80 => Self::FinanceQuant,
            0x90 => Self::RoboticsSwarm,
            0xA0 => Self::MediaBroadcasting,
            0xB0 => Self::EducationHeritage,
            0xC0 => Self::LegalLogistics,
            _ => Self::Custom,
        }
    }
}

/// 16-bit structured opcode decoupling high-byte domain from low-byte task variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HierarchicalOpcode {
    pub domain: OpcodeDomain,
    pub task_code: u8,
}

impl HierarchicalOpcode {
    pub const fn new(domain: OpcodeDomain, task_code: u8) -> Self {
        Self { domain, task_code }
    }

    pub fn from_u16(code: u16) -> Self {
        let high = (code >> 8) as u8;
        let low = (code & 0x00FF) as u8;
        Self {
            domain: OpcodeDomain::from_high_byte(high),
            task_code: low,
        }
    }

    pub fn to_u16(&self) -> u16 {
        ((self.domain as u16) << 8) | (self.task_code as u16)
    }

    /// Canonical standard MCP tool name identifier for this opcode.
    pub fn canonical_mcp_tool_name(&self) -> &'static str {
        match (self.domain, self.task_code) {
            (OpcodeDomain::VisionAi, 0x01) => "skynet://vision.upscale.real_esrgan",
            (OpcodeDomain::VisionAi, 0x02) => "skynet://vision.generate.flux_schnell",
            (OpcodeDomain::AudioSpeech, 0x01) => "skynet://audio.transcribe.whisper_v3",
            (OpcodeDomain::AudioSpeech, 0x02) => "skynet://audio.stem_separation.demucs_v4",
            (OpcodeDomain::LlmNlp, 0x01) => "skynet://llm.generate.llama3_8b",
            (OpcodeDomain::LlmNlp, 0x02) => "skynet://llm.finetune.lora_70b",
            (OpcodeDomain::Graphics3D, 0x01) => "skynet://graphics.render.blender_cycles",
            (OpcodeDomain::Graphics3D, 0x02) => "skynet://graphics.reconstruct.gaussian_splatting",
            (OpcodeDomain::BioScience, 0x01) => "skynet://bio.folding.alphafold3",
            (OpcodeDomain::BioScience, 0x02) => "skynet://bio.docking.autodock_vina",
            (OpcodeDomain::HpcEngineering, 0x01) => "skynet://hpc.cfd.openfoam",
            (OpcodeDomain::CryptographyZk, 0x01) => "skynet://crypto.zk.snark_prover",
            (OpcodeDomain::ConfidentialTee, 0x01) => "skynet://privacy.tee.medical_pii_redaction",
            (OpcodeDomain::FinanceQuant, 0x01) => "skynet://defi.batch_auction.cow_solver",
            (OpcodeDomain::EarthSpace, 0x01) => "skynet://earth.weather.wrf_forecast",
            (OpcodeDomain::DataEtl, 0x01) => "skynet://data.pipeline.polars_parquet",
            (OpcodeDomain::RoboticsSwarm, 0x01) => "skynet://robotics.slam.fast_lio_3d",
            (OpcodeDomain::MediaBroadcasting, 0x01) => "skynet://media.transcode.svt_av1",
            (OpcodeDomain::EducationHeritage, 0x01) => "skynet://edu.heritage.hannom_ocr",
            (OpcodeDomain::LegalLogistics, 0x01) => "skynet://legal.logistics.vrp_optimizer",
            _ => "skynet://generic.compute.task",
        }
    }
}

/// Target hardware architecture profile required by an execution capsule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapsuleTargetArch {
    /// Nvidia CUDA (Tensor Cores / Compute Capability 7.5+)
    CudaX86_64,
    /// Apple Silicon Metal 3 (Unified Memory UMA)
    MetalAarch64,
    /// Universal WebAssembly with SIMD128
    WasmSimd,
    /// Linux x86_64 AVX-512 CPU Multithreaded
    CpuX86_64Avx512,
    /// ARM64 Linux NEON (Raspberry Pi 5 / Graviton)
    CpuArm64Neon,
    /// Intel SGX / AMD SEV-SNP Confidential Hardware Enclave
    TeeHardwareEnclave,
}

/// Standalone, self-contained runtime capsule manifest auto-fetched by workers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapsuleManifest {
    /// Canonical MCP Tool Identifier (e.g. `skynet://vision.upscale.real_esrgan`).
    pub mcp_tool_name: String,
    /// Numeric hierarchical opcode.
    pub opcode: u16,
    /// BLAKE3 checksum of the executable capsule bundle.
    pub capsule_hash: [u8; 32],
    /// Target execution architecture.
    pub target_arch: CapsuleTargetArch,
    /// Total download size in bytes.
    pub download_bytes: u64,
    /// Minimum VRAM required in Megabytes (0 if CPU/RAM-only).
    pub min_vram_mb: u32,
    /// Minimum system RAM required in Megabytes.
    pub min_ram_mb: u32,
    /// Content identifier or URI for peer-to-peer fountain retrieval.
    pub fountain_uri: String,
}

impl CapsuleManifest {
    pub fn is_compatible_with(&self, vram_mb: u32, ram_mb: u32, arch: CapsuleTargetArch) -> bool {
        self.target_arch == arch && vram_mb >= self.min_vram_mb && ram_mb >= self.min_ram_mb
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hierarchical_opcode_roundtrip() {
        let op = HierarchicalOpcode::new(OpcodeDomain::VisionAi, 0x01);
        let code = op.to_u16();
        assert_eq!(code, 0x1001);

        let parsed = HierarchicalOpcode::from_u16(code);
        assert_eq!(parsed.domain, OpcodeDomain::VisionAi);
        assert_eq!(parsed.task_code, 0x01);
        assert_eq!(parsed.canonical_mcp_tool_name(), "skynet://vision.upscale.real_esrgan");
    }

    #[test]
    fn test_capsule_manifest_hardware_compatibility() {
        let manifest = CapsuleManifest {
            mcp_tool_name: "skynet://vision.upscale.real_esrgan".to_string(),
            opcode: 0x1001,
            capsule_hash: [0xabu8; 32],
            target_arch: CapsuleTargetArch::CudaX86_64,
            download_bytes: 45_000_000,
            min_vram_mb: 8192,
            min_ram_mb: 16384,
            fountain_uri: "skynet-p2p://capsules/real_esrgan_cuda.capsule".to_string(),
        };

        // Meets requirements
        assert!(manifest.is_compatible_with(12288, 32768, CapsuleTargetArch::CudaX86_64));
        // Insufficient VRAM
        assert!(!manifest.is_compatible_with(4096, 32768, CapsuleTargetArch::CudaX86_64));
        // Wrong architecture
        assert!(!manifest.is_compatible_with(16384, 32768, CapsuleTargetArch::MetalAarch64));
    }
}
