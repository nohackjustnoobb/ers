use path_clean::PathClean;
use regex::Regex;
use roxmltree::{Document, Node, ParsingOptions};
use std::path::Path;

pub enum TextStyle {
    Regular,
    Bold,
    Italic,
    Underline,
}

pub enum ContentType {
    LineBreak,
    Text {
        text: String,
        style: TextStyle,
        hints: Option<String>,
        href: Option<String>,
    },
    Image {
        path: String,
    },
    Img {
        path: String,
    },
}

pub struct Page {
    pub title: String,
    pub content: Vec<ContentType>,
}

impl Page {
    fn parse_image_or_img(node: Node, path: &Path) -> ContentType {
        let href = node
            .attributes()
            .find(|e| e.name() == "href" || e.name() == "src")
            .unwrap()
            .value();

        let rel_path = Path::new(href);
        let root_path = path.parent().unwrap().join(rel_path).clean();
        let string_path = root_path.to_str().unwrap().to_string();

        if node.tag_name().name() == "img" {
            ContentType::Img { path: string_path }
        } else {
            ContentType::Image { path: string_path }
        }
    }

    fn parse_text(node: Node, style: TextStyle) -> Option<ContentType> {
        let text = node
            .text()
            .unwrap_or_default()
            .replace("\n", "")
            .trim()
            .to_string();

        (!text.is_empty()).then_some(ContentType::Text {
            text,
            style,
            hints: None,
            href: None,
        })
    }

    // FIXME not working
    fn parse_a(node: Node, path: &Path) -> Option<ContentType> {
        let href = node.attribute("href").unwrap();

        let rel_path = Path::new(href);
        let root_path = path.parent().unwrap().join(rel_path).clean();
        let string_path = root_path.to_str().unwrap().to_string();

        let re = Regex::new(r"#.*$").unwrap();
        let link = re.replace_all(&string_path, "").into_owned();

        let text = node.text().unwrap_or_default().replace("\n", "");
        (!text.is_empty()).then_some(ContentType::Text {
            text,
            style: TextStyle::Underline,
            hints: None,
            href: Some(link),
        })
    }

    fn parse_ruby(node: Node) -> Vec<ContentType> {
        let mut rb = Vec::new();
        let mut rt = Vec::new();

        for ele in node.children() {
            match ele.tag_name().name() {
                "rb" => rb.push(ele.text().unwrap().to_string()),
                "rt" => rt.push(ele.text().unwrap().to_string()),
                _ => (),
            }
        }

        let mut result = vec![];
        for i in 0..rb.len() {
            result.push(ContentType::Text {
                text: rb.get(i).unwrap().to_string(),
                style: TextStyle::Regular,
                hints: Some(rt.get(i).unwrap().to_string()),
                href: None,
            });
        }

        result
    }

    fn parse_children(node: Node, path: &Path) -> Vec<ContentType> {
        let mut result = Vec::new();

        if node.has_children() {
            for e in node.children() {
                match e.tag_name().name() {
                    "div" | "svg" | "span" => result.extend(Page::parse_children(e, path)),
                    "p" => {
                        let parsed = Page::parse_children(e, path);
                        let show_push = parsed
                            .iter()
                            .find(|v| matches!(v, ContentType::Text { .. }))
                            .is_some();

                        result.extend(parsed);

                        if show_push {
                            result.push(ContentType::LineBreak);
                        }
                    }
                    "br" => result.push(ContentType::LineBreak),
                    "ruby" => result.extend(Page::parse_ruby(e)),
                    "a" => Page::parse_a(e, path)
                        .map(|v| result.push(v))
                        .unwrap_or_default(),
                    "image" | "img" => result.push(Page::parse_image_or_img(e, path)),
                    "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "i" | "u" | "" => Page::parse_text(
                        e,
                        match e.tag_name().name() {
                            "b" => TextStyle::Bold,
                            "i" => TextStyle::Italic,
                            "u" => TextStyle::Underline,
                            "" => TextStyle::Regular,
                            _ => TextStyle::Bold,
                        },
                    )
                    .map(|v| result.push(v))
                    .unwrap_or_default(),

                    _ => println!("Unsupported Tag Name: <{}>", e.tag_name().name()),
                }
            }
        }

        result
    }

    pub fn new(content: String, title: Option<String>, path: &Path) -> Page {
        let doc = Document::parse_with_options(
            content.as_str(),
            ParsingOptions {
                allow_dtd: true,
                ..Default::default()
            },
        )
        .unwrap();
        let root = doc.root_element();
        let body = root.children().find(|e| e.has_tag_name("body")).unwrap();
        let parsed = Page::parse_children(body, path);

        let in_doc_title = root
            .descendants()
            .find(|x| x.tag_name().name() == "title")
            .unwrap()
            .text()
            .unwrap()
            .to_string();

        Page {
            title: title.unwrap_or(in_doc_title),
            content: parsed,
        }
    }

    #[allow(unused)]
    pub fn print(&self) {
        for i in &self.content {
            match i {
                ContentType::Text {
                    text,
                    style: _,
                    hints,
                    href,
                } => {
                    if href.is_none() {
                        if hints.is_none() {
                            print!("{}", text)
                        } else {
                            print!("{}{{{}}}", text, hints.clone().unwrap())
                        }
                    } else {
                        print!("[{}]({})", text, href.clone().unwrap())
                    }
                }
                ContentType::Image { path } => println!("[Image]({})", path),
                ContentType::Img { path } => println!("[Img]({})", path),
                ContentType::LineBreak => println!(),
            }
        }
    }
}
