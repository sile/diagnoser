extern crate clap;
extern crate diagnoser;

use clap::App;
use clap::Arg;
use clap::SubCommand;
use diagnoser::beam::Module;

fn main() {
    let matches = App::new("diagnoser")
        .subcommand(SubCommand::with_name("dump-ast").arg(Arg::with_name("BEAM_FILE")
            .required(true)
            .index(1)))
        .get_matches();
    if let Some(matches) = matches.subcommand_matches("dump-ast") {
        let beam_file = matches.value_of("BEAM_FILE").unwrap();
        let module = Module::from_beam_file(beam_file)
            .expect(&format!("Can't parse file: {}", beam_file));
        println!("{:?}", module);
    } else {
        println!("{}", matches.usage());
        std::process::exit(1);
    }
}
