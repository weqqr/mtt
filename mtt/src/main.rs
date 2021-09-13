#![allow(dead_code)]

mod net;
mod serialize;

use crate::net::Connection;

fn main() -> anyhow::Result<()> {
    let mut connection = Connection::new("127.0.0.1:30000")?;

    connection.send_payload(b"test", true)?;

    println!("Hello, world!");

    Ok(())
}
