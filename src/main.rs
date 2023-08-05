use std::fmt::Display;

use zellij_tile::prelude::*;

use zellaunch::{
    parser::InputIterator,
    worker::{TaskDiscoveryWorker, TaskOption},
};

#[derive(Default)]
pub struct State {
    input: Input,
    options: Vec<TaskOption>,
}

pub struct Input {
    text: String,
    cursor_position: usize,
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
                            self.launch_tasks();
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

impl State {
    fn launch_tasks(&self) {
        self.input
            .iter()
            .map(|mut t| {
                if let Some(i) = t.index {
                    let chosen = self.options.get(i - 1).unwrap();
                    t.command = Some(&chosen.command);
                    let mut args: Vec<&str> = chosen.args.iter().map(String::as_str).collect();
                    args.extend(t.args);
                    t.args = args;
                }
                t
            })
            .for_each(|t| open_command_pane(t.command.unwrap(), t.args));
    }
}

impl Input {
    pub fn push(&mut self, c: char) {
        // `push` should be more efficient than `insert` when appending
        if self.cursor_position == self.text.len() {
            self.text.push(c);
        } else {
            self.text.insert(self.cursor_position, c);
        }
        self.cursor_position += 1;
    }

    pub fn pop(&mut self) -> Option<char> {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            Some(self.text.remove(self.cursor_position))
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor_position = 0;
    }

    pub fn move_cursor(&mut self, delta: isize) {
        let new_position = (self.cursor_position as isize) + delta;
        if new_position >= 0 && new_position <= (self.text.len() as isize) {
            self.cursor_position = new_position as usize;
        }
    }

    pub fn iter(&self) -> InputIterator {
        InputIterator::new(self.text.as_str())
    }
}

impl Default for Input {
    fn default() -> Self {
        Self {
            text: String::new(),
            cursor_position: 0,
        }
    }
}

impl Display for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.text)
    }
}
