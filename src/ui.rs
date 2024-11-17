use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Padding, Paragraph, Wrap},
    Frame,
};
use ratatui_image::{picker::Picker, StatefulImage};
use tui_scrollview::ScrollView;

use crate::{
    app::{App, CurrentScreen},
    models::page::{ContentType, TextStyle},
};

pub fn ui(frame: &mut Frame, app: &mut App) {
    match app.current_screen {
        CurrentScreen::Info { .. } => render_info(frame, app),
        CurrentScreen::Reading { .. } => render_reading(frame, app),
    }
}

fn render_info(frame: &mut Frame, app: &mut App) {
    let instruction_chunk = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(100),
            Constraint::Length(1), // Instructions
        ])
        .split(frame.area());
    let instructions = Paragraph::new("[Up/Down ► Navigate] [Enter ► Start Reading] [Q ► Quit]")
        .style(Style::default().light_yellow())
        .alignment(Alignment::Center);
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

    if let CurrentScreen::Info { toc_state } = &mut app.current_screen {
        frame.render_stateful_widget(toc, right_chunks[1], toc_state);
    } else {
        frame.render_widget(toc, right_chunks[1]);
    }

    // Render Cover
    if app.book.cover.is_some() {
        let dyn_img = app
            .book
            .images
            .get_mut(&app.book.cover.clone().unwrap())
            .unwrap()
            .get();

        let mut picker = Picker::from_query_stdio().unwrap();
        let mut protocol = picker.new_resize_protocol(dyn_img.clone());

        let image = StatefulImage::new(None);

        frame.render_stateful_widget(image, chunks[0], &mut protocol);
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

        // Render the centered paragraph inside the adjusted area
        frame.render_widget(centered_paragraph, centered_area);
    }
}

fn render_reading(frame: &mut Frame, app: &mut App) {
    let mut page = None;
    let mut state = None;
    if let CurrentScreen::Reading {
        page: path,
        content_state,
    } = &mut app.current_screen
    {
        page = app.book.pages.get(path);
        state = Some(content_state)
    }
    let page = page.unwrap();
    let state = state.unwrap();

    let instruction_chunk = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100), Constraint::Length(1)])
        .split(frame.area());
    let instructions = Paragraph::new("[Up/Down ► Navigate] [Q ► Quit]")
        .style(Style::default().light_yellow())
        .alignment(Alignment::Center);
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
            ContentType::Image { path } => content.push(Span::raw(format!("[Image]({})", path))),
            ContentType::Img { path } => content.push(Span::raw(format!("[Img]({})", path))),
            ContentType::LineBreak => {
                lines.push(Line::from(content.clone()));
                content.clear();
            }
        }
    }

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true });

    let mut content_size = inner_area.as_size();
    content_size.width -= 2;
    content_size.height = paragraph.line_count(content_size.width) as u16;
    let mut scroll_view: ScrollView = ScrollView::new(content_size);
    scroll_view.render_widget(
        paragraph,
        Rect::new(0, 0, content_size.width, content_size.height),
    );
    frame.render_stateful_widget(scroll_view, inner_area, state);
}
