# Messenger

[Actix]: https://actix.rs/
[WebRTC]: https://webrtc.rs/

Messenger example in Rust with [Actix] and [WebRTC]

## Setup

```bash
# Serve with hot reload
$ cargo install cargo-watch
$ cargo watch -c -x run

# Build for production (Only Tier 1 targets recommended)
$ cargo build --release --target=<arch><sub>-<vendor>-<sys>-<abi>
```

## Environment Variables

| Variable         | Default Value | Description                                                                                                                   |
|------------------|:-------------:|-------------------------------------------------------------------------------------------------------------------------------|
| `RUST_LOG`       |       -       | `env_logger` output controller. Module declarations take comma separated entries formatted like `path::to::module=log_level`. |
| `MESSENGER_IP`   |  `127.0.0.1`  | IP address where the server will run.                                                                                         |
| `MESSENGER_PORT` |    `8080`     | Port that the server will listen to.                                                                                          |

## License

This project (rust-actix-webrtc-messenger) is available under the MIT license. See the [LICENSE](LICENSE) file for more info.
