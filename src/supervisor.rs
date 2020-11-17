use std::sync::mpsc::{self, Sender, Receiver, TryRecvError};
use std::thread;
use std::collections::HashMap;
use subprocess::CaptureData;
pub use crate::Opt;
pub use crate::evaluator::{execute, Response};


struct Task{
    thread_index: usize,
    options: Opt,
    result: CaptureData,
}

pub fn clone_data(data: &CaptureData) -> CaptureData{
    match data{
        CaptureData{exit_status, stdout: std_out, stderr: std_err} =>{
            CaptureData{exit_status: *exit_status, stdout: std_out.to_vec(), stderr: std_err.to_vec()}
        }
    }
}

pub fn supervisor(requests: Receiver<(usize, Opt)>, responses: Sender<Response>) -> (){
    let mut tasks = HashMap::new();
    let (task_finished_sender, task_finished_receiver) = mpsc::channel();
    let mut last_result: Option<CaptureData> = None;
    loop {
        match requests.try_recv(){
            Ok((index, opt)) => {
                let finished = task_finished_sender.clone();
                let handle = thread::spawn(move ||{
                    let (opt, result) = execute(opt);
                    let task = Task{
                        thread_index: index,
                        options: opt,
                        result: result
                    };
                    finished.send(task).unwrap();
                });
                tasks.insert(index, handle);
            },
            Err(TryRecvError::Empty) =>(),
            Err(TryRecvError::Disconnected) =>{
                panic!("thread disconected (requests)");
            },
        };

        match task_finished_receiver.try_recv(){
            Ok(Task{
                thread_index,
                options,
                result,
            }) => {
                let response = Response{
                    options: options,
                    result: clone_data(&result),
                    last_result: last_result,
                };
                responses.send(response).unwrap();
                last_result = Some(result);
                match tasks.remove(&thread_index) {
                    Some(handle) => {
                        handle.join().unwrap();
                        ()
                    },
                    None => (),
                };
                if tasks.is_empty(){
                    break;
                }
            },
            Err(TryRecvError::Empty) =>(),
            Err(TryRecvError::Disconnected) =>{
                panic!("thread disconected (finished_tasks)");
            },
        };
    }
}
