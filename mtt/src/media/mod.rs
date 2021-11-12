use anyhow::Result;
use std::fmt::Write;
use std::path::PathBuf;

pub struct MediaStorage {
    cache_dir: PathBuf,
}

fn encode_hex(data: &[u8]) -> String {
    let mut output = String::new();
    for byte in data {
        write!(&mut output, "{:02x}", byte).unwrap();
    }
    output
}

impl MediaStorage {
    pub fn new() -> Self {
        let mut cache_dir = dirs::cache_dir().unwrap();
        cache_dir.push("mtt");
        cache_dir.push("media");

        println!("{:?}", cache_dir);

        Self { cache_dir }
    }

    pub fn contains(&self, hash: &[u8]) -> bool {
        self.cache_dir.join(encode_hex(hash)).exists()
    }

    pub fn get(&self, hash: &[u8]) -> Option<Vec<u8>> {
        let path = self.cache_dir.join(encode_hex(hash));
        std::fs::read(path).ok()
    }

    pub fn insert(&self, hash: &[u8], data: &Vec<u8>) -> Result<()> {
        let path = self.cache_dir.join(encode_hex(hash));
        Ok(std::fs::write(path, data)?)
    }
}
