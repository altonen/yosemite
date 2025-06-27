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

use std::time::Duration;

// Asynchronous repliable datagrams:
//    cargo run --example repliable
//
// Synchronous repliable datagrams:
//    cargo run --example repliable --no-default-features --features sync

#[cfg(all(feature = "tokio", not(feature = "sync")))]
#[tokio::main]
async fn main() {
    use yosemite::{style::Repliable, Session};

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .try_init()
        .unwrap();

    let mut server = Session::<Repliable>::new(Default::default()).await.unwrap();
    let destination = server.destination().to_owned();

    tokio::spawn(async move {
        let mut buffer = [0u8; 1024];

        for _ in 0..3 {
            // read message from client and send it back to them
            let (nread, destination) = server.recv_from(&mut buffer).await.unwrap();
            server.send_to(&buffer[..nread], &destination).await.unwrap();
        }

        tokio::time::sleep(Duration::from_secs(5)).await;
    });

    let mut client = Session::<Repliable>::new(Default::default()).await.unwrap();
    let mut buffer = [0u8; 1024];

    for message in vec!["hello, world", "testing 123", "goodbye, world"] {
        // send message to server
        client.send_to(message.as_bytes(), &destination).await.unwrap();

        // read it back and verify the echoed value matches the sent value
        let (nread, _) = client.recv_from(&mut buffer).await.unwrap();

        tracing::info!(
            "received = {}",
            std::str::from_utf8(&buffer[..nread]).unwrap()
        );
        assert_eq!(message.as_bytes(), &buffer[..nread]);

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

#[cfg(all(feature = "sync", not(feature = "tokio")))]
fn main() {
    use yosemite::{style::Repliable, Session};

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .try_init()
        .unwrap();

    let mut server = Session::<Repliable>::new(Default::default()).unwrap();
    let destination = server.destination().to_owned();

    std::thread::spawn(move || {
        let mut buffer = [0u8; 1024];

        for _ in 0..3 {
            // read message from client and send it back to them
            let (nread, destination) = server.recv_from(&mut buffer).unwrap();
            server.send_to(&buffer[..nread], &destination).unwrap();
        }

        std::thread::sleep(Duration::from_secs(5));
    });

    let mut client = Session::<Repliable>::new(Default::default()).unwrap();
    let mut buffer = [0u8; 1024];

    for message in vec!["hello, world", "testing 123", "goodbye, world"] {
        // send message to server
        client.send_to(message.as_bytes(), &destination).unwrap();

        // read it back and verify the echoed value matches the sent value
        let (nread, _) = client.recv_from(&mut buffer).unwrap();

        tracing::info!(
            "received = {}",
            std::str::from_utf8(&buffer[..nread]).unwrap()
        );
        assert_eq!(message.as_bytes(), &buffer[..nread]);

        std::thread::sleep(Duration::from_secs(1));
    }
}
