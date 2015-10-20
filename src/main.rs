#[macro_use]
extern crate log;
extern crate env_logger;
extern crate bit_vec;
extern crate getopts;
extern crate rustbox;
extern crate time;

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
    let world = err!(parser::load_from_file(&cfg.map_filename));
    debug!("world: {}: {:?}", &cfg.map_filename, world);

    if let Err(e) = ui::run(world) {
        println!("{}", e);
        process::exit(1);
    }
}

struct Config {
    map_filename:   String,
}

impl Config {
    fn cmdline_usage(progname: &str, opts: getopts::Options) -> String {
        let brief = format!("Usage: {} [options] FILE", progname);
        format!("{}", opts.usage(&brief))
    }

    fn from_cmdline() -> Result<Config, String> {
        let mut args = env::args();
        let pname = args.next().unwrap();
        let args: Vec<String> = args.collect();

        let mut opts = getopts::Options::new();
        opts.optflag("h", "help", "print this help screen");
        let m = match opts.parse(&args) {
            Ok(m) => m,
            Err(e) => {
                return Err(format!("{}", e));
            }
        };
        if m.opt_present("h") {
            return Err(Config::cmdline_usage(&pname, opts));
        }
        if m.free.len() != 1 {
            return Err(format!("Exactly one argument expected!"));
        }
        Ok(Config {
            map_filename: m.free.into_iter().next().unwrap(),
        })
    }
}
