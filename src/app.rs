use crossterm::event::{self, KeyCode, KeyEventKind};
use ratatui::{backend::Backend, widgets::ListState, Terminal};
use tui_scrollview::ScrollViewState;

use crate::{models::book::Book, ui::ui};

pub enum CurrentScreen {
    Info {
        toc_state: ListState,
    },
    Reading {
        page: String,
        content_state: ScrollViewState,
    },
}

pub struct App {
    pub book: Book,
    pub current_screen: CurrentScreen,
    exit: bool,
}

impl App {
    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) {
        while !self.exit {
            terminal.draw(|f| ui(f, self)).unwrap();
            self.handle_event();
        }
    }

    pub fn new(path: &str) -> App {
        App {
            book: Book::new(path),
            exit: false,
            current_screen: CurrentScreen::Info {
                toc_state: ListState::default(),
            },
        }
    }

    fn handle_event(&mut self) {
        match event::read().unwrap() {
            event::Event::Key(key) => {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Char('Q') => self.exit = true, // Global shortcut
                        _ => match &mut self.current_screen {
                            CurrentScreen::Info { toc_state } => match key.code {
                                KeyCode::Enter => {
                                    self.current_screen = CurrentScreen::Reading {
                                        page: if toc_state.selected().is_none() {
                                            self.book.order.first().unwrap().clone()
                                        } else {
                                            self.book
                                                .toc
                                                .get(toc_state.selected().unwrap())
                                                .unwrap()
                                                .0
                                                .clone()
                                        },
                                        content_state: ScrollViewState::new(),
                                    };
                                }
                                KeyCode::Up => {
                                    if toc_state.selected().is_none() && !self.book.toc.is_empty() {
                                        toc_state.select(Some(self.book.toc.len() - 1));
                                    } else if toc_state.selected().unwrap() > 0 {
                                        toc_state.select(Some(toc_state.selected().unwrap() - 1));
                                    } else {
                                        toc_state.select(None);
                                    }
                                }
                                KeyCode::Down => {
                                    if toc_state.selected().is_none() && !self.book.toc.is_empty() {
                                        toc_state.select(Some(0));
                                    } else if toc_state.selected().unwrap()
                                        < (self.book.toc.len() - 1)
                                    {
                                        toc_state.select(Some(toc_state.selected().unwrap() + 1));
                                    } else {
                                        toc_state.select(None);
                                    }
                                }
                                _ => (),
                            },
                            CurrentScreen::Reading { content_state, .. } => match key.code {
                                KeyCode::Up => content_state.scroll_up(),
                                KeyCode::Down => content_state.scroll_down(),
                                _ => (),
                            },
                        },
                    }
                }
            }
            _ => (),
        }
    }
}
