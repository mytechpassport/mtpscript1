use clap::{Arg, Command};

fn main() {
    let matches = Command::new("mtpscript")
        .version("0.1.0")
        .author("Anomaly")
        .about("MTPScript CLI tool")
        .subcommand(
            Command::new("compile")
                .about("Compile MTPScript to JS")
                .arg(Arg::new("input").required(true))
                .arg(Arg::new("output").short('o').long("output").required(true)),
        )
        .subcommand(
            Command::new("run")
                .about("Run a snapshot")
                .arg(Arg::new("input").required(true)),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("compile", sub_matches)) => {
            let input = sub_matches.get_one::<String>("input").unwrap();
            let output = sub_matches.get_one::<String>("output").unwrap();
            println!("Compiling {} to {}", input, output);
            // TODO: implement compilation
        }
        Some(("run", sub_matches)) => {
            let input = sub_matches.get_one::<String>("input").unwrap();
            println!("Running {}", input);
            // TODO: implement running
        }
        _ => {
            println!("Use --help for usage");
        }
    }
}
