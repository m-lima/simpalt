use zellij_tile::ZellijPlugin;
use zellij_tile::shim::report_panic;

zellij_tile::register_plugin!(plugin::State);

type Result<T = ()> = std::io::Result<T>;

mod left;
mod plugin;
mod right;
