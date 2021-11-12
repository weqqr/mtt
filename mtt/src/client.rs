use crate::media::MediaStorage;
use crate::net::{connect, Credentials, Request, Response};
use anyhow::Result;
use log::warn;
use mtt_core::game::Game;
use mtt_core::math::Vector3;
use mtt_core::world::World;
use mtt_protocol::clientbound::*;
use mtt_protocol::serverbound::{ClientReady, GotBlocks, Init2, RequestMedia, ServerBound};
use mtt_serialize::{RawBytesUnsized, Serialize};
use std::io::Cursor;
use tokio::sync::mpsc;

const BS: f32 = 10.0;

pub struct Client {
    request_tx: mpsc::Sender<Request>,
    response_rx: mpsc::Receiver<Response>,
    media_ready: bool,
    nodes_ready: bool,
}

impl Client {
    pub fn connect(server_address: String, credentials: Credentials) -> Self {
        let (request_tx, response_rx) = connect(server_address, credentials);

        Self {
            request_tx,
            response_rx,
            media_ready: false,
            nodes_ready: false,
        }
    }

    fn send(&self, packet: impl Into<ServerBound>, reliable: bool, channel: u8) -> Result<()> {
        Ok(self.request_tx.blocking_send(Request::Send {
            packet: packet.into(),
            reliable,
            channel,
        })?)
    }

    pub fn is_ready(&self) -> bool {
        self.media_ready && self.nodes_ready
    }

    fn send_client_ready(&self) -> Result<()> {
        let packet = ClientReady {
            version_major: 5,
            version_minor: 5,
            version_patch: 0,
            reserved: 0x77,
            full_version: format!("mtt {}", env!("CARGO_PKG_VERSION")),
            formspec_version: 4,
        };
        self.send(packet, true, 0)
    }

    fn handle_auth_accept(&mut self, _packet: AuthAccept) -> Result<()> {
        let packet = Init2 {
            language_code: "".to_string(),
        };
        self.send(packet, true, 0)?;

        Ok(())
    }

    fn handle_announce_media(&mut self, media: &mut MediaStorage, packet: AnnounceMedia) -> Result<()> {
        media.set_digests(packet.digests);
        let missing_files = media.missing_files();
        let packet = RequestMedia { media: missing_files };
        self.send(packet, true, 0)?;
        Ok(())
    }

    fn handle_media(&mut self, media: &mut MediaStorage, packet: Media) -> Result<()> {
        for (name, data) in packet.files {
            media.insert(&name, &data)?;
        }

        if packet.bunch_id == packet.bunch_count - 1 {
            self.media_ready = true;
        }

        if self.is_ready() {
            self.send_client_ready()?;
        }

        Ok(())
    }

    fn handle_time_of_day(&mut self, world: &mut World, packet: TimeOfDay) {
        world.time = packet.time as f32;
        world.time_speed = packet.time_speed;
    }

    fn handle_nodedef(&mut self, game: &mut Game, packet: NodeDef) -> Result<()> {
        *game = Game::deserialize_nodes(&packet.data.0)?;
        self.nodes_ready = true;

        if self.is_ready() {
            self.send_client_ready()?;
        }

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

    fn handle_packet(
        &mut self,
        media: &mut MediaStorage,
        game: &mut Game,
        world: &mut World,
        packet: ClientBound,
    ) -> Result<()> {
        match packet {
            ClientBound::AuthAccept(packet) => self.handle_auth_accept(packet)?,
            ClientBound::AnnounceMedia(packet) => self.handle_announce_media(media, packet)?,
            ClientBound::Media(packet) => self.handle_media(media, packet)?,
            ClientBound::TimeOfDay(packet) => self.handle_time_of_day(world, packet),
            ClientBound::NodeDef(packet) => self.handle_nodedef(game, packet)?,
            ClientBound::MovePlayer(packet) => self.handle_move_player(world, packet),
            ClientBound::BlockData(packet) => self.handle_block_data(world, packet)?,
            _ => warn!("Ignoring {:?}", packet),
        }

        Ok(())
    }

    pub fn process_packets(&mut self, media: &mut MediaStorage, game: &mut Game, world: &mut World) {
        while let Ok(response) = self.response_rx.try_recv() {
            match response {
                Response::Disconnect => (),
                Response::Error(err) => panic!("{}", err),
                Response::Receive(packet) => self.handle_packet(media, game, world, packet).unwrap(),
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
