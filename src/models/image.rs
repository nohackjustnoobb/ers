use image::{load_from_memory, DynamicImage, GenericImageView};

pub struct Image {
    content: Vec<u8>,
    parsed: Option<DynamicImage>,
    width: Option<u32>,
    height: Option<u32>,
}

impl Image {
    pub fn new(content: Vec<u8>) -> Image {
        Image {
            content,
            parsed: None,
            width: None,
            height: None,
        }
    }

    pub fn get(&mut self) -> &DynamicImage {
        if self.parsed.is_none() {
            self.parsed = Some(load_from_memory(&self.content).unwrap());
            self.content.clear();
        }

        let dem = self.parsed.as_ref().unwrap().dimensions();

        self.width = Some(dem.0);
        self.height = Some(dem.1);

        self.parsed.as_ref().unwrap()
    }

    // FIXME temp fix only
    pub fn cal_width(&self, height: usize) -> u16 {
        return if self.width.unwrap() > self.height.unwrap() {
            (self.width.unwrap() * (height as u32) / self.height.unwrap() / 2) as u16
        } else {
            (self.width.unwrap() * (height as u32) / self.height.unwrap()) as u16
        };
    }
}
