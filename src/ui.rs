use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Padding, Paragraph, Wrap},
    Frame,
};
use ratatui_image::{CropOptions, Resize};

use crate::{
    app::{App, Screen},
    models::page::{ContentType, TextStyle},
    widgets::custom_thread_image::{ThreadImage, ThreadProtocol},
};

pub fn ui(frame: &mut Frame, app: &mut App) {
    match app.current_screen {
        Screen::Info { .. } => render_info(frame, app),
        Screen::Reading { .. } => render_reading(frame, app),
    }
}

fn render_info(frame: &mut Frame, app: &mut App) {
    let instruction = match &app.current_screen {
        Screen::Info { prev_screen, .. } if prev_screen.is_none() => {
            "[Up/Down ► Navigate] [Enter ► Start Reading] [Q ► Quit]"
        }
        Screen::Info { .. } => {
            "[Esc ► Return] [Up/Down ► Navigate] [Enter ► Start Reading] [Q ► Quit]"
        }
        _ => unreachable!(),
    };
    let instructions = Paragraph::new(instruction)
        .style(Style::default().light_yellow())
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
        .block(Block::new().padding(Padding::horizontal(2)));
    let instructions_line = instructions.line_count(frame.area().width) as u16;
    let instruction_chunk = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(100),
            Constraint::Length(instructions_line),
        ])
        .split(frame.area());
    frame.render_widget(instructions, instruction_chunk[1]);

    let main_block = Block::default()
        .borders(Borders::ALL)
        .title(" 📔 Book Info ")
        .padding(Padding::symmetric(2, 1));
    let main_area = instruction_chunk[0];
    let inner_area = main_block.inner(main_area);
    frame.render_widget(main_block, main_area);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Length(2),
            Constraint::Percentage(70),
        ])
        .split(inner_area);

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(0)])
        .split(chunks[2]);

    let title_block = Paragraph::new(app.book.title.as_str())
        .style(Style::new().bold().light_magenta())
        .wrap(Wrap { trim: false });
    frame.render_widget(title_block, right_chunks[0]);

    // Render TOC
    let mut contents = vec![];
    for ele in &app.book.toc {
        contents.push(ListItem::from(Span::styled(
            ele.1.to_string(),
            Style::new().underlined().bold().light_blue(),
        )));
    }

    let toc = List::new(contents)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::new().light_cyan())
                .padding(Padding::horizontal(1))
                .title("Table of Contents")
                .title_style(Style::new().light_cyan()),
        )
        .highlight_style(Style::default().bg(Color::LightCyan).fg(Color::Black))
        .highlight_symbol(" ► ");

    if let Screen::Info { toc_state, .. } = &mut app.current_screen {
        frame.render_stateful_widget(toc, right_chunks[1], toc_state);
    } else {
        frame.render_widget(toc, right_chunks[1]);
    }

    // Render Cover
    if app.book.cover.is_some() {
        let cover_path = app.book.cover.clone().unwrap();
        let cover_state = app.image_state.get_mut(&cover_path);
        if cover_state.is_none() {
            let dyn_img = app.book.images.get_mut(&cover_path).unwrap().get();

            app.image_state.insert(
                cover_path,
                ThreadProtocol::new(
                    app.tx_worker.clone(),
                    app.picker.new_resize_protocol(dyn_img.clone()),
                ),
            );
        } else {
            let image = ThreadImage::new(cover_path);
            frame.render_stateful_widget(image, chunks[0], cover_state.unwrap());
        }
    } else {
        let block_widget = Block::default().borders(Borders::ALL);
        let block_content_area = block_widget.inner(chunks[0]);

        frame.render_widget(block_widget, chunks[0]);

        let centered_paragraph = Paragraph::new("No Cover").alignment(Alignment::Center);
        let vertical_padding = (block_content_area.height.saturating_sub(1)) / 2;
        let centered_area = Rect {
            x: block_content_area.x,
            y: block_content_area.y + vertical_padding,
            width: block_content_area.width,
            height: block_content_area.height.saturating_sub(vertical_padding),
        };

        frame.render_widget(centered_paragraph, centered_area);
    }
}

enum WidgetType<'a> {
    Paragraph(Paragraph<'a>),
    Image(String),
}

fn render_reading(frame: &mut Frame, app: &mut App) {
    let Screen::Reading { page, offset } = &mut app.current_screen else {
        unreachable!()
    };
    let page = app.book.pages.get(page).unwrap();

    let instructions = Paragraph::new(
        "[I ► Book Info] [Up/Down ► Scroll] [Left/Right ► Navigate Between Chapters] [Q ► Quit]",
    )
    .style(Style::default().light_yellow())
    .alignment(Alignment::Center)
    .wrap(Wrap { trim: true })
    .block(Block::new().padding(Padding::horizontal(2)));
    let instructions_line = instructions.line_count(frame.area().width) as u16;
    let instruction_chunk = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(100),
            Constraint::Length(instructions_line),
        ])
        .split(frame.area());
    frame.render_widget(instructions, instruction_chunk[1]);

    let main_block = Block::default()
        .borders(Borders::ALL)
        .title(format!("{}{}", " 📖 Reading - ", page.title))
        .padding(Padding::symmetric(2, 1));
    let main_area = instruction_chunk[0];
    let inner_area = main_block.inner(main_area);
    frame.render_widget(main_block, main_area);

    let mut lines = vec![];
    let mut content = vec![];
    let mut widgets: Vec<WidgetType> = vec![];

    let mut heights: Vec<usize> = vec![];
    let mut total_height: usize = 0;

    macro_rules! push_paragraph {
        ($text:expr) => {{
            let paragraph = Paragraph::new($text).wrap(Wrap { trim: true });

            let line_count = paragraph.line_count(inner_area.width);
            total_height += line_count;
            heights.push(line_count);
            widgets.push(WidgetType::Paragraph(paragraph));
        }};
    }

    for i in &page.content {
        match i {
            ContentType::Text { text, style, .. } => {
                let mut style_ = Style::new();
                match style {
                    TextStyle::Bold => style_ = style_.bold(),
                    TextStyle::Italic => style_ = style_.italic(),
                    TextStyle::Underline => style_ = style_.underlined(),
                    _ => (),
                }
                content.push(Span::styled(text, style_));
            }
            ContentType::Image(path) | ContentType::Img(path) => {
                if !content.is_empty() {
                    lines.push(Line::from(content.clone()));
                    content.clear();
                }

                if !lines.is_empty() {
                    push_paragraph!(lines.clone());
                    lines.clear();
                }

                total_height += inner_area.height as usize;
                heights.push(inner_area.height as usize);
                widgets.push(WidgetType::Image(path.clone()));
            }
            ContentType::LineBreak => {
                lines.push(Line::from(content.clone()));
                content.clear();
            }
        }
    }

    if !content.is_empty() {
        lines.push(Line::from(content));
    }

    if !lines.is_empty() {
        push_paragraph!(lines);
    }

    // Clamp the offset to make sure it does not exceed the total height of the content
    if total_height > inner_area.height as usize {
        *offset = (*offset).min(total_height - inner_area.height as usize);
    } else {
        *offset = 0;
    }

    let mut reduce_height = 0;
    let mut current_widget = 0;

    let Rect { x, y, height, .. } = inner_area;
    let base_x = x as usize;
    let base_y = y as usize;
    let base_height = height as usize;

    while reduce_height < *offset + base_height && current_widget < heights.len() {
        let height = *heights.get(current_widget).unwrap();
        let mut visible_bottom = false;

        if reduce_height + height > *offset {
            let (y, widget_height, scroll_offset) = if reduce_height < *offset {
                // Partially visible at the top
                let visible_height = (height - (*offset - reduce_height)).min(base_height);
                (base_y, visible_height, *offset - reduce_height)
            } else if reduce_height + height > *offset + base_height {
                // Partially visible at the bottom
                let visible_height = *offset + base_height - reduce_height;
                visible_bottom = true;
                (base_y + reduce_height - *offset, visible_height, 0)
            } else {
                // Fully visible
                (base_y + reduce_height - *offset, height, 0)
            };
            let rect = Rect {
                x: base_x as u16,
                y: y as u16,
                height: widget_height as u16,
                ..inner_area
            };

            match &widgets[current_widget] {
                WidgetType::Paragraph(paragraph) => {
                    frame.render_widget(paragraph.clone().scroll((scroll_offset as u16, 0)), rect);
                }
                WidgetType::Image(path) => {
                    let state = app.image_state.get_mut(path);

                    if state.is_some() {
                        let thr_img = ThreadImage::new(path.clone()).resize(Resize::Crop(Some(
                            CropOptions {
                                clip_left: false,
                                clip_top: !visible_bottom,
                            },
                        )));
                        let img = app.book.images.get(path).unwrap();

                        frame.render_stateful_widget(
                            thr_img,
                            Rect {
                                x: (rect.width / 2 - img.cal_width(height)).max(rect.x),
                                ..rect
                            },
                            state.unwrap(),
                        );
                    } else {
                        let dyn_img = app.book.images.get_mut(path).unwrap().get();

                        app.image_state.insert(
                            path.clone(),
                            ThreadProtocol::new(
                                app.tx_worker.clone(),
                                app.picker.new_resize_protocol(dyn_img.clone()),
                            ),
                        );
                    }
                }
            }
        }

        reduce_height += height;
        current_widget += 1;
    }
}
