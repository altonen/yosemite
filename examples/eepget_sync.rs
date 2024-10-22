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

//! cargo run --example=eepget_sync --no-default-features --features sync -- <host>

fn main() {
    #[cfg(feature = "sync")]
    {
        use tracing_subscriber::prelude::*;
        use yosemite::{Stream, StreamOptions};

        use std::{
            env,
            io::{Read, Write},
        };

        tracing_subscriber::registry()
            .with(tracing_subscriber::fmt::layer())
            .try_init()
            .unwrap();

        let host = env::args().nth(1).expect("host");

        let mut stream = Stream::new(host, StreamOptions::default()).unwrap();

        stream
            .write_all("GET / HTTP/1.1\r\n\r\n".as_bytes())
            .unwrap();

        let mut buffer = vec![0u8; 8192];
        let nread = stream.read(&mut buffer).unwrap();

        tracing::info!("{:?}", std::str::from_utf8(&buffer[..nread]));
    }
}
