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

    let app = setup(Opt::from_args());

    let exit_code = match app {
        Ok((app, printer, exit_tasks, setup_env, shell_command)) => {
            run_app(app, printer, exit_tasks, setup_env, shell_command)
        }
        Err(err) => {
            if !err.message.is_empty() {
                eprintln!("{}", err.message);
            }
            err.exit_code
        }
    };

    process::exit(exit_code.into());
}

fn run_app(
    app: App,
    printer: Printer,
    exit_tasks: ExitTasks,
    setup_env: SetupEnv,
    shell_command: ShellCommand,
) -> ExitCode {
    use crate::io::ExitCode;
    use crate::state::{State, Summary};
    use std::fs::File;

    let command = |state: State| -> (ExitCode, State) { shell_command.run(state) };

    let printer = |stdout: &str, state: State| -> State { printer.print(stdout, state) };

    let exit_tasks = |summary: Summary, tmpfile: File| {
        exit_tasks.run(summary, tmpfile);
    };
    let setup_environment = |item: Option<String>, index: f64, actual_count: f64| {
        setup_env.run(item, index, actual_count)
    };

    app.run(&printer, &command, &exit_tasks, &setup_environment)
}
