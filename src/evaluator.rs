use subprocess::{Exec, ExitStatus, CaptureData};
use crate::ErrorCode;
use crate::Opt;

pub struct Response{
    pub options: Opt,
    pub result: CaptureData,
    pub last_result: Option<CaptureData>,
}

impl Response{
    pub fn should_stop(&self) -> bool {
        let result_checks = [
            stop_if_fail,
            stop_if_success,
            stop_if_contains,
            stop_if_match,
            stop_if_error
        ];

        result_checks.iter().any(|&fun| fun(&self.result, &self.options))
        || should_stop_on_comparison(&self.options, &self.last_result.as_ref(), &self.result)
    }
}

fn stop_if_contains(result: &CaptureData, options: &Opt) -> bool{
    // --until-contains
    let result_str = result.stdout_str();
    if let Some(string) = &options.until_contains {
        result_str.contains(string)
    } else {
        false
    }
}
fn stop_if_match(result: &CaptureData, options: &Opt) -> bool{
    // --until-match
    let result_str = result.stdout_str();
    if let Some(regex) = &options.until_match {
        regex.captures(&result_str).is_some()
    } else {
        false
    }
}
fn stop_if_error(result: &CaptureData, options: &Opt) -> bool{
    if let Some(error_code) = &options.until_error {
        match error_code {
            ErrorCode::Any => !result.exit_status.success(),
            ErrorCode::Code(code) => result.exit_status == ExitStatus::Exited(*code)
        }
    } else {
        false
    }
}

fn stop_if_success(result: &CaptureData, options: &Opt) -> bool{
    options.until_success && result.exit_status.success()
}
fn stop_if_fail(result: &CaptureData, options: &Opt) -> bool{
    options.until_fail && !(result.exit_status.success())
}

fn is_equivalent(previous: &Option<&CaptureData>, current: &CaptureData) -> bool{
    if let Some(prev) = previous{
        prev.stdout == current.stdout
    } else {
        false
    }
}

fn is_differrent(previous: &Option<&CaptureData>, current: &CaptureData) -> bool{
    if let Some(prev) = previous{
        prev.stdout != current.stdout
    } else {
        false
    }
}

pub fn should_stop_on_comparison(options: &Opt, previous: &Option<&CaptureData>, current: &CaptureData) -> bool {

    (options.until_same && is_equivalent(&previous.clone(), &current.clone()))
    || (options.until_changes && is_differrent(&previous.clone(), &current.clone()))
}

pub fn execute(opt: Opt) -> (Opt, CaptureData) {
    let result = Exec::shell(opt.input.join(""))
         .capture().unwrap();
    (opt, result)
}

