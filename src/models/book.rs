use epub::doc::EpubDoc;
use regex::Regex;
use std::collections::HashMap;

use super::{image::Image, page::Page};

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
        // TODO write my own epub parser
        let doc = EpubDoc::new(path);
        let mut doc = doc.unwrap();

        // Extract the title
        let title = doc.mdata("title").unwrap();

        // Extract the cover
        let mut cover = match doc.get_cover_id() {
            Ok(id) => Option::Some(id),
            Err(_) => Option::None,
        };

        // Extract all the images
        let mut images = HashMap::new();
        let img_reg = Regex::new(r"image/.*").unwrap();
        let res = doc.resources.clone();
        for (key, value) in res {
            if img_reg.is_match(&value.1) {
                let content = doc.get_resource_by_path(&value.0).unwrap();
                let path_string = value.0.to_str().unwrap();

                if cover.as_ref().is_some_and(|x| &key == x) {
                    cover = Some(path_string.to_string());
                }

                images.insert(path_string.to_string(), Image::new(content));
            }
        }

        let mut toc = vec![];
        let remove_hashtag_re = Regex::new(r"#.*$").unwrap();

        for ele in &doc.toc {
            let link = remove_hashtag_re
                .replace_all(ele.content.to_str().unwrap(), "")
                .into_owned();
            toc.push((link, ele.label.clone()));
        }

        // Extract all the pages
        let num_pages = doc.get_num_pages();
        let mut pages = HashMap::new();
        let mut order = vec![];

        for i in 1..=num_pages {
            let content = doc.get_current_str().unwrap();
            let path = doc.get_current_path().unwrap();
            let path_string = path.to_str().unwrap().to_string();
            order.push(path_string.clone());

            pages.insert(
                path_string,
                Page::new(
                    content,
                    toc.iter()
                        .find(|x| x.0 == path.to_str().unwrap())
                        .map(|x| x.1.clone()),
                    path.as_ref(),
                ),
            );

            if i != num_pages {
                doc.go_next().unwrap();
            }
        }

        Book {
            title,
            cover,
            pages,
            images,
            toc,
            order,
        }
    }
}
