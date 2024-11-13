use agave_geyser_plugin_interface::geyser_plugin_interface::{GeyserPlugin, GeyserPluginError, ReplicaAccountInfoV3};
use libloading::Library;
use solana_geyser_plugin_manager::geyser_plugin_manager::{
    GeyserPluginManagerError, LoadedGeyserPlugin,
};
use std::path::Path;
use std::sync::Arc;
use log::info;
use solana_program::clock::Epoch;
use solana_program::pubkey::Pubkey;
use solana_sdk::account::{AccountSharedData, ReadableAccount};
use solana_sdk::transaction::SanitizedTransaction;


#[derive(Debug)]
pub struct MockAccount {
    pub pubkey: Pubkey,
    pub lamports: u64,
    pub data: Vec<u8>,
    pub owner: Pubkey,
    pub executable: bool,
    pub rent_epoch: Epoch,
}

// see also GeyserPluginManager: load_plugin

pub fn setup_plugin(config_file: &Path) -> Result<Arc<LoadedGeyserPlugin>, GeyserPluginError> {

    let (mut new_plugin, new_lib, new_config_file) = load_plugin_from_config(config_file)
        .unwrap();

    setup_logger_for_plugin( new_plugin.as_ref())?;


    // Attempt to on_load with new plugin
    match new_plugin.on_load(config_file.as_os_str().to_str().unwrap(), true) {
        // On success, push plugin and library
        Ok(()) => {
            info!("Successfully loaded plugin: {}", new_plugin.name());
            // self.plugins.push(new_plugin);
            // self.libs.push(new_lib);
        }

        // On failure, return error
        Err(err) => {
            // return Err(jsonrpc_core::error::Error {
            //     code: ErrorCode::InvalidRequest,
            //     message: format!(
            //         "Failed to start new plugin (previous plugin was dropped!): {err}"
            //     ),
            //     data: None,
            // });
        }
    }

    Ok(Arc::new(new_plugin))
}

fn load_plugin_from_config(
    geyser_plugin_config_file: &Path,
) -> Result<(LoadedGeyserPlugin, Library, &str), GeyserPluginManagerError> {
    use std::{fs::File, io::Read, path::PathBuf};
    type PluginConstructor = unsafe fn() -> *mut dyn GeyserPlugin;
    use libloading::Symbol;

    let mut file = match File::open(geyser_plugin_config_file) {
        Ok(file) => file,
        Err(err) => {
            return Err(GeyserPluginManagerError::CannotOpenConfigFile(format!(
                "Failed to open the plugin config file {geyser_plugin_config_file:?}, error: {err:?}"
            )));
        }
    };

    let mut contents = String::new();
    if let Err(err) = file.read_to_string(&mut contents) {
        return Err(GeyserPluginManagerError::CannotReadConfigFile(format!(
            "Failed to read the plugin config file {geyser_plugin_config_file:?}, error: {err:?}"
        )));
    }

    let result: serde_json::Value = match json5::from_str(&contents) {
        Ok(value) => value,
        Err(err) => {
            return Err(GeyserPluginManagerError::InvalidConfigFileFormat(format!(
                "The config file {geyser_plugin_config_file:?} is not in a valid Json5 format, error: {err:?}"
            )));
        }
    };

    let libpath = result["libpath"]
        .as_str()
        .ok_or(GeyserPluginManagerError::LibPathNotSet)?;
    let mut libpath = PathBuf::from(libpath);
    if libpath.is_relative() {
        let config_dir = geyser_plugin_config_file.parent().ok_or_else(|| {
            GeyserPluginManagerError::CannotOpenConfigFile(format!(
                "Failed to resolve parent of {geyser_plugin_config_file:?}",
            ))
        })?;
        libpath = config_dir.join(libpath);
    }

    let plugin_name = result["name"].as_str().map(|s| s.to_owned());

    let config_file = geyser_plugin_config_file
        .as_os_str()
        .to_str()
        .ok_or(GeyserPluginManagerError::InvalidPluginPath)?;

    let (plugin, lib) = unsafe {
        let lib = Library::new(libpath)
            .map_err(|e| GeyserPluginManagerError::PluginLoadError(e.to_string()))?;
        let constructor: Symbol<PluginConstructor> = lib
            .get(b"_create_plugin")
            .map_err(|e| GeyserPluginManagerError::PluginLoadError(e.to_string()))?;
        let plugin_raw = constructor();
        (Box::from_raw(plugin_raw), lib)
    };
    Ok((
        LoadedGeyserPlugin::new(plugin, plugin_name),
        lib,
        config_file,
    ))
}

pub fn accountinfo_from_shared_account_data<'a>(
    account: &'a AccountSharedData,
    txn: &'a Option<&'a SanitizedTransaction>,
    pubkey: &'a Pubkey,
    write_version: u64,
) -> ReplicaAccountInfoV3<'a> {
    ReplicaAccountInfoV3 {
        pubkey: pubkey.as_ref(),
        lamports: account.lamports(),
        owner: account.owner().as_ref(),
        executable: account.executable(),
        rent_epoch: account.rent_epoch(),
        data: account.data(),
        write_version,
        txn: *txn,
    }
}

fn setup_logger_for_plugin(new_plugin: &dyn GeyserPlugin) -> Result<(), GeyserPluginError> {
    new_plugin
        .setup_logger(log::logger(), log::max_level())
}
