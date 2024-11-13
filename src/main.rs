use crate::geyser_plugin_util::{accountinfo_from_shared_account_data, setup_plugin};
use std::path::Path;
use agave_geyser_plugin_interface::geyser_plugin_interface::{ReplicaAccountInfoV3, ReplicaAccountInfoVersions};
use clap::Parser;
use log::{info, warn};
use solana_program::pubkey::Pubkey;
use solana_sdk::account::{AccountSharedData, ReadableAccount};
use solana_sdk::clock::Slot;
use tracing_subscriber::EnvFilter;

mod geyser_plugin_util;
mod yellowstone_mock_service;


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

    let plugin = setup_plugin(config_file.as_ref())
    .unwrap();


    // TODO adopt validator code

    let (channel_tx, mut channel_rx) = tokio::sync::mpsc::unbounded_channel();

    // tokio::task::spawn(yellowstone_mock_service::helloworld_traffic(channel_tx));
    tokio::task::spawn(yellowstone_mock_service::mainnet_traffic(channel_tx));


    std::thread::spawn(move || {
        'recv_loop: loop {
            match channel_rx.blocking_recv() {
                Some(mock_account) => {
                    info!("channel_rx.blocking_recv() returned {:?}", mock_account);
                    let slot = 999999;

                    // let v3 = accountinfo_from_shared_account_data(&account, &None, account_pubkey, 0);


                    let account_v3 = ReplicaAccountInfoV3 {
                        pubkey: mock_account.pubkey.as_ref(),
                        lamports: mock_account.lamports,
                        owner: mock_account.owner.as_ref(),
                        executable: mock_account.executable,
                        rent_epoch: mock_account.rent_epoch,
                        data: mock_account.data.as_ref(),
                        write_version: 999999,
                        txn: None,
                    };

                    let account = ReplicaAccountInfoVersions::V0_0_3(&account_v3);
                    plugin.update_account(account, slot, false).unwrap();

                }
                None => {
                    warn!("channel closed - shutting down");
                    break 'recv_loop;
                }
            }



            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }).join().unwrap();
}
