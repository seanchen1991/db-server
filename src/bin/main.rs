use std::process;

use db_server::server_init;

fn main() {
    if let Err(err) = server_init() {
        eprintln!("Error: {:?}", err);
        process::exit(1);
    }
}
