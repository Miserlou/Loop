mod io;
mod loop_iterator;
mod loop_step;
mod run;
mod setup;
mod state;
mod util;

fn main() {
    use run::run;
    use setup::setup;
    use state::ExitCode;
    use std::process;

    let m = setup();

    if m.is_no_command_supplied {
        eprintln!("No command supplied, exiting.");
        process::exit(ExitCode::MinorError.into());
    }

    let exit_code = run(m);

    process::exit(exit_code.into());
}
