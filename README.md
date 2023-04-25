# m0b0tbot

`mobot` is a telegram chat bot written in Rust. Uses a native implementation of the
Telegram Bot API.

# Usage

Set your telegram API token and then run `bin/hello/rs`.

```
export TELEGRAM_TOKEN=...

RUST_LOG=debug cargo run hello
```

## Dependencies

Need OpenSSL and pkg-config.

```
sudo apt-get install pkg-config libssl-dev
```

## TODO

-   [ ] Process handler return actions
-   [ ] Handler stack
