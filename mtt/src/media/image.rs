use anyhow::Result;
use png::{ColorType, Transformations};

pub struct Image {
    width: usize,
    height: usize,
    data: Vec<u8>,
}

impl Image {
    const DEPTH: usize = 4;

    pub fn new(width: usize, height: usize) -> Image {
        Image {
            width,
            height,
            data: vec![0; Self::DEPTH * width * height],
        }
    }

    pub fn load_png(data: &[u8]) -> Result<Image> {
        let mut decoder = png::Decoder::new(data);
        decoder.set_transformations(Transformations::normalize_to_color8());
        let mut reader = decoder.read_info()?;
        let mut data = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut data)?;
        // data.resize(info.buffer_size(), 0);

        let width = info.width as usize;
        let height = info.height as usize;

        // Re-encode into RGBA
        let data = match info.color_type {
            ColorType::Grayscale => unimplemented!(),
            ColorType::Rgb => data.chunks(3).map(|c| [c[0], c[1], c[2], 255]).flatten().collect(),
            ColorType::GrayscaleAlpha => unimplemented!(),
            ColorType::Rgba => data,
            // This should've been covered by `png`
            ColorType::Indexed => data,
        };

        Ok(Image { width, height, data })
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }
}
