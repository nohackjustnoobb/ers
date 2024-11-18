use image::{load_from_memory, DynamicImage};

pub struct Image {
    content: Vec<u8>,
    parsed: Option<DynamicImage>,
}

impl Image {
    pub fn new(content: Vec<u8>) -> Image {
        Image {
            content,
            parsed: None,
        }
    }

    pub fn get(&mut self) -> &DynamicImage {
        if self.parsed.is_none() {
            self.parsed = Some(load_from_memory(&self.content).unwrap());
        }

        self.parsed.as_ref().unwrap()
    }
}
