## yosemite

[![License](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/altonen/yosemite/blob/master/LICENSE) [![Crates.io](https://img.shields.io/crates/v/yosemite.svg)](https://crates.io/crates/yosemite) [![docs.rs](https://img.shields.io/docsrs/yosemite.svg)](https://docs.rs/yosemite/latest/yosemite/)

`yosemite` is a [SAMv3](https://geti2p.net/en/docs/api/samv3) client library for interacting with the [I2P](https://geti2p.net/) network.

It provides synchronous and asynchronous APIs and supports both [`tokio`](https://docs.rs/tokio/latest/tokio/) and [`smol`](https://docs.rs/smol/latest/smol/).

### Supported features

* Streams
  * Forwarding
  * `Read`/`Write` for synchronous streams
  * [`AsyncRead`](https://docs.rs/tokio/latest/tokio/io/trait.AsyncRead.html)/[`AsyncWrite`](https://docs.rs/tokio/latest/tokio/io/trait.AsyncWrite.html) for `tokio` streams
  * [`AsyncRead`](https://docs.rs/smol/latest/smol/struct.Async.html#impl-AsyncRead-for-Async%3CT%3E)/[`AsyncWrite`](https://docs.rs/smol/latest/smol/struct.Async.html#impl-AsyncWrite-for-Async%3CT%3E) for `smol` streams
* Datagrams
  * Repliable
  * Anonymous
* Primary sessions

### Usage

`tokio` is enabled by default:

```toml
yosemite = "0.4.2"
```

`sync` enables synchronous APIs:

```toml
yosemite = { version = "0.4.2", default-features = false, features = ["sync"] }
```

`smol` enables asynchronous APIs implemented with [`smol`](https://docs.rs/smol/latest/smol/):

```toml
yosemite = { version = "0.4.2", default-features = false, features = ["smol"] }
```

`tokio`, `smol`, and `sync` are all mutually exclusive and only one them can be enabled. The APIs are otherwise the same but `tokio` and `smol` require blocking calls to `.await`.

#### Example usage of the API:

```rust ignore
use tokio::io::AsyncReadExt;
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
