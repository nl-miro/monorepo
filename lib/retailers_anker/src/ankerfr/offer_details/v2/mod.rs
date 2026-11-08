//! Version 2 of the ankerfr offer details pipeline.

#![allow(dead_code)] // Stubs awaiting v2 pipeline integration.

use html_extractor::io::Architecture;
use retailer_sauce::structuralized::StructuralizedData;

pub fn architecture() -> Architecture {
    todo!()
}

pub fn normalize(_json: &str) -> Result<bodul_shared::NormalizedData, String> {
    todo!()
}

pub fn structuralize(_json: &str) -> Result<StructuralizedData, String> {
    todo!()
}
