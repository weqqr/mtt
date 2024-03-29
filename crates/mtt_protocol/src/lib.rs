use std::collections::VecDeque;
use std::io::{Cursor, Write};

use crate::clientbound::ClientBound;
use crate::frame::{Control, FrameHeader, FrameType, Reliability};
use crate::serverbound::ServerBound;
use anyhow::Result;
use mtt_serialize::Serialize;

pub mod clientbound;
pub mod frame;
pub mod serverbound;

pub enum Input<'a> {
    Receive(&'a [u8]),
    Packet {
        is_reliable: bool,
        packet: ServerBound,
    },
    None,
}

pub enum Output {
    Packet(ClientBound),
    Send(Vec<u8>),
    None,
}

pub enum ConnectionState {
    Start,
    Handshake,
    InGame,
}

pub struct Client {
    state: ConnectionState,
    output_queue: VecDeque<Output>,

    peer_id: u16,
    seqnum: u16,
}

impl Client {
    pub fn new() -> Self {
        Self {
            state: ConnectionState::Start,
            output_queue: VecDeque::new(),

            peer_id: 0,
            seqnum: 0xF1FE,
        }
    }

    pub fn poll_output(&mut self) -> impl Iterator<Item = Output> + '_ {
        self.output_queue.drain(..)
    }

    fn send_frame(&mut self, header: FrameHeader, data: &[u8]) -> Result<()> {
        let mut buf = Vec::new();

        header.serialize(&mut buf)?;

        buf.write_all(data)?;

        self.output_queue.push_back(Output::Send(buf));

        Ok(())
    }

    fn send_ack(&mut self, channel: u8, seqnum: u16) -> Result<()> {
        let header = FrameHeader {
            peer_id: self.peer_id,
            channel,
            reliability: Reliability::Unreliable,
            ty: FrameType::Control(Control::Ack { seqnum }),
        };

        self.send_frame(header, &[])
    }

    fn handle_control(&mut self, control: Control) {
        match control {
            Control::Ack { seqnum } => {}
            Control::SetPeerId { peer_id } => {
                self.peer_id = peer_id;
            }
            Control::Ping => {}
            Control::Disco => {}
        }
    }

    fn handle_clientbound_data(&mut self, data: &[u8]) -> Result<()> {
        let r = &mut Cursor::new(data);

        let frame_header = FrameHeader::deserialize(r)?;

        if let Reliability::Reliable { seqnum } = frame_header.reliability {
            self.send_ack(frame_header.channel, seqnum)?;
        }

        println!("{:?}", frame_header);

        match frame_header.ty {
            FrameType::Control(control) => self.handle_control(control),
            _ => {}
        }

        Ok(())
        // match self.state {
        //     ConnectionState::Start => todo!(),
        //     ConnectionState::Handshake => todo!(),
        //     ConnectionState::InGame => todo!(),
        // }
    }

    fn handle_serverbound_packet(&mut self, is_reliable: bool, packet: ServerBound) -> Result<()> {
        let reliability = if is_reliable {
            Reliability::Reliable { seqnum: self.seqnum }
        } else {
            Reliability::Unreliable
        };

        let frame = FrameHeader {
            peer_id: self.peer_id,
            channel: 0,
            reliability,
            ty: FrameType::Original,
        };

        let mut data = Vec::new();

        packet.serialize(&mut data)?;

        self.send_frame(frame, &data)
    }

    pub fn handle_input(&mut self, input: Input) -> Result<()> {
        match input {
            Input::Receive(data) => self.handle_clientbound_data(data),
            Input::Packet { is_reliable, packet } => self.handle_serverbound_packet(is_reliable, packet),
            Input::None => Ok(()),
        }
    }
}
