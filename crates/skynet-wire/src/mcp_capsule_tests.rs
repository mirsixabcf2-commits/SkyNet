//! `mcp_capsule_tests`: Unit tests for hierarchical opcodes and MCP capsule manifests.

use crate::mcp_capsule::{CapsuleManifest, CapsuleTargetArch, HierarchicalOpcode, OpcodeDomain};

#[test]
fn test_hierarchical_opcode_roundtrip() {
    let op = HierarchicalOpcode::new(OpcodeDomain::VisionAi, 0x01);
    let code = op.to_u16();
    assert_eq!(code, 0x1001);

    let parsed = HierarchicalOpcode::from_u16(code);
    assert_eq!(parsed.domain, OpcodeDomain::VisionAi);
    assert_eq!(parsed.task_code, 0x01);
    assert_eq!(
        parsed.canonical_mcp_tool_name(),
        "skynet://vision.upscale.real_esrgan"
    );
}

#[test]
fn test_micro_train_evolution_opcode_domain_roundtrip() {
    assert_eq!(OpcodeDomain::MicroTrainEvolution as u8, 0x13);

    let mlp = HierarchicalOpcode::from_u16(0x1301);
    assert_eq!(mlp.domain, OpcodeDomain::MicroTrainEvolution);
    assert_eq!(mlp.task_code, 0x01);
    assert_eq!(mlp.canonical_mcp_tool_name(), "skynet://micro_train.mlp_step");

    let lora = HierarchicalOpcode::from_u16(0x1302);
    assert_eq!(lora.domain, OpcodeDomain::MicroTrainEvolution);
    assert_eq!(
        lora.canonical_mcp_tool_name(),
        "skynet://micro_train.lora_adapter"
    );

    let plasmid = HierarchicalOpcode::from_u16(0x1303);
    assert_eq!(plasmid.domain, OpcodeDomain::MicroTrainEvolution);
    assert_eq!(
        plasmid.canonical_mcp_tool_name(),
        "skynet://evolution.plasmid_guided"
    );
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

    assert!(manifest.is_compatible_with(12288, 32768, CapsuleTargetArch::CudaX86_64));
    assert!(!manifest.is_compatible_with(4096, 32768, CapsuleTargetArch::CudaX86_64));
    assert!(!manifest.is_compatible_with(16384, 32768, CapsuleTargetArch::MetalAarch64));
}
