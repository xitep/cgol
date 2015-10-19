#[macro_use]
extern crate log;
extern crate env_logger;
extern crate bit_vec;

use std::env;

mod world;
mod parser;

fn main() {
    env_logger::init().unwrap();

    for arg in env::args().skip(1) {
        match parser::load_from_file(&arg) {
            Err(e) => {
                println!("{}", e);
                return;
            }
            Ok(mut w) => {
                debug!("{}: {:?}", arg, w);
                for i in 0..10 {
                    println!("arg ({}):\n{}", i, w);
                    w.advance_generation();
                }
            }
        }
    }
}
