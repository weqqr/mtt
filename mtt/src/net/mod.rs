use crate::net::connection::connection_task;
use tokio::net::ToSocketAddrs;

pub mod clientbound;
pub mod connection;
pub mod packet;
pub mod serverbound;

pub use self::connection::{Request, Response};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

pub fn connect<A>(address: A, player_name: String, password: String) -> (Sender<Request>, Receiver<Response>)
where
    A: ToSocketAddrs + Send + Sync + 'static,
{
    let (request_tx, request_rx) = mpsc::channel(10);
    let (response_tx, response_rx) = mpsc::channel(10);

    let _ = tokio::spawn(connection_task(address, request_rx, response_tx, player_name, password));

    (request_tx, response_rx)
}
