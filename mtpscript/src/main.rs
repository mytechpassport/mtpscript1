mod cli;

fn main() {
    if let Err(err) = cli::commands::run() {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}
