use crate::task::tasks::{Runnable, RunnableStatus};
use std::cell::RefCell;
use std::fs::File;
use std::io::Result;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Child, Command};

pub(crate) struct BashScriptTask {
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
        } else if !script.is_file() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Script {} is not a file", script.display()),
            ));
        }
        let permissions = script.metadata()?.permissions();
        if permissions.mode() & 0o111 == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                format!(
                    "Script {} is not executable, has permissions {}",
                    script.display(),
                    permissions.mode()
                ),
            ));
        }

        Ok(BashScriptTask {
            script: script,
            proc: None,
        })
    }
}

impl Runnable for BashScriptTask {
    fn start(&mut self, output: File, error: File) -> RunnableStatus {
        let command = Command::new("bash")
            .arg(&self.script)
            .stdout(output)
            .stderr(error)
            .spawn();

        if let Err(e) = command {
            return RunnableStatus::Error(format!("Failed to start script: {}", e));
        }

        self.proc = Some(RefCell::new(command.unwrap()));
        RunnableStatus::Running
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

// Some tests
//
#[test]
pub(crate) fn test_missing_fails() {
    // get the path to the script in the tests folder relative to the crate root
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("scripts")
        .join("random_file.sh");
    let task = BashScriptTask::new(script);
    assert!(task.is_err());
}

#[test]
pub(crate) fn test_not_executable_fails() {
    // get the path to the script in the tests folder relative to the crate root
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("scripts")
        .join("not_executable.sh");
    let task = BashScriptTask::new(script);
    assert!(task.is_err());
}

#[test]
pub(crate) fn test_executable_passes() {
    // get the path to the script in the tests folder relative to the crate root
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("scripts")
        .join("test_loop.sh");

    let task = BashScriptTask::new(script);
    assert!(task.is_ok());
    assert!(matches!(task.unwrap().status(), RunnableStatus::Waiting));
}

#[test]
pub(crate) fn test_running() {
    // get the path to the script in the tests folder relative to the crate root
    use std::thread::sleep;
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("scripts")
        .join("test_loop.sh");
    let mut task = BashScriptTask::new(script).unwrap();
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("scripts")
        .join("test_loop.out");
    let error = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("scripts")
        .join("test_loop.err");

    let output_file = File::create(&output).expect("Failed to create output file");
    let error_file = File::create(&error).expect("Failed to create error file");

    let status = task.start(output_file, error_file);
    assert!(matches!(status, RunnableStatus::Running));
    // sleep for one second to allow the script to run
    sleep(std::time::Duration::from_secs(1));
    assert!(matches!(task.status(), RunnableStatus::Finished));

    // get the contents of the output
    let output = std::fs::read_to_string(output).expect("Failed to read output file");
    let expected = (1..11).map(|x| format!("{}\n", x)).collect::<String>();
    assert_eq!(output, expected);
}

#[test]
pub(crate) fn test_invalid_script() {
    // get the path to the script in the tests folder relative to the crate root
    use std::thread::sleep;
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("scripts")
        .join("test_invalid.sh");
    let mut task = BashScriptTask::new(script).unwrap();
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("scripts")
        .join("test_invalid.out");
    let error = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("scripts")
        .join("test_invalid.err");

    let output_file = File::create(&output).expect("Failed to create output file");
    let error_file = File::create(&error).expect("Failed to create error file");

    let status = task.start(output_file, error_file);
    sleep(std::time::Duration::from_secs(1));
    assert!(matches!(task.status(), RunnableStatus::Error(_)));

    // get the contents of the error
    let error = std::fs::read_to_string(error).expect("Failed to read error file");
    // check that the messgae contains "command not found"
    assert!(error.contains("command not found"));
}
