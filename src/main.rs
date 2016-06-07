extern crate clap;
extern crate diagnoser;

use clap::App;
use clap::Arg;
use clap::SubCommand;
use diagnoser::beam::Module;
use diagnoser::typing;

fn main() {
    let matches = App::new("diagnoser")
        .subcommand(SubCommand::with_name("dump-ast").arg(Arg::with_name("BEAM_FILE")
            .required(true)
            .index(1)))
        .subcommand(SubCommand::with_name("analyze").arg(Arg::with_name("BEAM_FILE")
            .required(true)
            .takes_value(true)
            .multiple(true)))
        .get_matches();
    if let Some(matches) = matches.subcommand_matches("dump-ast") {
        let beam_file = matches.value_of("BEAM_FILE").unwrap();
        let module = Module::from_beam_file(beam_file)
            .expect(&format!("Can't parse file: {}", beam_file));
        println!("{:?}", module);
    } else if let Some(matches) = matches.subcommand_matches("analyze") {
        let mut env = typing::Env::new();
        for beam_file in matches.values_of("BEAM_FILE").unwrap() {
            println!("LOAD: {}", beam_file);
            let module = Module::from_beam_file(beam_file)
                .expect(&format!("Can't parse file: {}", beam_file));
            env.add_module(module);
            println!("   => types:{}, specs:{}", env.types.len(), env.specs.len());
        }
    } else {
        println!("{}", matches.usage());
        std::process::exit(1);
    }
}
