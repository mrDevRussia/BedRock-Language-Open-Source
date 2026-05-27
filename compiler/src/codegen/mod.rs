pub mod mips;
pub mod arm;
pub mod ir_emit;

use crate::ir::IrModule;
use serde::Serialize;

// ─────────────────────────────────────────────────────────
//  SourceMapEntry — shared by all backends that produce one
// ─────────────────────────────────────────────────────────
#[derive(Debug, Clone, Serialize)]
pub struct SourceMapEntry {
    pub line:        usize,
    pub address:     u32,
    pub instruction: u32,
    pub source:      String,
}

// ─────────────────────────────────────────────────────────
//  Backend trait
//  كل معمارية تطبق ده
// ─────────────────────────────────────────────────────────
pub trait Backend {
    /// يأخذ IrModule ويرجع binary bytes
    fn compile(&mut self, module: &IrModule) -> Vec<u8>;

    /// Source map — اختياري، الـ stubs ترجع vec فاضي
    fn get_source_map(&self) -> Vec<SourceMapEntry> {
        Vec::new()
    }
}

// ─────────────────────────────────────────────────────────
//  Target enum
// ─────────────────────────────────────────────────────────
#[derive(Debug, Clone, PartialEq)]
pub enum Target {
    Mips,
    Arm,
    Ir,
}

impl Target {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "mips" => Target::Mips,
            "arm"  => Target::Arm,
            "ir"   => Target::Ir,
            other  => {
                eprintln!(
                    "[TARGET ERROR] Unknown target '{}'\n  Available: mips, arm, ir\n  Defaulting to: mips",
                    other
                );
                Target::Mips
            }
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Target::Mips => "mips",
            Target::Arm  => "arm",
            Target::Ir   => "ir",
        }
    }

    pub fn output_extension(&self) -> &str {
        match self {
            Target::Mips => "bin",
            Target::Arm  => "bin",
            Target::Ir   => "ir",
        }
    }
}

// ─────────────────────────────────────────────────────────
//  Factory — يرجع الـ backend الصح
// ─────────────────────────────────────────────────────────
pub fn select_backend(target: &Target) -> Box<dyn Backend> {
    match target {
        Target::Mips => Box::new(mips::MipsBackend::new()),
        Target::Arm  => Box::new(arm::ArmBackend::new()),
        Target::Ir   => Box::new(ir_emit::IrEmitBackend::new()),
    }
}