## yosemite

[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE) [![Crates.io](https://img.shields.io/crates/v/yosemite.svg)](https://crates.io/crates/yosemite) [![docs.rs](https://img.shields.io/docsrs/yosemite.svg)](https://docs.rs/yosemite/latest/yosemite/)

`yosemite` is a [SAMv3](https://geti2p.net/en/docs/api/samv3) client library for interacting with the [I2P](https://geti2p.net/) network.

It provides both synchronous and asynchronous APIs which are configurable via `sync` and `async` feature flags, respectively.

### Supported features

* Streams
  * Forwarding
  * `Read`/`Write` for synchronous streams
  * `AsyncRead`/`AsyncWrite` for asynchronous streams

### Usage

`async` is enabled by default, giving access to asynchronous APIs:

```cargo
yosemite = "0.1.0"
```

`sync` enables synchronous APIs:

```cargo
yosemite = { version = "0.1.0", default-features = false, features = ["sync"] }
```

`sync` and `async` are mutually exclusive, only one or the other can be enabled. The APIs are otherwise the same but `async` requires blocking calls to `.await`.

See [`examples/`](https://github.com/altonen/yosemite/tree/master/examples) for instructions on how to use `yosemite`.

### Copying

MIT
