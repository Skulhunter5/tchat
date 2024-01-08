use std::{env, process::ExitCode};

use tchat::Client;

fn main() -> ExitCode {
    let mut args = env::args();
    let _program = args.next().expect("program");
    let server_address = match args.next() {
        Some(addr) => addr,
        None => {
            eprintln!("Please specify the ip of the server to connect to");
            return ExitCode::from(1);
        }
    };
    let client = Client::new(server_address);
    match client.run() {
        Ok(()) => {}
        Err(e) => {
            eprintln!("Error: {e}");
            return ExitCode::from(1);
        }
    }

    ExitCode::SUCCESS
}
