use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufRead, BufReader};
use zellij_tile::{shim::post_message_to_plugin, ZellijWorker};

#[derive(Default, Serialize, Deserialize)]
pub struct TaskDiscoveryWorker {}

impl<'de> ZellijWorker<'de> for TaskDiscoveryWorker {
    fn on_message(&mut self, _message: String, _payload: String) {
        let mut files: Vec<String> = vec![];
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
                        files.push(entry.path().to_str().unwrap()[5..].to_string());
                    }
                }
            }
        }
        eprintln!("files: {:?}", files);
        post_message_to_plugin("init".to_string(), serde_json::to_string(&files).unwrap());
    }
}
