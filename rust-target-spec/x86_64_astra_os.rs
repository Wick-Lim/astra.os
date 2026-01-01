// Target specification for x86_64-astra_os
// This file goes in: rust/compiler/rustc_target/src/spec/x86_64_astra_os.rs

use crate::spec::{Cc, LinkerFlavor, Lld, PanicStrategy, Target, TargetOptions};

pub fn target() -> Target {
    let mut base = super::none_base::opts();
    base.cpu = "x86-64".into();
    base.max_atomic_width = Some(64);
    base.features = "-mmx,-sse,+soft-float".into();

    // Important: We're using LLD for linking
    base.linker_flavor = LinkerFlavor::Gnu(Cc::No, Lld::Yes);
    base.linker = Some("rust-lld".into());

    // OS-specific settings
    base.os = "astra_os".into();
    base.vendor = "unknown".into();
    base.env = "".into();

    // Kernel settings
    base.panic_strategy = PanicStrategy::Abort;
    base.disable_redzone = true;
    base.executables = true;

    // Code model for kernel
    base.code_model = Some("kernel".into());

    // Relocation model
    base.relocation_model = "static".into();

    // Thread local storage (we'll implement this)
    base.has_thread_local = true;

    Target {
        llvm_target: "x86_64-unknown-none".into(),
        pointer_width: 64,
        data_layout: "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128".into(),
        arch: "x86_64".into(),
        options: base,
    }
}
