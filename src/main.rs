use zellij_tile::prelude::*;

#[derive(Default)]
pub struct State {
    input: String,
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self) {
        subscribe(&[EventType::Key, EventType::Timer]);
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
                            let args = self.input.split(" ").collect::<Vec<&str>>();
                            open_command_pane(args[0], args[1..].to_vec());
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
            _ => false,
        }
    }

    fn render(&mut self, _rows: usize, _cols: usize) {
        println!("Task:\n{}", self.input);
    }
}
