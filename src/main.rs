extern crate macaroni_lang;
use macaroni_lang::*;

extern crate getopts;
use getopts::Options;

use std::io;
use std::io::prelude::*;
use std::env;
use std::fs::File;

fn main() {
    let mut mac = macaroni::Macaroni::new();
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("h", "help", "output this usage information");
    opts.optflag("v", "version", "output the current Macaroni version");
    opts.optflag("i", "interactive", "start an interactive REPL");
    opts.optopt("e", "evaluate", "takes one parameter, runs as Macaroni code",
        "[code]");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(_) => {
            usage(&program, opts);
            return;
        }
    };

    if matches.opt_present("h") {
        usage(&program, opts);
    } else if matches.opt_present("v") {
        println!("version 0.0.1 (alpha)");
    } else if matches.opt_present("i") {
        loop {
            print!(">>> ");
            io::stdout().flush().unwrap();
            let mut line = String::new();
            if io::stdin().read_line(&mut line).unwrap() == 0 { break; }

            println!(" => {:?}", mac.run(line));
        }
    } else if matches.opt_present("e") {
        mac.run(matches.opt_str("e").unwrap());
    } else {
        match matches.free.len() {
            0 => {
                let mut code = String::new();
                io::stdin().read_to_string(&mut code).unwrap();
                mac.run(code);
            },
            1 => {
                let mut code = String::new();
                match File::open(matches.free[0].clone()) {
                    Ok(mut f) => {
                        match f.read_to_string(&mut code) {
                            Ok(_) => { mac.run(code); },
                            Err(_) => file_err(&matches.free[0])
                        }
                    },
                    Err(_) => file_err(&matches.free[0])
                }
            },
            _ => usage(&program, opts)
        }
    }
}

fn usage(program: &str, opts: Options) {
    print!("{}", opts.usage(&format!("Usage: {} [filename] [options...]",
        program)));
}

fn file_err(name: &str) {
    println!("could not read file {}", name);
}
