use clap::{value_parser, Arg, Command};
use crc32fast::hash;
use mtpscript_core::compiler::codegen;
use mtpscript_core::errors::compile::CompileError;
use mtpscript_core::errors::runtime::RuntimeError;
use mtpscript_core::errors::MtpError;
use mtpscript_core::ir::lower;
use mtpscript_core::lexer::scanner::Scanner;
use mtpscript_core::parser::Parser;
use mtpscript_core::runtime::Interpreter;
use mtpscript_core::security::sign::sign_ecdsa_p256;
use mtpscript_core::security::verify;
use sha2::{Digest, Sha256};
use std::error::Error;
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
    Server(tiny_http::Error),
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
            CliError::Server(err) => write!(f, "Server error: {}", err),
            CliError::Snapshot(msg) => write!(f, "Snapshot error: {}", msg),
        }
    }
}

impl Error for CliError {}

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

impl From<tiny_http::Error> for CliError {
    fn from(err: tiny_http::Error) -> Self {
        CliError::Server(err)
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
            serve_command(port)
        }
        Some(("run", sub)) => {
            let input = PathBuf::from(sub.get_one::<String>("input").unwrap());
            let cert = sub.get_one::<String>("cert").map(PathBuf::from);
            run_command(&input, cert.as_deref())
        }
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
}

fn compile_command(input: &Path, output: &Path) -> Result<(), CliError> {
    let source = fs::read_to_string(input)?;
    let mut scanner = Scanner::new(&source);
    let tokens = scanner.scan_tokens()?;
    let mut parser = Parser::new(&tokens);
    let program = parser.parse()?;
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

    let crc = hash(&snapshot);
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

fn serve_command(port: u16) -> Result<(), CliError> {
    let server = Server::http(("0.0.0.0", port))?;
    println!("Serving MTPScript runtime on http://0.0.0.0:{}", port);

    for request in server.incoming_requests() {
        let header = Header::from_bytes(b"Content-Type", b"text/plain; charset=utf-8")?;
        let response = Response::from_string("MTPScript runtime placeholder").with_header(header);
        request.respond(response)?;
    }

    Ok(())
}
