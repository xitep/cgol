#[macro_use]
extern crate log;
extern crate env_logger;
extern crate bit_vec;
extern crate getopts;
extern crate rustbox;
extern crate time;
extern crate rand;

use std::env;
use std::process;

mod world;
mod parser;
mod ui;

fn main() {
    env_logger::init().unwrap();

    macro_rules! err {
        ($expr:expr) => {
            match $expr {
                Ok(v) => v,
                Err(e) => {
                    println!("{}", e);
                    process::exit(1);
                }
            }
        }
    }

    let cfg = err!(Config::from_cmdline());
    let world = match cfg.map_filename.as_ref() {
        None => None,
        Some(f) => Some(err!(parser::load_from_file(f))),
    };

    if let Err(e) = ui::run(world) {
        println!("{}", e);
        process::exit(1);
    }
}

struct Config {
    map_filename: Option<String>,
}

impl Config {
    fn cmdline_usage(progname: &str, opts: getopts::Options) -> String {
        let brief = format!("Usage: {} [options]", progname);
        format!("{}", opts.usage(&brief))
    }

    fn from_cmdline() -> Result<Config, String> {
        let mut args = env::args();
        let pname = args.next().unwrap();
        let args: Vec<String> = args.collect();

        let mut opts = getopts::Options::new();
        opts.optflag("h", "help", "print this help screen");
        opts.optopt("f", "file", "load map from FILE", "FILE");
        let m = match opts.parse(&args) {
            Ok(m) => m,
            Err(e) => {
                return Err(format!("{}", e));
            }
        };
        if m.opt_present("h") {
            return Err(Config::cmdline_usage(&pname, opts));
        }
        if !m.free.is_empty() {
            return Err("No arguments expected!".to_owned());
        }
        Ok(Config { map_filename: m.opt_str("file") })
    }
}
