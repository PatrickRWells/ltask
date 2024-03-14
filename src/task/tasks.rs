// Basic definition of a task. A task is just a unit of work that can be scheduled
// and executed by the scheduler.
use chrono::DateTime;
use std::cell::RefCell;
use std::io::Result;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Child, Command};

enum RunnableStatus {
    Waiting,
    Running,
    Finished,
    Killed,
    Error(String),
}

trait Runnable {
    fn start(&mut self, log_file: &PathBuf);
    fn status(&self) -> RunnableStatus;
    fn kill(&mut self) -> RunnableStatus;
}

struct BashScriptTask {
    script: PathBuf,
    proc: Option<RefCell<Child>>,
}

impl BashScriptTask {
    fn new(script: PathBuf) -> Result<BashScriptTask> {
        // Check that the script actually exists
        // and is executable
        if !script.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Script {} does not exist", script.display()),
            ));
        }

        let permissions = script.metadata()?.permissions();
        if !permissions.mode() & 0o111 != 0o111 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                format!("Script {} is not executable", script.display()),
            ));
        }

        Ok(BashScriptTask {
            script: script,
            proc: None,
        })
    }
}

impl Runnable for BashScriptTask {
    fn start(&mut self, log_file: &PathBuf) {
        let command = Command::new("bash")
            .arg(&self.script)
            .arg("2>&1")
            .arg(">>")
            .arg(log_file)
            .spawn()
            .expect("Failed to start script");

        self.proc = Some(RefCell::new(command));
    }

    fn status(&self) -> RunnableStatus {
        match &self.proc {
            Some(proc) => {
                if let Some(status) = proc.borrow_mut().try_wait().expect("Failed to get status") {
                    if status.success() {
                        RunnableStatus::Finished
                    } else {
                        RunnableStatus::Error(format!("Script failed with status {}", status))
                    }
                } else {
                    RunnableStatus::Running
                }
            }
            None => RunnableStatus::Waiting,
        }
    }
    fn kill(&mut self) -> RunnableStatus {
        match &mut self.proc {
            Some(proc) => {
                let status = proc.borrow_mut().kill();
                if status.is_ok() {
                    RunnableStatus::Killed
                } else {
                    RunnableStatus::Error("Failed to kill script".to_string())
                }
            }
            None => RunnableStatus::Waiting,
        }
    }
}
