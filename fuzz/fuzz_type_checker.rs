use mtpscript_core::parser::ast::Program;
use mtpscript_core::types::checker::type_check_program;

#[cfg(fuzzing)]
fuzz_target!(|data: &[u8]| {
    if let Ok(input) = std::str::from_utf8(data) {
        // Try to parse as MTPScript
        if let Ok(program) = mtpscript_core::parser::Parser::new(
            &mtpscript_core::lexer::scanner::Scanner::new(input)
                .scan_tokens()
                .unwrap(),
        )
        .parse()
        {
            // Type check - should not crash
            let _ = type_check_program(&program);
        }
    }
});
