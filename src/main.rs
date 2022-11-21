extern crate clap;
extern crate diagnoser;

use clap::Parser;
use diagnoser::env::Env;
use diagnoser::module::Module;

#[derive(Parser)]
enum Args {
    DumpAst { beam_file: String },
    Analyze { beam_file: Vec<String> },
}

fn main() {
    let args = Args::parse();
    match args {
        Args::DumpAst { beam_file } => {
            let module = Module::from_beam_file(&beam_file)
                .expect(&format!("Can't parse file: {}", beam_file));
            println!("{:?}", module);
        }
        Args::Analyze { beam_file } => {
            let mut env = Env::new();
            for beam_file in &beam_file {
                println!("LOAD: {}", beam_file);
                let module = Module::from_beam_file(beam_file)
                    .expect(&format!("Can't parse file: {}", beam_file));
                env.add_module(module);
            }
        }
    }
}
