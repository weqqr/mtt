use crate::net::{connect, Credentials, Request, Response};
use anyhow::Result;
use log::warn;
use mtt_core::game::Game;
use mtt_core::math::Vector3;
use mtt_core::world::World;
use mtt_protocol::clientbound::*;
use mtt_protocol::serverbound::{ClientReady, GotBlocks, Init2, ServerBound};
use mtt_serialize::{RawBytesUnsized, Serialize};
use std::io::Cursor;
use tokio::sync::mpsc;

const BS: f32 = 10.0;

pub struct Client {
    request_tx: mpsc::Sender<Request>,
    response_rx: mpsc::Receiver<Response>,
}

impl Client {
    pub fn connect(server_address: String, credentials: Credentials) -> Self {
        let (request_tx, response_rx) = connect(server_address, credentials);

        Self {
            request_tx,
            response_rx,
        }
    }

    fn send(&self, packet: impl Into<ServerBound>, reliable: bool, channel: u8) -> Result<()> {
        Ok(self.request_tx.blocking_send(Request::Send {
            packet: packet.into(),
            reliable,
            channel,
        })?)
    }

    fn handle_auth_accept(&mut self, _packet: AuthAccept) -> Result<()> {
        let packet = Init2 {
            language_code: "".to_string(),
        };
        self.send(packet, true, 0)?;

        Ok(())
    }

    fn handle_announce_media(&mut self, _packet: AnnounceMedia) -> Result<()> {
        Ok(())
    }

    fn handle_time_of_day(&mut self, world: &mut World, packet: TimeOfDay) {
        world.time = packet.time as f32;
        world.time_speed = packet.time_speed;
    }

    fn handle_nodedef(&mut self, game: &mut Game, packet: NodeDef) -> Result<()> {
        *game = Game::deserialize_nodes(&packet.data.0)?;
        let packet = ClientReady {
            version_major: 5,
            version_minor: 5,
            version_patch: 0,
            reserved: 0,
            full_version: format!("mtt {}", env!("CARGO_PKG_VERSION")),
            formspec_version: 4,
        };
        self.send(packet, true, 0)?;

        Ok(())
    }

    fn handle_move_player(&mut self, world: &mut World, packet: MovePlayer) {
        world.player.position = packet.position / BS;
        world.player.look_dir = Vector3::from_euler_angles(packet.pitch, packet.yaw);
    }

    fn handle_block_data(&mut self, world: &mut World, packet: BlockData) -> Result<()> {
        world.map.update_or_set(packet.position, packet.block);
        let mut blocks = Cursor::new(Vec::new());
        packet.position.serialize(&mut blocks)?;

        let packet = GotBlocks {
            count: 1,
            blocks: RawBytesUnsized(blocks.into_inner()),
        };
        self.send(packet, true, 0)?;

        Ok(())
    }

    fn handle_packet(&mut self, game: &mut Game, world: &mut World, packet: ClientBound) -> Result<()> {
        match packet {
            ClientBound::AuthAccept(packet) => self.handle_auth_accept(packet)?,
            ClientBound::AnnounceMedia(packet) => self.handle_announce_media(packet)?,
            ClientBound::TimeOfDay(packet) => self.handle_time_of_day(world, packet),
            ClientBound::NodeDef(packet) => self.handle_nodedef(game, packet)?,
            ClientBound::MovePlayer(packet) => self.handle_move_player(world, packet),
            ClientBound::BlockData(packet) => self.handle_block_data(world, packet)?,
            _ => warn!("Ignoring {:?}", packet),
        }

        Ok(())
    }

    pub fn process_packets(&mut self, game: &mut Game, world: &mut World) {
        while let Ok(response) = self.response_rx.try_recv() {
            match response {
                Response::Disconnect => (),
                Response::Error(err) => panic!("{}", err),
                Response::Receive(packet) => self.handle_packet(game, world, packet).unwrap(),
                _ => (),
            }
        }
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        let _ = self.request_tx.blocking_send(Request::Disconnect);
    }
}
