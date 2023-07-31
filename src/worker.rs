use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::{fmt, fmt::Display, fs};
use zellij_tile::{shim::post_message_to_plugin, ZellijWorker};

#[derive(Default, Serialize, Deserialize)]
pub struct TaskDiscoveryWorker {}

#[derive(Debug, Deserialize, Serialize)]
pub enum TaskRunner {
    Just,
    Npm,
    Shell,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TaskOption {
    pub task: String,
    pub runner: TaskRunner,
}

impl<'de> ZellijWorker<'de> for TaskDiscoveryWorker {
    fn on_message(&mut self, _message: String, _payload: String) {
        let mut files: Vec<TaskOption> = vec![];
        for f in ignore::Walk::new("/host") {
            if let Ok(entry) = f {
                if let Some(ft) = entry.file_type() {
                    if ft.is_dir() {
                        continue;
                    }
                    let file = match fs::File::open(entry.path()) {
                        Ok(file) => file,
                        Err(_) => continue,
                    };
                    // Check first line of file for shebang
                    let mut lines = BufReader::new(file).lines();
                    if lines.next().unwrap().unwrap().starts_with("#!") {
                        eprintln!("found shebang: {:?}", entry.path());
                        files.push(TaskOption {
                            task: entry.path().to_str().unwrap()[6..].to_string(),
                            runner: TaskRunner::Shell,
                        });
                    }
                }
            }
        }
        eprintln!("files: {:?}", files);
        post_message_to_plugin("init".to_string(), serde_json::to_string(&files).unwrap());
    }
}

impl Display for TaskOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.runner {
            TaskRunner::Just => write!(f, "{} (just)", self.task),
            TaskRunner::Npm => write!(f, "{} (npm)", self.task),
            TaskRunner::Shell => write!(f, "{}", self.task),
        }
    }
}
