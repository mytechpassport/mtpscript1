pub mod async_effect;
pub mod checker;

use crate::errors::compile::CompileError;
use crate::parser::ast::Program;

pub fn check_effects(program: &Program) -> Result<(), CompileError> {
    checker::check_program_effects(program)
}

pub fn desugar_async_effects(program: &mut Program) -> Result<(), CompileError> {
    async_effect::desugar_async_effects(program)
}
