use zellij_tile::prelude as zellij;

use crate::{left, right};

const REFRESH_INTERVAL_SECONDS: f64 = 1.0;

#[derive(Default)]
pub struct State {
    errors: Vec<std::io::Error>,
    print_errors: bool,
    pending_events: std::collections::VecDeque<zellij::Event>,
    got_permissions: bool,
}

impl State {
    fn handle_event(event: zellij::Event) -> bool {
        match event {
            zellij::Event::Timer(_) => {
                zellij::set_timeout(REFRESH_INTERVAL_SECONDS);
                drop(event);
                true
            }
            _ => false,
        }
    }
}

impl zellij::ZellijPlugin for State {
    fn load(&mut self, _configuration: std::collections::BTreeMap<String, String>) {
        self.errors = Vec::new();
        self.print_errors = false;
        self.pending_events = std::collections::VecDeque::new();
        self.got_permissions = false;

        zellij::request_permission(&[
            zellij::PermissionType::ReadApplicationState,
            zellij::PermissionType::ChangeApplicationState,
        ]);

        zellij::subscribe(&[
            zellij::EventType::Mouse,
            zellij::EventType::ModeUpdate,
            zellij::EventType::PaneUpdate,
            zellij::EventType::PermissionRequestResult,
            zellij::EventType::Timer,
            zellij::EventType::TabUpdate,
            zellij::EventType::SessionUpdate,
            zellij::EventType::CwdChanged,
        ]);

        zellij::set_timeout(REFRESH_INTERVAL_SECONDS);
    }

    fn update(&mut self, event: zellij::Event) -> bool {
        let mut result = false;

        if let zellij::Event::PermissionRequestResult(zellij::PermissionStatus::Granted) = event {
            self.got_permissions = true;

            while let Some(event) = self.pending_events.pop_front() {
                result |= Self::handle_event(event);
            }
        }

        if !self.got_permissions {
            self.pending_events.push_back(event);
            return false;
        }

        result || Self::handle_event(event)
    }

    fn pipe(&mut self, _pipe_message: zellij::PipeMessage) -> bool {
        false
    }

    fn render(&mut self, _rows: usize, cols: usize) {
        if !self.got_permissions {
            println!("Awaiting permissions");
            return;
        }

        if self.print_errors {
            for err in &self.errors {
                println!("{err:?}");
            }
            return;
        }

        let (left, left_len) = {
            let mut buffer = Vec::with_capacity(cols);

            if let Err(error) = left::render(&mut buffer) {
                self.errors.push(error);
            }

            match String::from_utf8(buffer) {
                Ok(string) => {
                    let len = count_cols(&string);
                    (string, len)
                }
                Err(error) => {
                    self.errors.push(std::io::Error::other(error));
                    (String::new(), 0)
                }
            }
        };

        let right = {
            let mut buffer = Vec::with_capacity(cols - left_len);

            if let Err(error) = right::render(&mut buffer) {
                self.errors.push(error);
            }

            match String::from_utf8(buffer) {
                Ok(string) => string,
                Err(error) => {
                    self.errors.push(std::io::Error::other(error));
                    String::new()
                }
            }
        };

        println!("{left}{right:00$}", cols - left_len);
    }
}

fn count_cols(string: &str) -> usize {
    unicode_segmentation::UnicodeSegmentation::graphemes(string, true)
        .scan((None, 0_usize), |(escaped, count), curr| {
            match *escaped {
                Some(true) => {
                    if curr == "m" {
                        *escaped = None;
                    }
                }
                Some(false) => {
                    if curr == "[" {
                        *escaped = Some(true);
                    } else {
                        *escaped = None;
                    }
                }
                None => {
                    if curr == "" {
                        *escaped = Some(false);
                    } else {
                        *count += 1;
                    }
                }
            }
            Some(*count)
        })
        .last()
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_capture() {
        let string = "bloink[31myo[43m[32mboom";
        assert_eq!(count_cols(string), 12);
    }
}
