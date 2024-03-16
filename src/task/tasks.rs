// Basic definition of a task. A task is just a unit of work that can be scheduled
// and executed by the scheduler.
use std::fs::File;

pub(crate) enum RunnableStatus {
    Waiting,
    Running,
    Finished,
    Killed,
    Error(String),
}

pub(crate) trait Runnable {
    fn start(&mut self, output: File, error: File) -> RunnableStatus;
    fn status(&self) -> RunnableStatus;
    fn kill(&mut self) -> RunnableStatus;
}
