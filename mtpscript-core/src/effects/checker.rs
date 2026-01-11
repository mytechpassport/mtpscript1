use crate::errors::compile::CompileError;
use crate::parser::ast::Program;

pub fn check_program_effects(_program: &Program) -> Result<(), CompileError> {
    // Placeholder implementation
    Ok(())
}
