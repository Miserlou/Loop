mod app;
mod io;
mod loop_iterator;
mod loop_step;
mod setup;
mod state;
mod util;

fn main() {
    use setup::{setup, Opt};
    use std::process;
    use structopt::StructOpt;

    let app = setup(Opt::from_args());

    let exit_code = match app {
        Ok((app, printer, exit_tasks)) => app.run(printer, exit_tasks),
        Err(err) => {
            if !err.message.is_empty() {
                eprintln!("{}", err.message);
            }
            err.exit_code
        }
    };

    process::exit(exit_code.into());
}
