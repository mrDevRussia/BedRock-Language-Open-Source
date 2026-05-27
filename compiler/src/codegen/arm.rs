// ARM Backend — Stub
// يُفعَّل بـ --target arm
// هيتكتب بعد اكتمال الـ MIPS IR-native

use crate::ir::IrModule;
use crate::codegen::{Backend, SourceMapEntry};

pub struct ArmBackend;

impl ArmBackend {
    pub fn new() -> Self { ArmBackend }
}

impl Backend for ArmBackend {
    fn compile(&mut self, _module: &IrModule) -> Vec<u8> {
        eprintln!(
            "[ARM] Backend not yet implemented.\n  Use --target mips or --target ir for now."
        );
        std::process::exit(1);
    }

    fn get_source_map(&self) -> Vec<SourceMapEntry> { Vec::new() }
}