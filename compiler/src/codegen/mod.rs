pub mod mips;
pub mod arm;
pub mod riscv;
pub mod ir_emit;

use crate::ir::IrModule;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct SourceMapEntry {
    pub line:        usize,
    pub address:     u32,
    pub instruction: u32,
    pub source:      String,
}


pub trait Backend {

    fn compile(&mut self, module: &IrModule) -> Vec<u8>;

    fn get_source_map(&self) -> Vec<SourceMapEntry> {
        Vec::new()
    }
}


#[derive(Debug, Clone, PartialEq)]
pub enum Target {
    Mips,
    MipsLe,  
    Arm,
    Riscv,
    Ir,
}

impl Target {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "mips" | "mips-be"          => Target::Mips,
            "mips-le"                   => Target::MipsLe,  
            "arm"                       => Target::Arm,
            "riscv" | "risc-v" | "rv32" => Target::Riscv,
            "ir"                        => Target::Ir,
            other => {
                eprintln!("[TARGET ERROR] Unknown target '{}'\n  Available: mips-be, mips-le, arm, riscv, ir\n  Defaulting to: mips-be", other);
                Target::Mips
            }
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Target::Mips   => "mips-be",
            Target::MipsLe => "mips-le", 
            Target::Arm    => "arm",
            Target::Riscv  => "riscv32",
            Target::Ir     => "ir",
        }
    }

    pub fn output_extension(&self) -> &str {
        match self {
            Target::Mips | Target::MipsLe => "bin",
            Target::Arm                   => "bin",
            Target::Riscv                 => "bin",
            Target::Ir                    => "ir",
        }
    }
}

pub fn select_backend(target: &Target) -> Box<dyn Backend> {
    match target {
        Target::Mips   => Box::new(mips::MipsBackend::new()),
        Target::MipsLe => Box::new(mips::MipsBackend::new_le()),
        Target::Arm    => Box::new(arm::ArmBackend::new()),
        Target::Riscv  => Box::new(riscv::RiscvBackend::new()),
        Target::Ir     => Box::new(ir_emit::IrEmitBackend::new()),
    }
}
