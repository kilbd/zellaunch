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
    pub args: Vec<String>,
    pub command: String,
    pub runner: TaskRunner,
    pub task: String,
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
                    if lines
                        .next()
                        .is_some_and(|r| r.is_ok_and(|l| l.starts_with("#!")))
                    {
                        let rel_path = entry.path().to_str().unwrap()[6..].to_string();
                        options.push(TaskOption {
                            command: format!("./{}", &rel_path),
                            task: rel_path,
                            ..Default::default()
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

impl Default for TaskOption {
    fn default() -> Self {
        Self {
            args: vec![],
            command: "".to_string(),
            runner: TaskRunner::Shell,
            task: "".to_string(),
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
                    command: "npm".to_string(),
                    runner: TaskRunner::Npm,
                    task: task.to_string(),
                    args: vec!["run".to_string(), task.to_string()],
                });
            }
        }
    }
    options
}
