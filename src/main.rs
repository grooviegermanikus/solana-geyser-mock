use crate::geyser_plugin_util::load_plugin_from_config;
use std::path::Path;
use agave_geyser_plugin_interface::geyser_plugin_interface::{ReplicaAccountInfoV3, ReplicaAccountInfoVersions};
use clap::Parser;
use log::info;
use solana_program::pubkey::Pubkey;
use solana_sdk::clock::Slot;
use tracing_subscriber::EnvFilter;

mod geyser_plugin_util;


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    // point to config.json
    #[arg(long)]
    pub geyser_plugin_config: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let args = Args::parse();

    info!("Loading geyser plugin from config: {}", args.geyser_plugin_config);
    let config_file = Path::new(&args.geyser_plugin_config);
    assert!(config_file.exists(), "Config file must exist");

    let (mut plugin, new_lib, new_config_file) = load_plugin_from_config(config_file)
    .unwrap();

    // TODO adopt validator code

    let owner_pubkey = Pubkey::new_unique();
    let account_pubkey = Pubkey::new_unique();

    let v3 = ReplicaAccountInfoV3 {
        pubkey: account_pubkey.as_ref(),
        lamports: 3333,
        owner: owner_pubkey.as_ref(),
        executable: false,
        rent_epoch: 0,
        data: &[23,12,12,12],
        write_version: 999999,
        txn: None,
    };

    let account = ReplicaAccountInfoVersions::V0_0_3(&v3);

    let slot = 990000000 as Slot;
    plugin.update_account(account, slot, false).unwrap()

    std::thread::spawn(move || {
        loop {


            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }).join().unwrap();
}
