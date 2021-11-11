use crate::net::{connect, Credentials, Request, Response};
use anyhow::Result;
use log::warn;
use mtt_core::game::Game;
use mtt_core::math::Vector3;
use mtt_core::world::World;
use mtt_protocol::clientbound::ClientBound;
use mtt_protocol::serverbound::{ClientReady, GotBlocks, Init2};
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

    fn handle_packet(&mut self, game: &mut Game, world: &mut World, packet: ClientBound) -> Result<()> {
        match packet {
            ClientBound::AuthAccept { .. } => self.request_tx.blocking_send(Request::Send {
                packet: Init2 {
                    language_code: "".to_string(),
                }
                .into(),
                reliable: true,
                channel: 0,
            })?,
            ClientBound::AnnounceMedia(media) => {}
            ClientBound::TimeOfDay(time_of_day) => {
                world.time = time_of_day.time as f32;
                world.time_speed = time_of_day.time_speed;
            }
            ClientBound::NodeDef(nodedef) => {
                *game = Game::deserialize_nodes(&nodedef.data.0)?;
                self.request_tx.blocking_send(Request::Send {
                    packet: ClientReady {
                        version_major: 5,
                        version_minor: 5,
                        version_patch: 0,
                        reserved: 0,
                        full_version: format!("mtt {}", env!("CARGO_PKG_VERSION")),
                        formspec_version: 4,
                    }
                    .into(),
                    reliable: true,
                    channel: 0,
                })?;
            }
            ClientBound::MovePlayer(move_player) => {
                world.player.position = move_player.position / BS;
                // Adjust position by player height
                // world.player.position.y += 1.6;
                world.player.look_dir = Vector3::from_euler_angles(move_player.pitch, move_player.yaw);
                println!(
                    "pos={:?} pitch={} yaw={} look_dir={:?}",
                    world.player.position, move_player.pitch, move_player.yaw, world.player.look_dir
                );
            }
            ClientBound::BlockData(block_data) => {
                world.map.update_or_set(block_data.position, block_data.block);
                let mut blocks = Cursor::new(Vec::new());
                block_data.position.serialize(&mut blocks)?;

                self.request_tx.blocking_send(Request::Send {
                    packet: GotBlocks {
                        count: 1,
                        blocks: RawBytesUnsized(blocks.into_inner()),
                    }
                    .into(),
                    reliable: true,
                    channel: 0,
                })?;
            }
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
