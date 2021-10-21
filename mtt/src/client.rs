use crate::game::Game;
use crate::math::Vector3;
use crate::net::clientbound::ClientBound;
use crate::net::serverbound::ServerBound;
use crate::net::{connect, Credentials, Request, Response};
use crate::serialize::{RawBytesUnsized, Serialize};
use crate::world::World;
use anyhow::Result;
use log::warn;
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
                packet: ServerBound::Init2 {
                    language_code: "".to_string(),
                },
                reliable: true,
                channel: 0,
            })?,
            ClientBound::TimeOfDay { time, time_speed } => {
                world.time = time as f32;
                world.time_speed = time_speed;
            }
            ClientBound::NodeDef { data } => {
                *game = Game::deserialize_nodes(&data.0)?;
                self.request_tx.blocking_send(Request::Send {
                    packet: ServerBound::ClientReady {
                        version_major: 5,
                        version_minor: 5,
                        version_patch: 0,
                        reserved: 0,
                        full_version: format!("mtt {}", env!("CARGO_PKG_VERSION")),
                        formspec_version: 4,
                    },
                    reliable: true,
                    channel: 0,
                })?;
            }
            ClientBound::MovePlayer { position, yaw, pitch } => {
                world.player.position = position / BS;
                // Adjust position by player height
                // world.player.position.y += 1.6;
                world.player.look_dir = Vector3::from_euler_angles(pitch, yaw);
                println!(
                    "pos={:?} pitch={} yaw={} look_dir={:?}",
                    world.player.position, pitch, yaw, world.player.look_dir
                );
            }
            ClientBound::BlockData { position, block } => {
                world.map.update_or_set(position, block);
                let mut blocks = Cursor::new(Vec::new());
                position.serialize(&mut blocks)?;

                self.request_tx.blocking_send(Request::Send {
                    packet: ServerBound::GotBlocks {
                        count: 1,
                        blocks: RawBytesUnsized(blocks.into_inner()),
                    },
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
