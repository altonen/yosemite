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

// Asynchronous primary session:
//    cargo run --example primary_session
//
// Synchronous primary session:
//    cargo run --example primary_session --no-default-features --features sync

#[cfg(all(feature = "tokio", not(feature = "sync")))]
#[tokio::main]
async fn main() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use yosemite::{
        style::{Primary, Repliable, Stream},
        Session,
    };

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .try_init()
        .unwrap();

    // create stream server session
    let mut session = Session::<Stream>::new(Default::default()).await.unwrap();
    let stream_destination = session.destination().to_owned();

    // start event loop for the server which only echoes the message back
    tokio::spawn(async move {
        while let Ok(mut stream) = session.accept().await {
            let mut buffer = vec![0u8; 14];

            stream.read_exact(&mut buffer).await.unwrap();
            stream.write_all(&mut buffer).await.unwrap();
        }
    });

    // create datagram server session
    let mut server = Session::<Repliable>::new(Default::default()).await.unwrap();
    let datagram_destination = server.destination().to_owned();

    // start event loop for the server which only echoes the message back
    tokio::spawn(async move {
        let mut buffer = [0u8; 1024];

        for _ in 0..3 {
            let (nread, destination) = server.recv_from(&mut buffer).await.unwrap();
            server.send_to(&buffer[..nread], &destination).await.unwrap();
        }
    });

    // create primary session and add two subsessions: streams and repliable datagrams
    let mut session = Session::<Primary>::new(Default::default()).await.unwrap();
    let mut stream_session = session.create_subsession::<Stream>(Default::default()).await.unwrap();
    let mut datagram_session =
        session.create_subsession::<Repliable>(Default::default()).await.unwrap();

    // connect to the stream server, send a message and read response back
    {
        let mut stream = stream_session.connect(&stream_destination).await.unwrap();
        stream.write_all(format!("hello, world 0").as_bytes()).await.unwrap();

        let mut buffer = vec![0u8; 14];
        stream.read_exact(&mut buffer).await.unwrap();

        tracing::info!("stream read: {:?}", std::str::from_utf8(&buffer));
    }

    // send datagram to the datagram server and read response back
    {
        let mut buffer = [0u8; 1024];

        // send message to server
        datagram_session
            .send_to("goodbye world".as_bytes(), &datagram_destination)
            .await
            .unwrap();

        // read it back and verify the echoed value matches the sent value
        let (nread, _) = datagram_session.recv_from(&mut buffer).await.unwrap();

        tracing::info!(
            "received = {}",
            std::str::from_utf8(&buffer[..nread]).unwrap()
        );
    }
}

#[cfg(all(feature = "sync", not(feature = "tokio")))]
fn main() {
    use std::io::{Read, Write};
    use yosemite::{
        style::{Primary, Repliable, Stream},
        Session,
    };

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .try_init()
        .unwrap();

    // create stream server session
    let mut session = Session::<Stream>::new(Default::default()).unwrap();
    let stream_destination = session.destination().to_owned();

    // start event loop for the server which only echoes the message back
    std::thread::spawn(move || {
        while let Ok(mut stream) = session.accept() {
            let mut buffer = vec![0u8; 14];

            stream.read_exact(&mut buffer).unwrap();
            stream.write_all(&mut buffer).unwrap();
        }
    });

    // create datagram server session
    let mut server = Session::<Repliable>::new(Default::default()).unwrap();
    let datagram_destination = server.destination().to_owned();

    // start event loop for the server which only echoes the message back
    std::thread::spawn(move || {
        let mut buffer = [0u8; 1024];

        for _ in 0..3 {
            let (nread, destination) = server.recv_from(&mut buffer).unwrap();
            server.send_to(&buffer[..nread], &destination).unwrap();
        }
    });

    // create primary session and add two subsessions: streams and repliable datagrams
    let mut session = Session::<Primary>::new(Default::default()).unwrap();
    let mut stream_session = session.create_subsession::<Stream>(Default::default()).unwrap();
    let mut datagram_session = session.create_subsession::<Repliable>(Default::default()).unwrap();

    // connect to the stream server, send a message and read response back
    {
        let mut stream = stream_session.connect(&stream_destination).unwrap();
        stream.write_all(format!("hello, world 0").as_bytes()).unwrap();

        let mut buffer = vec![0u8; 14];
        stream.read_exact(&mut buffer).unwrap();

        tracing::info!("stream read: {:?}", std::str::from_utf8(&buffer));
    }

    // send datagram to the datagram server and read response back
    {
        let mut buffer = [0u8; 1024];

        // send message to server
        datagram_session
            .send_to("goodbye world".as_bytes(), &datagram_destination)
            .unwrap();

        // read it back and verify the echoed value matches the sent value
        let (nread, _) = datagram_session.recv_from(&mut buffer).unwrap();

        tracing::info!(
            "received = {}",
            std::str::from_utf8(&buffer[..nread]).unwrap()
        );
    }
}
