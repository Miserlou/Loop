mod app;
mod io;
mod loop_iterator;
mod loop_step;
mod setup;
mod state;

fn main() {
    use setup::{setup, Opt};
    use std::process;
    use structopt::StructOpt;

    let exit_code = setup(Opt::from_args())
        .map(|(app, setup_env, command, printer)| {
            let setup_env =
                &|item: Option<String>, actual_count: f64, count: f64| {
                    setup_env.run(item, actual_count, count)
                };
            app.run(setup_env, &|| command.run(), printer).exit_code
        })
        .unwrap_or_else(|err| {
            if !err.message.is_empty() {
                eprintln!("{}", err.message);
            }
            err.exit_code
        })
        .into();

    process::exit(exit_code);
}
