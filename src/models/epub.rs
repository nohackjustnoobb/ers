use std::{
    collections::HashMap,
    fs::{self, File},
    io::Read,
    path::Path,
};

use regex::Regex;
use zip::ZipArchive;

pub struct Metadata {
    pub title: Option<String>,
    pub cover: Option<String>,
    pub language: Option<String>,
    pub creator: Vec<String>,
    pub publisher: Vec<String>,
    pub identifier: Vec<String>,
}

impl Metadata {
    pub fn new() -> Metadata {
        Metadata {
            title: None,
            cover: None,
            language: None,
            creator: vec![],
            publisher: vec![],
            identifier: vec![],
        }
    }
}

pub struct EpubDoc {
    archive: ZipArchive<File>,
    pub meta: Metadata,
    // Key is the id of the resource, and Value is the path to the resource and the type of the resource
    pub resources: HashMap<String, (String, String)>,
    // A vector of id
    pub spine: Vec<String>,
    // The first string is the path, and the second string is the title
    pub toc: Vec<(String, String)>,
}

impl EpubDoc {
    pub fn new(path: &str) -> Self {
        let fname = std::path::Path::new(path);
        let file = fs::File::open(fname).unwrap();

        let mut archive = zip::ZipArchive::new(file).unwrap();

        // get the metadata of epub file
        let mut content = String::new();
        {
            let meta = &mut archive
                .by_name("META-INF/container.xml")
                .expect("A valid EPUB format");
            meta.read_to_string(&mut content).unwrap();
        }
        let meta = roxmltree::Document::parse(content.as_str()).unwrap();

        // get the path of content.opf
        let root_file = meta
            .descendants()
            .find(|n| n.has_tag_name("rootfile"))
            .expect("Only one root file");
        let content_opf_path = root_file
            .attribute("full-path")
            .expect("A path to content.opf");

        // get the content.opf
        let mut content = String::new();
        {
            let content_opf = &mut archive
                .by_name(content_opf_path)
                .expect("A context.opf file");
            content_opf.read_to_string(&mut content).unwrap();
        }
        let content_opf: roxmltree::Document<'_> =
            roxmltree::Document::parse(content.as_str()).unwrap();

        // parse metadata of the book
        let mut parsed_meta = Metadata::new();
        let meta = content_opf
            .descendants()
            .find(|n| n.has_tag_name("metadata"))
            .expect("Atleast one metadata");
        for ele in meta.children() {
            match ele.tag_name().name() {
                "title" => parsed_meta.title = Some(ele.text().unwrap().to_string()),
                "language" => parsed_meta.language = Some(ele.text().unwrap().to_string()),
                "identifier" => parsed_meta.identifier.push(ele.text().unwrap().to_string()),
                "publisher" => parsed_meta.publisher.push(ele.text().unwrap().to_string()),
                "creator" => parsed_meta.creator.push(ele.text().unwrap().to_string()),
                "meta" => {
                    if ele.attribute("name").is_some_and(|n| n == "cover") {
                        parsed_meta.cover = Some(ele.attribute("content").unwrap().to_string())
                    }
                }
                _ => (),
            }
        }

        // parse all the documents in the epub
        let mut base_path = Path::new(content_opf_path)
            .parent()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        if !base_path.is_empty() {
            base_path = format!("{}/", base_path);
        }
        let mut resources = HashMap::new();
        let manifest = content_opf
            .descendants()
            .find(|n| n.has_tag_name("manifest"))
            .expect("At least one manifest");
        for ele in manifest.children() {
            if ele.is_element() {
                let key = ele.attribute("id").unwrap().to_string();
                let path = base_path.clone() + ele.attribute("href").unwrap();

                if parsed_meta.cover.clone().is_some_and(|x| x == key) {
                    parsed_meta.cover = Some(path.clone());
                }

                resources.insert(
                    key,
                    (path, ele.attribute("media-type").unwrap().to_string()),
                );
            }
        }

        // parse the spine
        let spine: Vec<String> = content_opf
            .descendants()
            .find(|n| n.has_tag_name("spine"))
            .expect("Atleast one spine")
            .children()
            .filter(|n| n.is_element())
            .map(|n| n.attribute("idref").unwrap().to_string())
            .collect();

        // parse toc
        let mut toc = vec![];
        if resources.get("ncx").is_some() {
            let mut content = String::new();
            let toc_ncx = &mut archive
                .by_name(resources.get("ncx").unwrap().0.as_str())
                .unwrap();
            toc_ncx.read_to_string(&mut content).unwrap();
            let toc_ncx: roxmltree::Document<'_> =
                roxmltree::Document::parse(content.as_str()).unwrap();

            let nav_map = toc_ncx
                .descendants()
                .find(|n| n.has_tag_name("navMap"))
                .unwrap();
            let re = Regex::new(r"#.*$").unwrap();
            for ele in nav_map.children() {
                if ele.is_element() {
                    let path = ele
                        .descendants()
                        .find(|n| n.has_tag_name("content"))
                        .unwrap()
                        .attribute("src")
                        .unwrap()
                        .to_string();
                    let path = base_path.clone() + re.replace_all(&path, "").into_owned().as_str();

                    toc.push((
                        path,
                        ele.descendants()
                            .find(|n| n.has_tag_name("text"))
                            .unwrap()
                            .text()
                            .unwrap()
                            .to_string(),
                    ));
                }
            }
        }

        EpubDoc {
            meta: parsed_meta,
            resources,
            archive,
            spine,
            toc,
        }
    }

    pub fn get_by_id(&mut self, id: &String) -> String {
        self.get_by_path(&self.resources.get(id).unwrap().0.clone())
    }

    pub fn get_by_path(&mut self, path: &String) -> String {
        let mut content = String::new();

        let mut file = self.archive.by_name(path).unwrap();
        file.read_to_string(&mut content).unwrap();

        content
    }

    pub fn get_raw_by_path(&mut self, path: &String) -> Vec<u8> {
        let mut content = vec![];

        let mut file = self.archive.by_name(path).unwrap();
        file.read_to_end(&mut content).unwrap();

        content
    }
}
