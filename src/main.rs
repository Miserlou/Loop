mod app;
mod io;
mod loop_iterator;
mod loop_step;
mod setup;
mod state;
mod util;

use app::App;
use io::{ExitCode, ExitTasks, Printer, SetupEnv, ShellCommand};

fn main() {
    use setup::{setup, Opt};
    use std::process;
    use structopt::StructOpt;

    let exit_code = setup(Opt::from_args())
        .map(|(app, printer, exit_tasks, setup_env, shell_command)| {
            run_app(app, printer, exit_tasks, setup_env, shell_command)
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

fn run_app(
    app: App,
    printer: Printer,
    exit_tasks: ExitTasks,
    setup_env: SetupEnv,
    shell_command: ShellCommand,
) -> ExitCode {
    use crate::state::{State, Summary};
    use std::fs::File;

    let command = &|state: State| -> (ExitCode, State) { shell_command.run(state) };

    let printer = &|stdout: &str, state: State| -> State { printer.print(stdout, state) };

    let exit_tasks = &|summary: Summary, tmpfile: File| {
        exit_tasks.run(summary, tmpfile);
    };
    let setup_environment = &|item: Option<String>, actual_count: f64, count: f64| {
        setup_env.run(item, actual_count, count)
    };

    app.run(printer, command, exit_tasks, setup_environment)
}
