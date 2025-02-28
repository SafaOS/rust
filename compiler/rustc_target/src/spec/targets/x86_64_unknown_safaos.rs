use crate::spec::{
    Cc, LinkerFlavor, Lld, PanicStrategy, RelocModel, StackProbeType, Target, TargetMetadata,
    TargetOptions,
};

pub fn target() -> Target {
    Target {
        llvm_target: "x86_64-unknown-none".into(),
        pointer_width: 64,
        data_layout:
            "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128".into(),
        arch: "x86_64".into(),
        options: TargetOptions {
            os: "safaos".into(),
            cpu: "x86-64".into(),
            linker: Some("rust-lld".into()),
            linker_flavor: LinkerFlavor::Gnu(Cc::No, Lld::Yes),
            stack_probes: StackProbeType::Inline,
            relocation_model: RelocModel::Static,
            disable_redzone: true,
            panic_strategy: PanicStrategy::Abort,
            features: "-mmx,-sse,+soft-float".into(),
            ..Default::default()
        },
        metadata: TargetMetadata { description: None, tier: None, host_tools: None, std: None },
    }
}
