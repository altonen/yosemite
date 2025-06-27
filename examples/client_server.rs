// Permission is hereby granted, free of charge, to any person obtaining a
// copy of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
// OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use tracing_subscriber::prelude::*;

// Asynchronous client-server:
//    cargo run --example client_server
//
// Synchronous client-server:
//    cargo run --example client_server --no-default-features --features sync

#[cfg(all(feature = "tokio", not(feature = "sync")))]
#[tokio::main]
async fn main() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use yosemite::{style::Stream, Session};

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .try_init()
        .unwrap();

    let mut session = Session::<Stream>::new(Default::default()).await.unwrap();
    let destination = session.destination().to_owned();

    tokio::spawn(async move {
        while let Ok(mut stream) = session.accept().await {
            let mut buffer = vec![0u8; 14];

            stream.read_exact(&mut buffer).await.unwrap();
            stream.write_all(&mut buffer).await.unwrap();

            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    });

    for i in 0..3 {
        let mut session = Session::new(Default::default()).await.unwrap();
        let mut stream = session.connect(&destination).await.unwrap();

        stream.write_all(format!("hello, world {i}").as_bytes()).await.unwrap();

        let mut buffer = vec![0u8; 14];
        stream.read_exact(&mut buffer).await.unwrap();

        tracing::info!("stream {i} read: {:?}", std::str::from_utf8(&buffer));
    }
}

#[cfg(all(feature = "sync", not(feature = "tokio")))]
fn main() {
    use std::io::{Read, Write};
    use yosemite::{style::Stream, Session};

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .try_init()
        .unwrap();

    let mut session = Session::<Stream>::new(Default::default()).unwrap();
    let destination = session.destination().to_owned();

    std::thread::spawn(move || {
        while let Ok(mut stream) = session.accept() {
            let mut buffer = vec![0u8; 14];

            stream.read_exact(&mut buffer).unwrap();
            stream.write_all(&mut buffer).unwrap();

            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });

    for i in 0..3 {
        let mut session = Session::new(Default::default()).unwrap();
        let mut stream = session.connect(&destination).unwrap();

        stream.write_all(format!("hello, world {i}").as_bytes()).unwrap();

        let mut buffer = vec![0u8; 14];
        stream.read_exact(&mut buffer).unwrap();

        tracing::info!("stream {i} read: {:?}", std::str::from_utf8(&buffer));
    }
}
