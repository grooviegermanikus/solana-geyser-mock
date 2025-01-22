

## Run the mock server
```bash
export RUST_LOG=info,solana_geyser_mock=debug
cargo run --release -- --geyser-plugin-config config.json --account-bytes-per-slot 4000000 --compressibility 0.5
```

Parameters:
- `--account-bytes-per-slot` : Number of bytes to be generated per slot
- `--compressibility` : Compressibility (=inverse entropy) of the generated data

## Run the client (patched yellowstone)
(use version from branch update_client_to_test_performance on blockworks fork)
```bash
cargo run --release --bin client -- --endpoint http://host_ip:10000 subscribe --accounts
```

## Run the client (QUIC Geyser Plugin)
```bash
export RUST_LOG=info
cargo run --release --bin quic-plugin-tester-client -- -u 127.0.0.1:10900
```


## Example Output
Both test clients (yellowstone + quic) produce output similar to this:
```
[2024-11-27T16:39:26Z INFO  client]  DateTime : 5
[2024-11-27T16:39:26Z INFO  client]  Bytes Transfered : 72.316 Mbs/s
[2024-11-27T16:39:26Z INFO  client]  Accounts transfered size (uncompressed) : 72.316 Mbs
[2024-11-27T16:39:26Z INFO  client]  Accounts Notified : 13981
[2024-11-27T16:39:26Z INFO  client]  Slots Notified : 0
[2024-11-27T16:39:26Z INFO  client]  Blockmeta notified : 0
[2024-11-27T16:39:26Z INFO  client]  Transactions notified : 0
[2024-11-27T16:39:26Z INFO  client]  Blocks notified : 0
[2024-11-27T16:39:26Z INFO  client]  Average delay by accounts : 1.69 ms
[2024-11-27T16:39:26Z INFO  client]  Cluster Slots: 0, Account Slot: 42000435, Slot Notification slot: 0, BlockMeta slot: 0, Block slot: 0
```

You typically want to inspect `Bytes Ttransfered` and `Average delay by accounts`.

## TROUBLESHOOTING
### Stalling after Loading geyser plugin

```
2025-01-22T09:58:34.208419Z  INFO solana_geyser_mock: Loading geyser plugin from config: /home/groovie/work/geyser-grpc-proxy/config-mock-yellowstone.json
```

Solutions:
* set prometheus port to an unused one OR remove the "prometheus" field from the config file

### Segfault

Solution:
* make sure solana version matches (roughly); can use `cargo tree` for that
* check that rust version matches