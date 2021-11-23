use crate::Error::InvalidWorldInfo;
use std::fs::read_to_string;
use std::path::Path;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("World info file (world.mt) is invalid: {0}")]
    InvalidWorldInfo(&'static str),

    #[error("Unsupported backend: {0}")]
    UnsupportedBackend(String),
}

#[derive(Debug, Clone)]
pub enum BackendType {
    Sqlite,
}

#[derive(Debug, Clone)]
pub struct WorldInfo {
    pub auth_backend: BackendType,
    pub backend: BackendType,
    pub game: String,
    pub player_backend: BackendType,
    pub world_name: String,
}

fn parse_backend_type(value: &str) -> Result<BackendType, Error> {
    match value {
        "sqlite3" => Ok(BackendType::Sqlite),
        backend => Err(Error::UnsupportedBackend(backend.to_owned())),
    }
}

impl WorldInfo {
    pub fn parse(info: &str) -> Result<WorldInfo, Error> {
        let variables = info
            .lines()
            .map(|line| line.splitn(2, '=').map(|part| part.trim()))
            .filter_map(|mut parts| Some((parts.next()?, parts.next()?)));

        let mut auth_backend = Err(Error::InvalidWorldInfo("auth_backend is required"));
        let mut backend = Err(Error::InvalidWorldInfo("backend is required"));
        let mut game = Err(Error::InvalidWorldInfo("gameid is required"));
        let mut player_backend = Err(Error::InvalidWorldInfo("player_backend is required"));
        let mut world_name = Err(Error::InvalidWorldInfo("world_name is required"));

        for (name, value) in variables {
            match name {
                "auth_backend" => auth_backend = parse_backend_type(value),
                "backend" => backend = parse_backend_type(value),
                "gameid" => game = Ok(value.to_owned()),
                "player_backend" => player_backend = parse_backend_type(value),
                "world_name" => world_name = Ok(value.to_owned()),
                _ => (),
            }
        }

        Ok(WorldInfo {
            auth_backend: auth_backend?,
            backend: backend?,
            game: game?,
            player_backend: player_backend?,
            world_name: world_name?,
        })
    }
}

pub struct World {
    pub info: WorldInfo,
}

impl World {
    pub fn open<P: AsRef<Path>>(p: P) -> Result<Self, Error> {
        let world_mt_path = p.as_ref().join("world.mt");
        let info = std::fs::read_to_string(world_mt_path)?;
        let info = WorldInfo::parse(&info)?;

        Ok(Self { info })
    }
}
