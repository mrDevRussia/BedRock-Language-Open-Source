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

    fn compile(&mut self, module: &IrModule) -> Vec<u8>;

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
    MipsLe,  
    Arm,
    Ir,
}

impl Target {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "mips" | "mips-be" => Target::Mips,
            "mips-le"          => Target::MipsLe,  
            "arm"              => Target::Arm,
            "ir"               => Target::Ir,
            other => {
                eprintln!("[TARGET ERROR] Unknown target '{}'\n  Available: mips-be, mips-le, arm, ir\n  Defaulting to: mips-be", other);
                Target::Mips
            }
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Target::Mips   => "mips-be",
            Target::MipsLe => "mips-le", 
            Target::Arm    => "arm",
            Target::Ir     => "ir",
        }
    }

    pub fn output_extension(&self) -> &str {
        match self {
            Target::Mips | Target::MipsLe => "bin",
            Target::Arm                   => "bin",
            Target::Ir                    => "ir",
        }
    }
}

pub fn select_backend(target: &Target) -> Box<dyn Backend> {
    match target {
        Target::Mips   => Box::new(mips::MipsBackend::new()),
        Target::MipsLe => Box::new(mips::MipsBackend::new_le()),
        Target::Arm    => Box::new(arm::ArmBackend::new()),
        Target::Ir     => Box::new(ir_emit::IrEmitBackend::new()),
    }
}