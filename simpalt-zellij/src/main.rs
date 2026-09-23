zellij_tile::register_plugin!(State);

#[derive(Default)]
struct State {}

impl zellij_tile::ZellijPlugin for State {
    fn load(&mut self, configuration: std::collections::BTreeMap<String, String>) {}

    fn update(&mut self, event: zellij_tile::prelude::Event) -> bool {
        false
    }

    fn pipe(&mut self, pipe_message: zellij_tile::prelude::PipeMessage) -> bool {
        false
    }

    fn render(&mut self, rows: usize, cols: usize) {}
}
