use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect, Size},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Padding, Paragraph, Wrap},
    Frame,
};
use tui_scrollview::ScrollView;

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
    Paragraph(Paragraph<'a>, u16),
    Image(String, u16),
}

fn render_reading(frame: &mut Frame, app: &mut App) {
    let Screen::Reading {
        page,
        content_state: state,
    } = &mut app.current_screen
    else {
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

    let content_size_width = inner_area.width - 2;
    let mut total_height = 0;

    macro_rules! push_paragraph {
        ($text:expr) => {{
            let paragraph = Paragraph::new($text).wrap(Wrap { trim: true });

            let line_count = paragraph.line_count(content_size_width);
            total_height += line_count;
            widgets.push(WidgetType::Paragraph(paragraph, line_count as u16));
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

                push_paragraph!(format!("[Image]({})", path));
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

    let mut scroll_view: ScrollView =
        ScrollView::new(Size::new(content_size_width, total_height as u16));

    let mut reduce_height = 0;
    for widget in widgets {
        match widget {
            WidgetType::Paragraph(paragraph, height) => {
                scroll_view.render_widget(
                    paragraph,
                    Rect::new(0, reduce_height, content_size_width, height as u16),
                );
                reduce_height += height;
            }
            WidgetType::Image(..) => (),
        }
    }

    frame.render_stateful_widget(scroll_view, inner_area, state);
}
