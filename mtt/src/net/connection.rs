use crate::net::clientbound::ClientBound;
use crate::net::packet::{Control, PacketHeader, PacketType, Reliability};
use crate::net::serverbound::ServerBound;
use crate::serialize::{RawBytes16, Serialize};
use anyhow::Result;
use log::info;
use sha2::Sha256;
use srp::client::{srp_private_key, SrpClient};
use std::io::{Cursor, Write};
use tokio::net::{ToSocketAddrs, UdpSocket};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::Duration;

const SPLIT_THRESHOLD: usize = 400;

pub struct Connection {
    socket: UdpSocket,
    peer_id: u16,
    seqnum: [u16; 3],
}

#[derive(Debug)]
pub struct ReceivedPacket {
    pub header: PacketHeader,
    pub body: Option<ClientBound>,
}

impl Connection {
    async fn connect<A: ToSocketAddrs>(address: A, player_name: String) -> Result<(Self, ClientBound)> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;

        socket.connect(address).await?;

        let mut conn = Self {
            socket,
            seqnum: [0xFFDC; 3],
            peer_id: 0,
        };

        let packet = ServerBound::Handshake {};
        conn.send_and_wait_for(packet, true, 0, |packet| {
            matches!(packet.header.ty, PacketType::Control(Control::SetPeerId { .. }))
        })
        .await?;

        let packet = ServerBound::Init {
            max_serialization_version: 29,
            supported_compression_modes: 0,
            min_protocol_version: 40,
            max_protocol_version: 40,
            player_name: player_name.clone(),
        };
        let hello = conn
            .send_and_wait_for(packet, false, 1, |packet| {
                matches!(packet.body, Some(ClientBound::Hello { .. }))
            })
            .await?;

        Ok((conn, hello.body.unwrap()))
    }

    async fn send_and_wait_for<F>(
        &mut self,
        packet: ServerBound,
        reliable: bool,
        channel: u8,
        criterion: F,
    ) -> Result<ReceivedPacket>
    where
        F: Fn(&ReceivedPacket) -> bool,
    {
        let mut resend_interval = tokio::time::interval(Duration::from_millis(100));
        let timeout = tokio::time::sleep(Duration::from_secs(10));
        tokio::pin!(timeout);
        loop {
            tokio::select! {
                _ = resend_interval.tick() => {
                    self.send_packet(packet.clone(), reliable, channel).await?;
                }
                packet = self.receive_packet() => {
                    let packet = packet?;
                    if criterion(&packet) {
                        break Ok(packet);
                    }
                }
                _ = &mut timeout => {
                    anyhow::bail!("timed out");
                }
            }
        }
    }

    fn process_control(&mut self, control: &Control) {
        match control {
            Control::Ack { seqnum } => {
                info!("Server ACK {}", seqnum);
            }
            Control::SetPeerId { peer_id } => {
                info!("Setting peer_id = {}", peer_id);
                self.peer_id = *peer_id;
            }
            Control::Ping => {}
            Control::Disco => {}
        }
    }

    pub async fn receive_packet(&mut self) -> Result<ReceivedPacket> {
        let mut buf = [0; 1500];
        self.socket.recv(&mut buf).await?;

        let mut buf = Cursor::new(buf);

        let header = PacketHeader::deserialize(&mut buf)?;

        let reliability = header.reliability.clone();
        let channel = header.channel;

        let packet = match &header.ty {
            PacketType::Control(control) => {
                self.process_control(control);
                Ok(ReceivedPacket { header, body: None })
            }
            PacketType::Original => {
                let clientbound = ClientBound::deserialize(&mut buf)?;
                Ok(ReceivedPacket {
                    header,
                    body: Some(clientbound),
                })
            }
        };

        info!("RECV {:?}", packet);

        if let Reliability::Reliable { seqnum } = reliability {
            self.send_ack(seqnum, channel).await.unwrap();
        }

        packet
    }

    async fn send_packet(&mut self, packet: ServerBound, reliable: bool, channel: u8) -> Result<()> {
        info!("SEND {:?}", packet);
        let mut data = Vec::new();
        packet.serialize(&mut data)?;
        self.send(&data, reliable, channel).await?;
        Ok(())
    }

    async fn send_disconnect(&mut self) -> Result<()> {
        let packet_header = PacketHeader {
            peer_id: self.peer_id,
            channel: 0,
            reliability: Reliability::Unreliable,
            ty: PacketType::Control(Control::Disco),
        };

        let mut buf = Vec::new();
        packet_header.serialize(&mut buf)?;

        self.socket.send(&buf).await?;
        Ok(())
    }

    async fn send_ack(&mut self, seqnum: u16, channel: u8) -> Result<()> {
        let control = Control::Ack { seqnum };

        let packet_header = PacketHeader {
            peer_id: self.peer_id,
            channel,
            reliability: Reliability::Unreliable,
            ty: PacketType::Control(control),
        };

        info!("ACK  {:?}", packet_header);

        let mut buf = Vec::new();
        packet_header.serialize(&mut buf)?;

        self.socket.send(&buf).await?;
        Ok(())
    }

    async fn send(&mut self, payload: &[u8], reliable: bool, channel: u8) -> Result<()> {
        if payload.len() > SPLIT_THRESHOLD {
            todo!("split packets")
        }

        let reliability = if reliable {
            let reliability = Reliability::Reliable {
                seqnum: self.seqnum[channel as usize],
            };
            self.seqnum[channel as usize] = self.seqnum[channel as usize].wrapping_add(1);
            reliability
        } else {
            Reliability::Unreliable
        };

        let packet_header = PacketHeader {
            peer_id: self.peer_id,
            channel,
            reliability,
            ty: PacketType::Original,
        };

        let mut data = Vec::new();
        packet_header.serialize(&mut data)?;
        data.write_all(payload)?;

        self.socket.send(&data).await?;

        Ok(())
    }
}

#[derive(Debug)]
pub enum Request {
    Send {
        packet: ServerBound,
        reliable: bool,
        channel: u8,
    },
    Authenticate {
        player_name: String,
        password: String,
    },
    Disconnect,
}

#[derive(Debug)]
pub enum Response {
    Error(anyhow::Error),
    Receive(ClientBound),
    Disconnect,
}

macro_rules! send_err {
    ($e:expr, $channel:expr) => {
        match $e {
            Ok(value) => value,
            Err(err) => match $channel.send(Response::Error(err)).await {
                _ => return,
            },
        }
    };
}

async fn perform_auth(
    conn: &mut Connection,
    player_name: String,
    password: String,
    response_tx: &Sender<Response>,
) -> Result<()> {
    // FIXME: use random values
    let a = [5; 64];
    let srp_client = SrpClient::<Sha256>::new(&a, &srp::groups::G_2048);
    let a = srp_client.get_a_pub();

    let packet = ServerBound::SrpBytesA {
        data: RawBytes16(a),
        based_on: 1,
    };

    conn.send_packet(packet, true, 1).await?;

    let (salt, b_pub) = loop {
        let response = conn.receive_packet().await?;
        match response.body {
            Some(ClientBound::Hello { .. }) => {
                println!("Ignoring extraneous ClientBound::Hello during auth");
            }
            Some(ClientBound::SrpBytesSB { s, b }) => break (s.0, b.0),
            Some(packet) => response_tx.send(Response::Receive(packet)).await?,
            _ => (),
        }
    };

    let lowercase_name = player_name.as_bytes().to_ascii_lowercase();

    let private_key = srp_private_key::<Sha256>(&lowercase_name, password.as_bytes(), salt.as_slice());
    let verifier = srp_client
        .process_reply_with_username_and_salt(player_name.as_bytes(), &salt, &private_key, &b_pub)
        .unwrap();
    let packet = ServerBound::SrpBytesM {
        data: RawBytes16(verifier.get_proof().to_vec()),
    };
    conn.send_packet(packet, true, 1).await?;

    Ok(())
}

pub(super) async fn connection_task<A: ToSocketAddrs>(
    address: A,
    mut request_rx: Receiver<Request>,
    response_tx: Sender<Response>,
    player_name: String,
) {
    let conn = Connection::connect(address, player_name);
    let conn = tokio::time::timeout(Duration::from_secs(5), conn);

    let conn = match conn.await {
        Ok(conn) => conn,
        Err(_) => Err(anyhow::anyhow!("connection timed out")),
    };
    let (mut conn, hello) = send_err!(conn, response_tx);

    response_tx.send(Response::Receive(hello)).await.unwrap();

    loop {
        tokio::select! {
            request = request_rx.recv() => {
                match request {
                    Some(Request::Send { packet, reliable, channel }) => {
                        let packet = packet;
                        send_err!(conn.send_packet(packet, reliable, channel).await, response_tx);
                    }
                    Some(Request::Authenticate { player_name, password}) => {
                        send_err!(perform_auth(&mut conn, player_name, password, &response_tx).await, response_tx);
                    }
                    None | Some(Request::Disconnect) => {
                        send_err!(conn.send_disconnect().await, response_tx);
                        break;
                    }
                }
            }
            packet = conn.receive_packet() => {
                if let Some(clientbound) = send_err!(packet, response_tx).body {
                    response_tx.send(Response::Receive(clientbound)).await.unwrap();
                }
            }
        }
    }
}