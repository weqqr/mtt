pub mod image;

use crate::media::image::Image;
use anyhow::{Context, Result};
use log::warn;
use std::collections::HashMap;
use std::fmt::Write;
use std::path::PathBuf;

type ImageIndex = usize;

pub struct MediaStorage {
    cache_dir: PathBuf,
    digests: HashMap<String, Vec<u8>>,
    image_index: HashMap<String, usize>,
    indexed_images: Vec<Image>,
    last_index: ImageIndex,
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
            image_index: HashMap::new(),
            indexed_images: Vec::new(),
            last_index: 0,
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

    pub fn get(&self, name: &str) -> Option<Vec<u8>> {
        let digest = self.digests.get(name)?;
        let path = self.cache_dir.join(encode_hex(digest));
        std::fs::read(path).ok()
    }

    fn store_image_in_cache(&mut self, image: Image) -> ImageIndex {
        self.indexed_images.push(image);
        let stored_image_index = self.last_index;
        self.last_index += 1;
        stored_image_index
    }

    pub fn load_image(&mut self, name: &str) -> Option<&Image> {
        if let Some(index) = self.image_index.get(name) {
            return self.indexed_images.get(*index);
        }

        if name.ends_with("png") {
            let data = self.get(name)?;
            let image = Image::load_png(&data).ok()?;
            let index = self.store_image_in_cache(image);
            self.indexed_images.get(index)
        } else {
            warn!("Unsupported texture format: {}", name);
            None
        }
    }

    pub fn insert(&self, name: &str, data: &Vec<u8>) -> Result<()> {
        let digest = self.digests.get(name).context("server sent unannounced file")?;
        let path = self.cache_dir.join(encode_hex(digest));
        Ok(std::fs::write(path, data)?)
    }
}
