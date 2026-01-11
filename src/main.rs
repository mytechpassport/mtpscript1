use clap::{Arg, Command};
use mtpscript_core::*;
use std::fs;
use std::process;

fn main() {
    // Initialize crypto audit
    mtpscript_core::security::crypto_audit::init_crypto_audit();

    // Initialize dynamic taint tracking
    mtpscript_core::taint::init_dynamic_taint_tracking();

    // Initialize schema registry
    mtpscript_core::validation::init_schema_registry();

    let matches = Command::new("mtpscript")
        .version("0.1.0")
        .author("MTPScript Team")
        .about("MTPScript compiler and runtime")
        .subcommand(
            Command::new("compile")
                .about("Compile MTPScript to JavaScript")
                .arg(Arg::new("input").help("Input .mtp file").required(true))
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .help("Output .js file")
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("snapshot")
                .about("Create snapshot from JavaScript")
                .arg(Arg::new("input").help("Input .js file").required(true))
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .help("Output .msqs file")
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("run")
                .about("Run a snapshot")
                .arg(Arg::new("input").help("Input .msqs file").required(true)),
        )
        .subcommand(
            Command::new("execute")
                .about("Compile and run MTPScript file")
                .arg(Arg::new("input").help("Input .mtp file").required(true)),
        )
        .subcommand(Command::new("repl").about("Start interactive REPL"))
        .subcommand(Command::new("serve").about("Start HTTP server"))
        .get_matches();

    let result = match matches.subcommand() {
        Some(("compile", sub_matches)) => {
            let input = sub_matches.get_one::<String>("input").unwrap();
            let output = sub_matches.get_one::<String>("output").unwrap();
            compile_command(input, output)
        }
        Some(("snapshot", sub_matches)) => {
            let input = sub_matches.get_one::<String>("input").unwrap();
            let output = sub_matches.get_one::<String>("output").unwrap();
            snapshot_command(input, output)
        }
        Some(("run", sub_matches)) => {
            let input = sub_matches.get_one::<String>("input").unwrap();
            run_command(input)
        }
        Some(("execute", sub_matches)) => {
            let input = sub_matches.get_one::<String>("input").unwrap();
            execute_command(input)
        }
        Some(("repl", _)) => repl_command(),
        Some(("serve", _)) => serve_command(),
        _ => {
            eprintln!("No subcommand provided. Use --help for usage.");
            process::exit(1);
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn compile_command(input: &str, output: &str) -> Result<(), Box<dyn std::error::Error>> {
    use mtpscript_core::compiler::codegen::compile_ir_to_js;
    use mtpscript_core::ir::lower::lower_program;
    use mtpscript_core::lexer::scanner::Scanner;
    use mtpscript_core::parser::ast::*;
    use mtpscript_core::parser::Parser;
    use mtpscript_core::snapshot::create_test_snapshot;
    use mtpscript_core::snapshot::save_snapshot;
    use mtpscript_core::types::checker::type_check_program;

    println!("Compiling {} to {}", input, output);

    let content = std::fs::read_to_string(input)?;
    let tokens = Scanner::new(&content).scan_tokens()?;
    let program = Parser::new(&tokens).parse()?;
    type_check_program(&program)?;
    let ir = lower_program(&program)?;
    let js = compile_ir_to_js(&ir)?;
    let snapshot = create_test_snapshot(&js)?;
    save_snapshot(&snapshot, output)?;

    println!("Compilation successful");
    Ok(())
}

fn snapshot_command(input: &str, output: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating snapshot from {} to {}", input, output);
    // TODO: Implement snapshot creation
    Ok(())
}

fn run_command(input: &str) -> Result<(), Box<dyn std::error::Error>> {
    use mtpscript_core::snapshot::load_snapshot;
    use mtpscript_core::snapshot::extract_js_code;

    println!("Running snapshot {}", input);

    let snapshot = load_snapshot(input)?;
    let js = extract_js_code(&snapshot)?;
    println!("Extracted JS:\n{}", js);
    println!("Snapshot run successfully (JS extracted)");
    Ok(())
}

fn execute_command(input: &str) -> Result<(), Box<dyn std::error::Error>> {
    use tempfile::NamedTempFile;

    println!("Executing {}", input);

    // Create temp file for snapshot
    let temp_file = NamedTempFile::new()?;
    let temp_path = temp_file.path().to_string_lossy().to_string();

    // Compile
    compile_command(input, &temp_path)?;

    // Run
    run_command(&temp_path)?;

    Ok(())
}

    // Run
    let run_status = Command::new("mtpjs").arg(&temp_path).status()?;

    if !run_status.success() {
        return Err("Execution failed".into());
    }

    Ok(())
}

fn repl_command() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting REPL...");
    // TODO: Implement REPL
    Ok(())
}

fn serve_command() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting HTTP server");
    // TODO: Implement HTTP server
    Ok(())
}
