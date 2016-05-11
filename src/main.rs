extern crate clap;

use clap::App;
use clap::Arg;
use clap::SubCommand;

fn main() {
    let matches = App::new("diagnoser")
                      .subcommand(SubCommand::with_name("dump-ast")
                                      .arg(Arg::with_name("BEAM_FILE")
                                               .required(true)
                                               .index(1)))
                      .get_matches();
    if let Some(matches) = matches.subcommand_matches("dump-ast") {
        unimplemented!()
    } else {
        println!("{}", matches.usage());
        std::process::exit(1);
    }
}
