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

use futures::{AsyncReadExt, AsyncWriteExt};
use tracing_subscriber::prelude::*;
use yosemite::{style::Stream, Session};

// Detached connection establishment:
//   cargo run --example connect_detached --features=async-extra

/// Event loop for the server.
///
/// Accepts an inbound stream, echoes back the message that was receives and sleeps.
async fn server_event_loop(mut session: Session<Stream>) {
    while let Ok(mut stream) = session.accept().await {
        let mut buffer = vec![0u8; 14];

        stream.read_exact(&mut buffer).await.unwrap();
        stream.write_all(&mut buffer).await.unwrap();

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .try_init()
        .unwrap();

    let server1 = Session::<Stream>::new(Default::default()).await.unwrap();
    let server2 = Session::<Stream>::new(Default::default()).await.unwrap();
    let destination1 = server1.destination().to_owned();
    let destination2 = server2.destination().to_owned();

    tokio::spawn(server_event_loop(server1));
    tokio::spawn(server_event_loop(server2));

    let mut client = Session::<Stream>::new(Default::default()).await.unwrap();

    for (i, destination) in [destination1, destination2].iter().enumerate() {
        let future = client.connect_detached(&destination);

        tokio::spawn(async move {
            let mut stream = future.await.unwrap();

            stream.write_all(format!("hello, world {i}").as_bytes()).await.unwrap();
            let mut buffer = vec![0u8; 14];
            stream.read_exact(&mut buffer).await.unwrap();

            tracing::info!("stream {i} read: {:?}", std::str::from_utf8(&buffer));
        });
    }
}
