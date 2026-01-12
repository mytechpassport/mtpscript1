use clap::{Arg, Command};
use mtpscript_core::*;
use std::fs;
use std::io::{self, BufRead, Write};
use std::process;

fn main() {
    // Initialize crypto audit
    mtpscript_core::security::crypto_audit::init_crypto_audit();

    // Initialize dynamic taint tracking
    mtpscript_core::taint::init_dynamic_taint_tracking();

    // Initialize schema registry
    mtpscript_core::validation::init_schema_registry();

    // Initialize module registry
    mtpscript_core::modules::import::init_module_registry();

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
                )
                .arg(
                    Arg::new("key")
                        .short('k')
                        .long("key")
                        .help("Signing key file (optional)")
                        .required(false),
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
        .subcommand(
            Command::new("serve")
                .about("Start HTTP server")
                .arg(
                    Arg::new("port")
                        .short('p')
                        .long("port")
                        .help("Port to listen on")
                        .default_value("8080"),
                )
                .arg(
                    Arg::new("snapshot")
                        .short('s')
                        .long("snapshot")
                        .help("Snapshot file to serve")
                        .required(false),
                ),
        )
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
            let key = sub_matches.get_one::<String>("key");
            snapshot_command(input, output, key.map(|s| s.as_str()))
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
        Some(("serve", sub_matches)) => {
            let port = sub_matches.get_one::<String>("port").unwrap();
            let snapshot = sub_matches.get_one::<String>("snapshot");
            serve_command(port, snapshot.map(|s| s.as_str()))
        }
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

fn snapshot_command(input: &str, output: &str, _key_file: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    use mtpscript_core::snapshot::{create_snapshot, save_snapshot};
    use sha2::{Digest, Sha256};

    println!("Creating snapshot from {} to {}", input, output);

    // Read JavaScript code
    let js_code = std::fs::read_to_string(input)?;

    // Create snapshot with proper format
    let mut snapshot = Vec::new();

    // Magic bytes: "MTPJS\x00\x00\x00"
    snapshot.extend_from_slice(b"MTPJS\x00\x00\x00");

    // Version: 51 (for v5.1)
    snapshot.extend_from_slice(&51u32.to_le_bytes());

    // Size placeholder (will be updated)
    let size_offset = snapshot.len();
    snapshot.extend_from_slice(&0u64.to_le_bytes());

    // SHA-256 of JS content
    let js_hash = Sha256::digest(js_code.as_bytes());
    snapshot.extend_from_slice(&js_hash);

    // JS code
    snapshot.extend_from_slice(js_code.as_bytes());

    // Signature placeholder (128 bytes)
    snapshot.extend_from_slice(&[0u8; 128]);

    // Calculate final size (including CRC32)
    let final_size = snapshot.len() + 4;
    snapshot[size_offset..size_offset + 8].copy_from_slice(&(final_size as u64).to_le_bytes());

    // CRC32 of everything before the CRC
    let crc = crc32fast::hash(&snapshot);
    snapshot.extend_from_slice(&crc.to_le_bytes());

    // Write to file
    std::fs::write(output, &snapshot)?;

    println!("Snapshot created successfully ({} bytes)", snapshot.len());
    Ok(())
}

fn run_command(input: &str) -> Result<(), Box<dyn std::error::Error>> {
    use mtpscript_core::runtime::interpreter::Interpreter;
    use mtpscript_core::runtime::interpreter::InterpreterConfig;
    use mtpscript_core::snapshot::extract_js_code;
    use mtpscript_core::snapshot::load_snapshot;

    println!("Running snapshot {}", input);
    eprintln!("DEBUG: Starting run_command");

    let snapshot = load_snapshot(input)?;
    let js = extract_js_code(&snapshot)?;

    eprintln!("DEBUG: Extracted JS:\n{}", js);

    // Execute the JS
    let config = InterpreterConfig::default();
    let mut interpreter = Interpreter::new(config);

    // Inject effects
    eprintln!("DEBUG: About to inject effects");
    use mtpscript_core::runtime::effects::inject_effects;
    inject_effects(&mut interpreter, &[0; 32])?;
    eprintln!("DEBUG: Effects injected");

    let result = interpreter.execute(&js)?;

    println!("Execution result: {}", result);
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

fn repl_command() -> Result<(), Box<dyn std::error::Error>> {
    use mtpscript_core::runtime::interpreter::{Interpreter, InterpreterConfig};
    use mtpscript_core::runtime::effects::inject_effects;
    use mtpscript_core::lexer::scanner::Scanner;
    use mtpscript_core::parser::Parser;

    println!("MTPScript REPL v0.1.0");
    println!("Type 'exit' or 'quit' to exit, 'help' for help");
    println!();

    let config = InterpreterConfig::default();
    let mut interpreter = Interpreter::new(config);
    inject_effects(&mut interpreter, &[0; 32])?;

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("mtp> ");
        stdout.flush()?;

        let mut line = String::new();
        if stdin.lock().read_line(&mut line)? == 0 {
            // EOF
            println!();
            break;
        }

        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        match line {
            "exit" | "quit" => {
                println!("Goodbye!");
                break;
            }
            "help" => {
                println!("MTPScript REPL Commands:");
                println!("  exit, quit  - Exit the REPL");
                println!("  help        - Show this help message");
                println!("  .gas        - Show gas used");
                println!();
                println!("Enter MTPScript expressions to evaluate them.");
                continue;
            }
            ".gas" => {
                println!("Gas used: {}", interpreter.gas_used());
                continue;
            }
            _ => {}
        }

        // Try to parse and execute the input
        match execute_repl_line(&mut interpreter, line) {
            Ok(result) => {
                println!("=> {}", result);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }

    Ok(())
}

fn execute_repl_line(
    interpreter: &mut mtpscript_core::runtime::interpreter::Interpreter,
    line: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // Wrap expression in a return for execution
    let js_code = if line.contains("function") || line.contains("const") || line.contains("let") {
        format!("{}\n", line)
    } else {
        format!("return {};", line)
    };

    let result = interpreter.execute(&js_code)?;
    Ok(format!("{}", result))
}

fn serve_command(port: &str, snapshot_path: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    use tiny_http::{Server, Response, Header, Method};
    use mtpscript_core::runtime::interpreter::{Interpreter, InterpreterConfig};
    use mtpscript_core::runtime::effects::inject_effects;
    use mtpscript_core::snapshot::{load_snapshot, extract_js_code};

    let addr = format!("0.0.0.0:{}", port);
    println!("Starting HTTP server on {}", addr);

    let server = Server::http(&addr)
        .map_err(|e| format!("Failed to start server: {}", e))?;

    // Load snapshot if provided
    let js_code = if let Some(path) = snapshot_path {
        println!("Loading snapshot: {}", path);
        let snapshot = load_snapshot(path)?;
        Some(extract_js_code(&snapshot)?)
    } else {
        None
    };

    println!("Server started. Press Ctrl+C to stop.");
    println!();

    for request in server.incoming_requests() {
        let method = request.method().clone();
        let url = request.url().to_string();

        println!("{} {}", method, url);

        // Handle health check
        if url == "/health" {
            let response = Response::from_string(r#"{"status":"ok"}"#)
                .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap());
            request.respond(response)?;
            continue;
        }

        // Handle OpenAPI spec
        if url == "/openapi.json" {
            let spec = r#"{
  "openapi": "3.0.0",
  "info": {
    "title": "MTPScript API",
    "version": "0.1.0"
  },
  "paths": {
    "/health": {
      "get": {
        "summary": "Health check",
        "responses": {
          "200": {
            "description": "OK"
          }
        }
      }
    },
    "/execute": {
      "post": {
        "summary": "Execute MTPScript code",
        "requestBody": {
          "content": {
            "application/json": {
              "schema": {
                "type": "object",
                "properties": {
                  "code": {"type": "string"}
                }
              }
            }
          }
        },
        "responses": {
          "200": {
            "description": "Execution result"
          }
        }
      }
    }
  }
}"#;
            let response = Response::from_string(spec)
                .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap());
            request.respond(response)?;
            continue;
        }

        // Handle execution endpoint
        if url == "/execute" && method == Method::Post {
            // Read request body
            let mut body = String::new();
            request.as_reader().read_to_string(&mut body)?;

            // Parse JSON
            let parsed: serde_json::Value = serde_json::from_str(&body)
                .unwrap_or_else(|_| serde_json::json!({"code": body}));

            let code = parsed.get("code")
                .and_then(|v| v.as_str())
                .unwrap_or(&body);

            // Execute code
            let config = InterpreterConfig::default();
            let mut interpreter = Interpreter::new(config);
            inject_effects(&mut interpreter, &[0; 32])?;

            let result = match interpreter.execute(code) {
                Ok(v) => serde_json::json!({
                    "success": true,
                    "result": format!("{}", v),
                    "gas_used": interpreter.gas_used()
                }),
                Err(e) => serde_json::json!({
                    "success": false,
                    "error": format!("{}", e)
                }),
            };

            let response = Response::from_string(result.to_string())
                .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap());
            request.respond(response)?;
            continue;
        }

        // Execute loaded snapshot for other routes
        if let Some(ref code) = js_code {
            let config = InterpreterConfig::default();
            let mut interpreter = Interpreter::new(config);
            inject_effects(&mut interpreter, &[0; 32])?;

            // Set request context
            interpreter.global_scope.insert(
                "request".to_string(),
                mtpscript_core::runtime::value::Value::Object(
                    [
                        ("method".to_string(), mtpscript_core::runtime::value::Value::String(format!("{}", method))),
                        ("url".to_string(), mtpscript_core::runtime::value::Value::String(url.clone())),
                    ].into_iter().collect()
                ),
            );

            let result = match interpreter.execute(code) {
                Ok(v) => serde_json::json!({
                    "result": format!("{}", v)
                }),
                Err(e) => serde_json::json!({
                    "error": format!("{}", e)
                }),
            };

            let response = Response::from_string(result.to_string())
                .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap());
            request.respond(response)?;
        } else {
            // No snapshot loaded
            let response = Response::from_string(r#"{"error":"No snapshot loaded"}"#)
                .with_status_code(404)
                .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap());
            request.respond(response)?;
        }
    }

    Ok(())
}
