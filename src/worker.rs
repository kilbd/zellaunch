use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::Path;
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
        let mut options: Vec<TaskOption> = vec![];
        for f in ignore::Walk::new("/host") {
            if let Ok(entry) = f {
                if entry
                    .path()
                    .file_name()
                    .is_some_and(|f| f == "package.json")
                {
                    options.append(&mut get_npm_tasks(entry.path()));
                }
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
                        options.push(TaskOption {
                            task: entry.path().to_str().unwrap()[6..].to_string(),
                            runner: TaskRunner::Shell,
                        });
                    }
                }
            }
        }
        eprintln!("files: {:?}", options);
        post_message_to_plugin("init".to_string(), serde_json::to_string(&options).unwrap());
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

#[derive(Deserialize)]
struct PackageJson {
    scripts: HashMap<String, String>,
}

fn get_npm_tasks(path: &Path) -> Vec<TaskOption> {
    let mut options: Vec<TaskOption> = vec![];
    if let Ok(contents) = fs::read_to_string(path) {
        // If there are no scripts, parsing will fail
        if let Ok(json) = serde_json::from_str::<PackageJson>(&contents) {
            for task in json.scripts.keys() {
                options.push(TaskOption {
                    task: task.to_string(),
                    runner: TaskRunner::Npm,
                });
            }
        }
    }
    options
}
