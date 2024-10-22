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

//! cargo run --example=client_server_async -- <host>

use futures::{AsyncReadExt, AsyncWriteExt, StreamExt};
use tracing_subscriber::prelude::*;
use yosemite::{Listener, Stream, StreamOptions};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .try_init()
        .unwrap();

    let mut listener = Listener::new(Default::default()).await.unwrap();
    let destination = listener.destination().to_owned();

    tokio::spawn(async move {
        while let Some(mut stream) = listener.next().await {
            let mut buffer = vec![0u8; 14];
            stream.read_exact(&mut buffer).await.unwrap();

            tracing::info!("listener read: {:?}", std::str::from_utf8(&buffer));

            stream.write_all("goodbye, world".as_bytes()).await.unwrap();

            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    });

    for i in 0..3 {
        let mut stream = Stream::new(destination.clone(), StreamOptions::default())
            .await
            .unwrap();

        stream
            .write_all(format!("hello, world {i}").as_bytes())
            .await
            .unwrap();

        let mut buffer = vec![0u8; 14];
        stream.read_exact(&mut buffer).await.unwrap();

        tracing::info!("stream {i} read: {:?}", std::str::from_utf8(&buffer));

        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    }
}
