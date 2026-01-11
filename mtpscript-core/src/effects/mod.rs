pub mod checker;

use crate::errors::compile::CompileError;
use crate::parser::ast::Program;

pub fn check_effects(_program: &Program) -> Result<(), CompileError> {
    // Placeholder implementation - for now, always succeeds
    // Real implementation would check effect declarations and usage
    Ok(())
}
