use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fmt::Write;
use std::path::PathBuf;

pub struct MediaStorage {
    cache_dir: PathBuf,
    digests: HashMap<String, Vec<u8>>,
}

fn encode_hex(data: &[u8]) -> String {
    let mut output = String::new();
    for byte in data {
        write!(&mut output, "{:02x}", byte).unwrap();
    }
    output
}

impl MediaStorage {
    pub fn new() -> Result<Self> {
        let mut cache_dir = dirs::cache_dir().context("could not find cache directory")?;
        cache_dir.push("mtt");
        cache_dir.push("media");

        std::fs::create_dir_all(&cache_dir)?;

        Ok(Self {
            cache_dir,
            digests: HashMap::new(),
        })
    }

    pub fn set_digests(&mut self, digests: HashMap<String, Vec<u8>>) {
        self.digests = digests;
    }

    pub fn missing_files(&self) -> Vec<String> {
        self.digests
            .iter()
            .filter_map(|(name, digest)| (!self.contains(digest)).then(|| name).cloned())
            .collect()
    }

    pub fn contains(&self, digest: &[u8]) -> bool {
        self.cache_dir.join(encode_hex(digest)).exists()
    }

    pub fn get(&self, digest: &[u8]) -> Option<Vec<u8>> {
        let path = self.cache_dir.join(encode_hex(digest));
        std::fs::read(path).ok()
    }

    pub fn insert(&self, name: &str, data: &Vec<u8>) -> Result<()> {
        let digest = self.digests.get(name).context("server sent unannounced file")?;
        let path = self.cache_dir.join(encode_hex(digest));
        Ok(std::fs::write(path, data)?)
    }
}
