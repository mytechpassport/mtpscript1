use clap::{value_parser, Arg, Command};
use crc32fast;
use mtpscript_core::compiler::codegen;
use mtpscript_core::effects::desugar_async_effects;
use mtpscript_core::errors::compile::CompileError;
use mtpscript_core::errors::runtime::RuntimeError;
use mtpscript_core::errors::MtpError;
use mtpscript_core::ir::lower;
use mtpscript_core::lexer::scanner::Scanner;
use mtpscript_core::parser::Parser;
use mtpscript_core::runtime::Interpreter;
use mtpscript_core::security::sign::sign_ecdsa_p256;
use mtpscript_core::security::verify;
use mtpscript_core::types::checker::TypeChecker;
use sha2::{Digest, Sha256};
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::str;
use tiny_http::{Header, Response, Server};

#[derive(Debug)]
pub enum CliError {
    Io(io::Error),
    Compile(CompileError),
    Runtime(RuntimeError),
    Mtp(MtpError),
    Utf8(str::Utf8Error),
    TinyHttp(String),
    Snapshot(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::Io(err) => write!(f, "I/O error: {}", err),
            CliError::Compile(err) => write!(f, "Compilation error: {:?}", err),
            CliError::Runtime(err) => write!(f, "Runtime error: {:?}", err),
            CliError::Mtp(err) => write!(f, "MTP error: {:?}", err),
            CliError::Utf8(err) => write!(f, "Invalid UTF-8 data: {}", err),
            CliError::TinyHttp(err) => write!(f, "Server error: {}", err),
            CliError::Snapshot(msg) => write!(f, "Snapshot error: {}", msg),
        }
    }
}

impl From<io::Error> for CliError {
    fn from(err: io::Error) -> Self {
        CliError::Io(err)
    }
}

impl From<CompileError> for CliError {
    fn from(err: CompileError) -> Self {
        CliError::Compile(err)
    }
}

impl From<RuntimeError> for CliError {
    fn from(err: RuntimeError) -> Self {
        CliError::Runtime(err)
    }
}

impl From<MtpError> for CliError {
    fn from(err: MtpError) -> Self {
        CliError::Mtp(err)
    }
}

impl From<str::Utf8Error> for CliError {
    fn from(err: str::Utf8Error) -> Self {
        CliError::Utf8(err)
    }
}

pub fn run() -> Result<(), CliError> {
    let matches = build_cli().get_matches();

    match matches.subcommand() {
        Some(("compile", sub)) => {
            let input = PathBuf::from(sub.get_one::<String>("input").unwrap());
            let output = PathBuf::from(sub.get_one::<String>("output").unwrap());
            compile_command(&input, &output)
        }
        Some(("snapshot", sub)) => {
            let input = PathBuf::from(sub.get_one::<String>("input").unwrap());
            let output = PathBuf::from(sub.get_one::<String>("output").unwrap());
            let key = PathBuf::from(sub.get_one::<String>("key").unwrap());
            snapshot_command(&input, &output, &key)
        }
        Some(("serve", sub)) => {
            let port = *sub.get_one::<u16>("port").unwrap();
            let snapshot = PathBuf::from(sub.get_one::<String>("snapshot").unwrap());
            serve_command(&snapshot, port)
        }
        Some(("run", sub)) => {
            let input = PathBuf::from(sub.get_one::<String>("input").unwrap());
            let cert = sub.get_one::<String>("cert").map(PathBuf::from);
            run_command(&input, cert.as_deref())
        }
        Some(("execute", sub)) => {
            let input = PathBuf::from(sub.get_one::<String>("input").unwrap());
            execute_command(&input)
        }
        Some(("repl", _)) => repl_command(),
        _ => Ok(()),
    }
}

fn build_cli() -> Command {
    Command::new("mtp")
        .about("MTPScript CLI tool")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("compile")
                .about("Compile MTPScript source to deterministic JS")
                .arg(
                    Arg::new("input")
                        .help("MTPScript source file")
                        .required(true),
                )
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .help("Output JS file")
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("snapshot")
                .about("Package compiled JS into a .msqs snapshot")
                .arg(Arg::new("input").help("Input JS file").required(true))
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .help("Destination snapshot path")
                        .required(true),
                )
                .arg(
                    Arg::new("key")
                        .short('k')
                        .long("key")
                        .help("PEM private key for signing")
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("serve")
                .about("Start the reference HTTP server (hot reload not supported)")
                .arg(
                    Arg::new("snapshot")
                        .help("Snapshot file to serve")
                        .required(true),
                )
                .arg(
                    Arg::new("port")
                        .short('p')
                        .long("port")
                        .help("Port to listen on")
                        .value_parser(value_parser!(u16))
                        .default_value("8080"),
                ),
        )
        .subcommand(
            Command::new("run")
                .about("Run a compiled snapshot and optionally verify it")
                .arg(Arg::new("input").help("Snapshot file").required(true))
                .arg(
                    Arg::new("cert")
                        .short('c')
                        .long("cert")
                        .help("PEM certificate for verifying the snapshot"),
                ),
        )
        .subcommand(
            Command::new("execute")
                .about("Compile and run MTPScript file directly")
                .arg(
                    Arg::new("input")
                        .help("MTPScript source file")
                        .required(true),
                ),
        )
        .subcommand(Command::new("repl").about("Start interactive REPL"))
}

fn compile_command(input: &Path, output: &Path) -> Result<(), CliError> {
    let source = fs::read_to_string(input)?;
    let mut scanner = Scanner::new(&source)?;
    let tokens = scanner.scan_tokens()?;
    let mut parser = Parser::new(&tokens);
    let mut program = parser.parse()?;
    let mut type_checker = TypeChecker::new();
    type_checker.typecheck_program(&program)?;
    // Desugar await expressions to Async.await calls (per spec §7-a)
    desugar_async_effects(&mut program)?;
    let ir = lower::lower_ast_to_ir(&program)?;
    let js = codegen::compile_ir_to_js(&ir)?;
    fs::write(output, js)?;
    println!("Compiled '{}' -> '{}'", input.display(), output.display());
    Ok(())
}

fn snapshot_command(input: &Path, output: &Path, key: &Path) -> Result<(), CliError> {
    let js_bytes = fs::read(input)?;
    let hash: [u8; 32] = Sha256::digest(&js_bytes).into();
    let key_pem = fs::read_to_string(key)?;
    let signature = sign_ecdsa_p256(&hash, &key_pem)?;

    if signature.len() > 128 {
        return Err(CliError::Snapshot(format!(
            "Signature length {} exceeds 128 bytes",
            signature.len()
        )));
    }

    let mut snapshot = Vec::new();
    snapshot.extend_from_slice(b"MTPJS\x00\x00\x00");
    snapshot.extend_from_slice(&51u32.to_le_bytes());
    snapshot.extend_from_slice(&0u64.to_le_bytes());
    snapshot.extend_from_slice(&hash);
    snapshot.extend_from_slice(&js_bytes);

    let mut padded_signature = [0u8; 128];
    padded_signature[..signature.len()].copy_from_slice(&signature);
    snapshot.extend_from_slice(&padded_signature);

    let total_size = (snapshot.len() + 4) as u64;
    snapshot[12..20].copy_from_slice(&total_size.to_le_bytes());

    let crc = crc32fast::hash(&snapshot);
    snapshot.extend_from_slice(&crc.to_le_bytes());

    fs::write(output, &snapshot)?;
    println!("Snapshot written to '{}'", output.display());
    Ok(())
}

fn run_command(snapshot_path: &Path, cert: Option<&Path>) -> Result<(), CliError> {
    if let Some(cert_path) = cert {
        let snapshot_str = snapshot_path
            .to_str()
            .ok_or_else(|| CliError::Snapshot("Invalid snapshot path".into()))?;
        let cert_str = cert_path
            .to_str()
            .ok_or_else(|| CliError::Snapshot("Invalid certificate path".into()))?;
        verify::verify_snapshot(snapshot_str, cert_str)?;
        println!(
            "Verified snapshot using certificate '{}'",
            cert_path.display()
        );
    }

    let data = fs::read(snapshot_path)?;
    if data.len() < 132 {
        return Err(CliError::Snapshot("Snapshot file too small".into()));
    }

    let js_start = 52;
    let js_end = data.len() - 132;
    let js = str::from_utf8(&data[js_start..js_end])?;

    println!("Executing snapshot '{}'", snapshot_path.display());
    let mut interpreter = Interpreter::new();
    let result = interpreter.execute(js)?;
    println!("Execution result: {}", result);
    Ok(())
}

fn serve_command(snapshot_path: &Path, port: u16) -> Result<(), CliError> {
    let server = Server::http(("0.0.0.0", port)).map_err(|e| CliError::TinyHttp(e.to_string()))?;
    println!(
        "Serving MTPScript API from snapshot '{}' on http://0.0.0.0:{}",
        snapshot_path.display(),
        port
    );

    // Load snapshot
    let snapshot = fs::read(snapshot_path)?;

    // Create router (placeholder - in real impl, would parse from snapshot or config)
    let router = mtpscript_core::api::router::Router::new();

    // Gas limit from env
    let gas_limit = mtpscript_core::runtime::get_gas_limit();

    // Create request handler
    let handler = mtpscript_core::api::handler::RequestHandler::new(snapshot, gas_limit, router);

    for mut request in server.incoming_requests() {
        // Convert tiny_http request to our HttpRequest
        let method = request.method().to_string();
        let path = request.url().to_string();
        let mut headers = std::collections::HashMap::new();
        for header in request.headers() {
            headers.insert(header.field.to_string(), header.value.to_string());
        }
        let mut body = Vec::new();
        request.as_reader().read_to_end(&mut body)?;

        let http_req = mtpscript_core::api::handler::HttpRequest {
            method,
            path,
            headers,
            body,
        };

        // Handle request
        match handler.handle_request(http_req) {
            Ok(resp) => {
                let mut response = Response::from_data(resp.body);
                for (name, value) in resp.headers {
                    if let Ok(header) = Header::from_bytes(name.as_bytes(), value.as_bytes()) {
                        response = response.with_header(header);
                    }
                }
                if let Err(e) = request.respond(response) {
                    eprintln!("Error responding to request: {}", e);
                }
            }
            Err(e) => {
                let error_msg = format!("Internal server error: {:?}", e);
                let response = Response::from_string(error_msg)
                    .with_status_code(500)
                    .with_header(Header::from_bytes(b"Content-Type", b"text/plain").unwrap());
                if let Err(e) = request.respond(response) {
                    eprintln!("Error responding with error: {}", e);
                }
            }
        }
    }

    Ok(())
}

fn execute_command(input: &Path) -> Result<(), CliError> {
    println!("Executing MTPScript file: {}", input.display());

    // Compile to JS in memory
    let source = fs::read_to_string(input)?;
    let mut scanner = Scanner::new(&source)?;
    let tokens = scanner.scan_tokens()?;
    let mut parser = Parser::new(&tokens);
    let mut program = parser.parse()?;
    let mut type_checker = TypeChecker::new();
    type_checker.typecheck_program(&program)?;
    // Desugar await expressions to Async.await calls (per spec §7-a)
    desugar_async_effects(&mut program)?;
    let ir = lower::lower_ast_to_ir(&program)?;
    let js = codegen::compile_ir_to_js(&ir)?;

    // Execute JS directly
    let mut interpreter = Interpreter::new();
    match interpreter.execute(&js) {
        Ok(result) => {
            println!("Execution result: {}", result);
        }
        Err(e) => {
            // Check if it's a GasExhausted error and format as JSON
            let error_str = format!("{:?}", e);
            if error_str.contains("GasExhausted") {
                // Extract gas_limit and gas_used from the error
                if let Some(limit_start) = error_str.find("gas_limit: ") {
                    let limit_end = error_str[limit_start..]
                        .find(',')
                        .unwrap_or(error_str.len() - limit_start);
                    let gas_limit: u64 = error_str[limit_start + 11..limit_start + limit_end]
                        .trim()
                        .parse()
                        .unwrap_or(0);

                    if let Some(used_start) = error_str.find("gas_used: ") {
                        let used_end = error_str[used_start..]
                            .find('}')
                            .unwrap_or(error_str.len() - used_start);
                        let gas_used: u64 = error_str[used_start + 10..used_start + used_end]
                            .trim()
                            .parse()
                            .unwrap_or(0);

                        println!("Execution result: {{\"error\":\"GasExhausted\",\"gasLimit\":{},\"gasUsed\":{}}}", gas_limit, gas_used);
                        return Ok(());
                    }
                }
            }
            return Err(e.into());
        }
    }

    Ok(())
}

fn repl_command() -> Result<(), CliError> {
    use std::io::{self, Write};

    println!("MTPScript REPL - Type 'exit' to quit");
    println!("Note: MTPScript expects API definitions. Example: api GET \"/test\" uses {{}} {{ respond json({{\"msg\": \"hello\"}}) }}");

    loop {
        print!("mtp> ");
        io::stdout().flush()?;

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => break, // EOF
            Ok(_) => {}
            Err(e) => return Err(CliError::Io(e)),
        }

        let input = input.trim();

        if input == "exit" {
            println!("Goodbye!");
            break;
        }

        if input.is_empty() {
            continue;
        }

        // Try to evaluate the input
        match evaluate_repl_input(input) {
            Ok(result) => println!("=> {}", result),
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    Ok(())
}

fn evaluate_repl_input(input: &str) -> Result<String, CliError> {
    // Simple REPL that treats input as an expression
    let full_input = if input.contains('=') || input.contains('{') {
        input.to_string()
    } else {
        // Treat as expression - wrap in console.log
        format!("console.log({});", input)
    };

    let mut scanner = Scanner::new(&full_input)?;
    let tokens = scanner.scan_tokens()?;
    let mut parser = Parser::new(&tokens);
    let program = parser.parse()?;
    let mut type_checker = TypeChecker::new();
    type_checker.typecheck_program(&program)?;
    let ir = lower::lower_ast_to_ir(&program)?;
    let js = codegen::compile_ir_to_js(&ir)?;

    let mut interpreter = Interpreter::new();
    let result = interpreter.execute(&js)?;

    Ok(result.to_string())
}

impl From<()> for CliError {
    fn from(_: ()) -> Self {
        CliError::Snapshot("Empty result".into())
    }
}
