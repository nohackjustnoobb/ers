use regex::Regex;
use std::{collections::HashMap, path::Path};

use super::{epub::EpubDoc, image::Image, page::Page};

pub struct Book {
    pub title: String,
    pub cover: Option<String>,
    pub pages: HashMap<String, Page>,
    pub images: HashMap<String, Image>,
    pub order: Vec<String>,
    pub toc: Vec<(String, String)>,
}

impl Book {
    pub fn new(path: &str) -> Book {
        // Open the book
        let mut doc = EpubDoc::new(path);

        // Extract all the images
        let mut images = HashMap::new();
        let img_reg = Regex::new(r"image/.*").unwrap();
        let res = doc.resources.clone();
        for (_, value) in res {
            if img_reg.is_match(&value.1) {
                let content = doc.get_raw_by_path(&value.0);
                images.insert(value.0.clone(), Image::new(content));
            }
        }

        // Extract all the pages
        let mut pages = HashMap::new();
        let mut order = vec![];

        let spine = doc.spine.clone();
        for elem in spine {
            let content = doc.get_by_id(&elem);
            let path = Path::new(&doc.resources.get(&elem).unwrap().0);
            let path_string = path.to_str().unwrap().to_string();
            order.push(path_string.clone());

            pages.insert(
                path_string,
                Page::new(
                    content,
                    doc.toc
                        .iter()
                        .find(|x| x.0 == path.to_str().unwrap())
                        .map(|x| x.1.clone()),
                    path.as_ref(),
                ),
            );
        }

        Book {
            title: doc.meta.title.unwrap(),
            cover: doc.meta.cover,
            pages,
            images,
            toc: doc.toc,
            order,
        }
    }
}
