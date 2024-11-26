use image::{load_from_memory, DynamicImage};

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

        self.width = Some(self.parsed.as_ref().unwrap().width());
        self.height = Some(self.parsed.as_ref().unwrap().height());

        self.parsed.as_ref().unwrap()
    }
}
