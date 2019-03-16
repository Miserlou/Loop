mod io;
mod loop_iterator;
mod loop_step;
mod run;
mod setup;
mod state;
mod util;

fn main() {
    use run::run;
    use setup::{setup, Opt};
    use std::process;
    use structopt::StructOpt;

    let app = setup(Opt::from_args());

    let exit_code = match app {
        Ok(app) => run(app),
        Err(err) => {
            if !err.message.is_empty() {
                eprintln!("{}", err.message);
            }
            err.exit_code
        }
    };

    process::exit(exit_code.into());
}
