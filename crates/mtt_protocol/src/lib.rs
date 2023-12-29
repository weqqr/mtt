use std::collections::VecDeque;
use std::io::{Cursor, Write};

use crate::clientbound::ClientBound;
use crate::frame::{Control, FrameHeader, Reliability, FrameType};
use anyhow::Result;
use mtt_serialize::Serialize;

pub mod clientbound;
pub mod frame;
pub mod serverbound;

pub enum Input<'a> {
    Receive(&'a [u8]),
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
}

impl Client {
    pub fn new() -> Self {
        Self {
            state: ConnectionState::Start,
            output_queue: VecDeque::new(),

            peer_id: 0,
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

    pub fn handle_input(&mut self, input: Input) -> Result<()> {
        let Input::Receive(data) = input else {
            return Ok(())
        };

        let r = &mut Cursor::new(data);

        let frame_header = FrameHeader::deserialize(r)?;

        if let Reliability::Reliable { seqnum } = frame_header.reliability {
            self.send_ack(frame_header.channel, seqnum)?;
        }

        match self.state {
            ConnectionState::Start => todo!(),
            ConnectionState::Handshake => todo!(),
            ConnectionState::InGame => todo!(),
        }

        Ok(())
    }
}
