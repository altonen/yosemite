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

// Asynchronous destination generation:
//    cargo run --example generate_destination
//
// Synchronous destination generation:
//    cargo run --example generate_destination --no-default-features --features sync

#[cfg(all(feature = "async", not(feature = "sync")))]
#[tokio::main]
async fn main() {
    use yosemite::{style::Stream, DestinationKind, RouterApi, Session, SessionOptions};

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .try_init()
        .unwrap();

    // generate new destination
    //
    // yosemite's default signature type is 7
    let (destination, private_key) = RouterApi::generate_destination().await.unwrap();

    // generate new session using the generated destination
    let session = Session::<Stream>::new(SessionOptions {
        destination: DestinationKind::Persistent {
            private_key: private_key.clone(),
        },
        ..Default::default()
    })
    .await
    .unwrap();

    tracing::info!("generated destination = {destination}");
    tracing::info!("session destination = {}", session.destination());

    assert_eq!(private_key, session.destination());
}

#[cfg(all(feature = "sync", not(feature = "async")))]
fn main() {
    use yosemite::{style::Stream, DestinationKind, RouterApi, Session, SessionOptions};

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .try_init()
        .unwrap();

    // generate new destination
    //
    // yosemite's default signature type is 7
    let (destination, private_key) = RouterApi::generate_destination().unwrap();

    // generate new session using the generated destination
    let session = Session::<Stream>::new(SessionOptions {
        destination: DestinationKind::Persistent {
            private_key: private_key.clone(),
        },
        ..Default::default()
    })
    .unwrap();

    tracing::info!("generated destination = {destination}");
    tracing::info!("session destination = {}", session.destination());

    assert_eq!(private_key, session.destination());
}
