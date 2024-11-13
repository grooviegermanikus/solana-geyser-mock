use crate::geyser_plugin_util::load_plugin_from_config;
use std::path::Path;

mod geyser_plugin_util;

#[tokio::main]
async fn main() {
    // let mut geyser_plugin_manager = GeyserPluginManager::new();

    let (mut plugin, new_lib, new_config_file) = load_plugin_from_config(Path::new(
        "/Users/stefan/mango/projects/geyser-misc/config.json",
    ))
    .unwrap();
}
