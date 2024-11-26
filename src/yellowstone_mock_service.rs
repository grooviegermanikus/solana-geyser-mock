use bytes::{Bytes, BytesMut};
use rand::distributions::Standard;
use rand::{random, thread_rng, Rng, RngCore};
use solana_sdk::clock::UnixTimestamp;
use solana_sdk::pubkey::Pubkey;
// use solana_sdk::recent_blockhashes_account::update_account;
use std::ops::Add;
use std::path::Path;
use std::thread::{sleep, spawn};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use agave_geyser_plugin_interface::geyser_plugin_interface::ReplicaAccountInfoV3;
use libloading::Library;
use log::{debug, info};
use solana_sdk::account::{Account, AccountSharedData};
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::Instant;
use crate::geyser_plugin_util::MockAccount;

// - 20-80 MiB per Slot
// 4000 updates per Slot
pub async fn mainnet_traffic(geyser_channel: UnboundedSender<MockAccount>, bytes_per_slot: u64, compressibility: f64) {
    info!("Setup mainnet-like traffic source with {} bytes per slot and compressibility {}", bytes_per_slot, compressibility);
    let owner = Pubkey::new_unique();
    let account_pubkeys: Vec<Pubkey> = (0..100).map(|_| Pubkey::new_unique()).collect();

    for slot in 42_000_000.. {
        let slot_started_at = Instant::now();

        let sizes = vec![
            // mainnet distribution
            0, 8, 8, 165, 165, 165, 165, 11099, 11099, 11099, 11099, 11099, 11099,
            // shape with a lot larger sizes
            // 200000, 220000, 230000,
        ];
        // 10MB -> stream buffer size peaks at 30
        // 30MB -> stream buffer size peaks at 10000th and more
        // per slot
        let mut bytes_total = 0;

        let mut requested_sizes: Vec<u64> = Vec::new();

        for i in 0..99_999_999 {
            let data_size = sizes[i % sizes.len()];

            if bytes_total + data_size > bytes_per_slot {
                break;
            }

            requested_sizes.push(data_size);
            bytes_total += data_size;
        }

        debug!(
            "will send account updates for slot {} down the stream ({} bytes) in {} messages",
            slot,
            bytes_total,
            requested_sizes.len()
        );

        // distribute data over the slot duration (400ms) but leave some space
        let avg_delay = 0.350 / requested_sizes.len() as f64;

        for (i, data_bytes) in requested_sizes.into_iter().enumerate() {
            let next_message_at =
                slot_started_at.add(Duration::from_secs_f64(avg_delay * i as f64));

            let account_build_started_at = Instant::now();
            let mut data = vec![0; data_bytes as usize];
            let entropy_bytes = (data_bytes as f64 * (1.0 - compressibility)) as usize;
            assert!(entropy_bytes <= data_bytes as usize, "entropy_bytes overflow");
            fill_with_xor_prng(&mut data[0..entropy_bytes]);
            let data = data.to_vec();

            // using random slows down everything - could be the generator PRNG or the entropy preventing compression
            // let data: Vec<u8> = thread_rng().sample_iter(&Standard).take(data_bytes).collect();

            let account_pubkey = account_pubkeys[i % sizes.len()];

            // let ua = Account {
            //     lamports: 0,
            //     data,
            //     owner,
            //     executable: false,
            //     rent_epoch: 0,
            // };

            let epoch_us = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_micros() as u64;

            let account = MockAccount {
                slot,
                pubkey: account_pubkey,
                lamports: epoch_us,
                data,
                owner,
                executable: false,
                rent_epoch: 0,
            };

            // let shared = AccountSharedData::from(ua);

            // let v3 = ReplicaAccountInfoV3 {
            //     pubkey: account_pubkey.as_ref().clone(),
            //     lamports: 0,
            //     owner: owner.as_ref(),
            //     executable: false,
            //     rent_epoch: 0,
            //     data: data.as_ref(),
            //     write_version: 0,
            //     txn: None,
            // };

            // let update_account = MessageAccount {
            //     account: MessageAccountInfo {
            //         pubkey: account_pubkey,
            //         lamports: 0,
            //         owner,
            //         executable: false,
            //         rent_epoch: 0,
            //         data,
            //         write_version: 4321,
            //         txn_signature: None,
            //     },
            //     slot,
            //     is_startup: false,
            // };

            // let elapsed = account_build_started_at.elapsed();
            // // 0.25us
            // debug!("time consumed to build fake account message: {:.2}us", elapsed.as_secs_f64() * 1_000_000.0);


            geyser_channel
                .send(account)
                .expect("channel was closed");

            tokio::time::sleep_until(next_message_at).await;
        }

        let block_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as UnixTimestamp;

        // grpc_channel
        //     .send(Message::Slot(MessageSlot {
        //         slot,
        //         parent: Some(slot - 1),
        //         status: CommitmentLevel::Processed,
        //     }))
        //     .expect("channel was closed");

        // geyser_channel
        //     .send(Message::BlockMeta(MessageBlockMeta {
        //         parent_slot: slot - 1,
        //         slot,
        //         parent_blockhash: "nohash".to_string(),
        //         blockhash: "nohash".to_string(),
        //         rewards: vec![],
        //         block_time: Some(block_time),
        //         block_height: None,
        //         executed_transaction_count: 0,
        //         entries_count: 0,
        //     }))
        //     .expect("channel was closed");

        tokio::time::sleep_until(slot_started_at.add(Duration::from_millis(400))).await;
    }
}

pub async fn helloworld_traffic(grpc_channel: UnboundedSender<MockAccount>) {
    loop {
        let account_mock = MockAccount {
                slot: 999_888,
                pubkey: Pubkey::new_unique(),
                lamports: 0,
                owner: Pubkey::new_unique(),
                executable: false,
                rent_epoch: 0,
                data: vec![1, 2, 3],
            };


        grpc_channel
            .send(account_mock)
            .expect("send");
        debug!("sent account update down the stream");

        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

fn fill_with_xor_prng(binary: &mut [u8]) {
    let seed_n = binary.len();
    let mut state: u32 = 0xdeadbeef;
    for i_word in 0..seed_n / 4 {
        let mut x = state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        state = x;

        binary[i_word * 4 + 0] = (x >> 0) as u8;
        binary[i_word * 4 + 1] = (x >> 8) as u8;
        binary[i_word * 4 + 2] = (x >> 16) as u8;
        binary[i_word * 4 + 3] = (x >> 24) as u8;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_with_xor_prng_test() {
        let mut data_full_entropy = vec![0; 1000];
        fill_with_xor_prng(&mut data_full_entropy);
        let compressed_size = lz4_flex::compress(&data_full_entropy).len();
        assert_eq!(compressed_size, 1005);
    }

    #[test]
    fn fill_with_xor_prng_lowentropy_test() {
        let mut data_low_entropy = vec![0; 1000];
        fill_with_xor_prng(&mut data_low_entropy[0..200]);
        let compressed_size = lz4_flex::compress(&data_low_entropy).len();
        assert_eq!(compressed_size, 219);
    }
}
