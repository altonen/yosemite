## yosemite

[![License](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/altonen/yosemite/blob/master/LICENSE) [![Crates.io](https://img.shields.io/crates/v/yosemite.svg)](https://crates.io/crates/yosemite) [![docs.rs](https://img.shields.io/docsrs/yosemite.svg)](https://docs.rs/yosemite/latest/yosemite/)

`yosemite` is a [SAMv3](https://geti2p.net/en/docs/api/samv3) client library for interacting with the [I2P](https://geti2p.net/) network.

It provides both synchronous and asynchronous APIs which are configurable via `sync` and `async` feature flags, respectively.

### Supported features

* Streams
  * Forwarding
  * `Read`/`Write` for synchronous streams
  * `AsyncRead`/`AsyncWrite` for asynchronous streams
* Datagrams
  * Repliable
  * Anonymous

### Usage

`async` is enabled by default, giving access to asynchronous APIs:

```toml
yosemite = "0.2.0"
```

`sync` enables synchronous APIs:

```toml
yosemite = { version = "0.2.0", default-features = false, features = ["sync"] }
```

`sync` and `async` are mutually exclusive, only one or the other can be enabled. The APIs are otherwise the same but `async` requires blocking calls to `.await`.

#### Example usage of the API:

```rust no_run
use futures::AsyncReadExt;
use yosemite::{style::Stream, Session};

#[tokio::main]
async fn main() -> yosemite::Result<()> {
    let mut session = Session::<Stream>::new(Default::default()).await?;

    while let Ok(mut stream) = session.accept().await {
        println!("{} connected", stream.remote_destination());

        tokio::spawn(async move {
            let mut buffer = vec![0u8; 512];

            while let Ok(nread) = stream.read(&mut buffer).await {
                println!(
                    "client sent: {:?}",
                    std::str::from_utf8(&buffer[..nread])
                );
            }
        });
    }

    Ok(())
}
```

See [`examples`](https://github.com/altonen/yosemite/tree/master/examples) for instructions on how to use `yosemite`.

### Copying

MIT
