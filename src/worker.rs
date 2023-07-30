use serde::{Deserialize, Serialize};
use zellij_tile::{shim::post_message_to_plugin, ZellijWorker};

#[derive(Default, Serialize, Deserialize)]
pub struct TaskDiscoveryWorker {}

impl<'de> ZellijWorker<'de> for TaskDiscoveryWorker {
    fn on_message(&mut self, _message: String, _payload: String) {
        let mut files: Vec<String> = vec![];
        for f in ignore::Walk::new("/host") {
            if let Ok(entry) = f {
                files.push(entry.path().to_str().unwrap().to_string());
            }
        }
        eprintln!("files: {:?}", files);
        post_message_to_plugin("init".to_string(), serde_json::to_string(&files).unwrap());
    }
}
