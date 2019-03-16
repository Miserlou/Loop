mod io;
mod loop_iterator;
mod loop_step;
mod run;
mod setup;
mod state;
mod util;

fn main() {
    use io::ExitCode;
    use run::run;
    use setup::setup;
    use std::process;

    let app = setup();

    if app.is_no_command_supplied {
        eprintln!("No command supplied, exiting.");
        process::exit(ExitCode::MinorError.into());
    }

    let exit_code = run(app);

    process::exit(exit_code.into());
}
