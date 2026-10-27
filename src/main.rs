use std::process::ExitCode;

use seed_10::{Error, run};

fn main() -> ExitCode {
    match run(std::env::args().skip(1)) {
        Ok(out) => {
            println!("{out}");
            ExitCode::SUCCESS
        }
        Err(Error::Help) => {
            println!("{}", seed_10::usage());
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("{err}");
            eprintln!("{}", seed_10::usage());
            ExitCode::from(2)
        }
    }
}
