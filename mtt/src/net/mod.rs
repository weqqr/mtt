use crate::net::clientbound::ClientBound;
use crate::net::packet::{Control, PacketHeader, PacketType, Reliability};
use crate::net::serverbound::ServerBound;
use crate::serialize::Serialize;
use anyhow::Result;
use log::info;
use std::io::{Cursor, Write};
use tokio::net::{ToSocketAddrs, UdpSocket};
use tokio::time::Duration;

pub mod clientbound;
pub mod packet;
pub mod serverbound;

const SPLIT_THRESHOLD: usize = 400;

pub struct Connection {
    socket: UdpSocket,
    peer_id: u16,
    seqnum: u16,
}

#[derive(Debug)]
pub struct ReceivedPacket {
    pub header: PacketHeader,
    pub body: Option<ClientBound>,
}

impl Connection {
    pub async fn connect<A: ToSocketAddrs>(address: A, player_name: String) -> Result<(Self, ClientBound)> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;

        socket.connect(address).await?;

        let mut conn = Self {
            socket,
            seqnum: 0xFFDC,
            peer_id: 0,
        };

        let packet = ServerBound::Handshake {};
        conn.send_and_wait_for(packet, true, 0, |packet| {
            matches!(packet.header.ty, PacketType::Control(Control::SetPeerId { .. }))
        })
        .await?;

        let packet = ServerBound::Init {
            max_serialization_version: 28,
            supported_compression_modes: 0,
            min_protocol_version: 37,
            max_protocol_version: 39,
            player_name: player_name.clone(),
        };
        let hello = conn
            .send_and_wait_for(packet, false, 1, |packet| {
                matches!(packet.body, Some(ClientBound::Hello { .. }))
            })
            .await?;

        Ok((conn, hello.body.unwrap()))
    }

    pub async fn send_and_wait_for<F>(
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
                    let packet = packet.unwrap();
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

    pub fn process_control(&mut self, control: &Control) {
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

    pub async fn send_packet(&mut self, packet: ServerBound, reliable: bool, channel: u8) -> Result<()> {
        info!("SEND {:?}", packet);
        let mut data = Vec::new();
        packet.serialize(&mut data)?;
        self.send(&data, reliable, channel).await?;
        Ok(())
    }

    pub async fn send_disconnect(&mut self) -> Result<()> {
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

    pub async fn send_ack(&mut self, seqnum: u16, channel: u8) -> Result<()> {
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

    pub async fn send(&mut self, payload: &[u8], reliable: bool, channel: u8) -> Result<()> {
        if payload.len() > SPLIT_THRESHOLD {
            todo!("split packets")
        }

        let reliability = if reliable {
            let reliability = Reliability::Reliable { seqnum: self.seqnum };
            self.seqnum = self.seqnum.wrapping_add(1);
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
