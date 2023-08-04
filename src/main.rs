use zellij_tile::prelude::*;

use zellaunch::worker::{TaskDiscoveryWorker, TaskOption};

#[derive(Default)]
pub struct State {
    input: String,
    options: Vec<TaskOption>,
}

register_plugin!(State);
register_worker!(
    TaskDiscoveryWorker,
    task_discovery_worker,
    TASK_DISCOVERY_WORKER
);

impl ZellijPlugin for State {
    fn load(&mut self) {
        subscribe(&[EventType::Key, EventType::CustomMessage]);
        post_message_to("task_discovery", "init", "");
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::Key(key_press) => {
                match key_press {
                    //Key::PageDown => todo!(),
                    //Key::PageUp => todo!(),
                    //Key::Left => todo!(),
                    //Key::Down => todo!(),
                    //Key::Up => todo!(),
                    //Key::Right => todo!(),
                    //Key::Home => todo!(),
                    //Key::End => todo!(),
                    Key::Backspace => {
                        self.input.pop();
                        return true;
                    }
                    //Key::Delete => todo!(),
                    //Key::Insert => todo!(),
                    //Key::F(_) => todo!(),
                    Key::Char(c) => {
                        // Not sure why <Enter> isn't part of enum, but means
                        // we need to check for char here.
                        if c == '\n' {
                            launch_task(&self.input);
                            self.input.clear();
                            hide_self();
                            return true;
                        }
                        self.input.push(c);
                        return true;
                    }
                    //Key::Alt(_) => todo!(),
                    //Key::Ctrl(_) => todo!(),
                    //Key::BackTab => todo!(),
                    //Key::Null => todo!(),
                    Key::Esc => {
                        self.input.clear();
                        hide_self();
                        return true;
                    }
                    _ => false,
                }
            }
            Event::CustomMessage(_m, p) => {
                self.options = serde_json::from_str(&p).unwrap();
                eprintln!("Got options: {:?}", self.options);
                true
            }
            _ => false,
        }
    }

    fn render(&mut self, _rows: usize, _cols: usize) {
        for (index, opt) in self.options.iter().enumerate() {
            println!("{} - {}", index + 1, opt);
        }
        println!("Task:\n{}", self.input);
    }
}

fn launch_task(input: &String) {
    //TODO: Parse input to determine if running discovered task
    //or arbitrary command
    let args = input.split(" ").collect::<Vec<&str>>();
    open_command_pane(args[0], args[1..].to_vec());
}
