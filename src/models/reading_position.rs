use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{self, BufReader, BufWriter, Read},
    path::PathBuf,
};

#[derive(Serialize, Deserialize)]
pub struct ReadingPosition {
    pub page: String,
    pub offset: usize,
}

impl ReadingPosition {
    pub fn new(page: String, offset: usize) -> Self {
        Self { page, offset }
    }

    pub fn save(&self, book_hash: &str) -> io::Result<()> {
        let mut cache_dir = dirs::cache_dir().unwrap_or_else(|| PathBuf::from("."));
        cache_dir.push("ers");
        fs::create_dir_all(&cache_dir)?;

        let mut position_file = cache_dir;
        position_file.push(format!("{}.json", book_hash));

        let file = File::create(position_file)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, self)?;

        Ok(())
    }

    pub fn load(book_hash: &str) -> io::Result<Option<Self>> {
        let mut cache_dir = dirs::cache_dir().unwrap_or_else(|| PathBuf::from("."));
        cache_dir.push("ers");
        let mut position_file = cache_dir;
        position_file.push(format!("{}.json", book_hash));

        if !position_file.exists() {
            return Ok(None);
        }

        let file = File::open(position_file)?;
        let reader = BufReader::new(file);
        let position: ReadingPosition = serde_json::from_reader(reader)?;

        Ok(Some(position))
    }
}

pub fn calculate_book_hash(book_path: &str) -> io::Result<String> {
    let mut file = File::open(book_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(format!("{:x}", md5::compute(&buffer)))
}
