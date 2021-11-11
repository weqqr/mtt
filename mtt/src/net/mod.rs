use crate::net::connection::connection_task;
use tokio::net::ToSocketAddrs;

pub mod connection;

pub use self::connection::{Request, Response};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

pub struct Credentials {
    pub name: String,
    pub password: String,
}

pub fn connect<A>(address: A, credentials: Credentials) -> (Sender<Request>, Receiver<Response>)
where
    A: ToSocketAddrs + Send + Sync + 'static,
{
    let (request_tx, request_rx) = mpsc::channel(10);
    let (response_tx, response_rx) = mpsc::channel(10);

    let _ = tokio::spawn(connection_task(address, request_rx, response_tx, credentials));

    (request_tx, response_rx)
}
