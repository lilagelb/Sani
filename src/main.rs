use clap::{arg, command};
use std::env;
use std::fs;
use std::process;

fn main() {
    let matches = command!()
        .arg(arg!(<file> "The file to render"))
        .get_matches();

    if let Some(file) = matches.get_one::<String>("file") {
        let Ok(contents) = fs::read_to_string(file) else {
            eprintln!("unable to read file `{file}`");
            process::exit(exitcode::UNAVAILABLE);
        };
        let parsed = sani::parse(&contents);
        let render = sani::render(parsed);

        println!("{render}");
    }
    // note: `clap` will handle the case that no input file was passed in
}
