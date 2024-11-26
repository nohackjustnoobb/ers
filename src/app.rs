use std::{
    collections::HashMap,
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::Duration,
};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    backend::Backend,
    layout::{Position, Rect},
    widgets::ListState,
    Terminal,
};
use ratatui_image::{picker::Picker, protocol::StatefulProtocol, Resize};
use tui_scrollview::ScrollViewState;

use crate::{models::book::Book, ui::ui, widgets::custom_thread_image::ThreadProtocol};

#[derive(Clone)]
pub struct ReadingRecord {
    page: String,
    offset: Position,
}

pub enum Screen {
    Info {
        toc_state: ListState,
        prev_screen: Option<ReadingRecord>,
    },
    Reading {
        page: String,
        content_state: ScrollViewState,
    },
}

enum AppEvent {
    KeyEvent(KeyEvent),
    Redraw(String, StatefulProtocol),
}

pub struct App {
    pub book: Book,
    pub current_screen: Screen,
    pub picker: Picker,
    pub image_state: HashMap<String, ThreadProtocol>,
    pub tx_worker: Sender<(String, StatefulProtocol, Resize, Rect)>,
    exit: bool,
    rec_main: Receiver<AppEvent>,
}

impl App {
    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) {
        while !self.exit {
            terminal.draw(|f| ui(f, self)).unwrap();
            self.handle_event();
        }
    }

    pub fn new(path: &str) -> App {
        let picker = Picker::from_query_stdio().unwrap();

        let (tx_worker, rec_worker) = mpsc::channel::<(String, StatefulProtocol, Resize, Rect)>();
        let (tx_main, rec_main) = mpsc::channel();

        let tx_main_render = tx_main.clone();
        thread::spawn(move || loop {
            if let Ok((id, mut protocol, resize, area)) = rec_worker.recv() {
                protocol.resize_encode(&resize, None, area);
                tx_main_render.send(AppEvent::Redraw(id, protocol)).unwrap();
            }
        });

        thread::spawn(move || -> Result<(), std::io::Error> {
            loop {
                if ratatui::crossterm::event::poll(Duration::from_millis(1000)).unwrap() {
                    if let Event::Key(key) = event::read().unwrap() {
                        tx_main.send(AppEvent::KeyEvent(key)).unwrap();
                    }
                }
            }
        });

        App {
            book: Book::new(path),
            exit: false,
            current_screen: Screen::Info {
                toc_state: ListState::default(),
                prev_screen: None,
            },
            tx_worker,
            rec_main,
            picker,
            image_state: HashMap::new(),
        }
    }

    fn handle_event(&mut self) {
        let result = self.rec_main.try_recv();
        if result.is_err() {
            return;
        }

        match result.unwrap() {
            AppEvent::Redraw(id, proto) => {
                let state = self.image_state.get_mut(&id);

                if state.is_some() {
                    state.unwrap().set_protocol(proto);
                }
            }
            AppEvent::KeyEvent(key) => {
                if key.kind == KeyEventKind::Press {
                    self.handle_keypress(key.code);
                }
            }
        }
    }

    fn handle_keypress(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('q') | KeyCode::Char('Q') => self.exit = true, // Global shortcut
            _ => match &mut self.current_screen {
                Screen::Info {
                    toc_state,
                    prev_screen,
                    ..
                } => match code {
                    KeyCode::Enter => {
                        self.current_screen = Screen::Reading {
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
                    KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => {
                        if self.book.toc.is_empty() {
                            return;
                        }

                        if toc_state.selected().is_none() {
                            toc_state.select(Some(self.book.toc.len() - 1));
                        } else if toc_state.selected().unwrap() > 0 {
                            toc_state.select(Some(toc_state.selected().unwrap() - 1));
                        } else {
                            toc_state.select(None);
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => {
                        if self.book.toc.is_empty() {
                            return;
                        }

                        if toc_state.selected().is_none() {
                            toc_state.select(Some(0));
                        } else if toc_state.selected().unwrap() < (self.book.toc.len() - 1) {
                            toc_state.select(Some(toc_state.selected().unwrap() + 1));
                        } else {
                            toc_state.select(None);
                        }
                    }
                    KeyCode::Esc => {
                        if prev_screen.is_some() {
                            let prev_screen = prev_screen.as_ref().unwrap();

                            self.current_screen = Screen::Reading {
                                page: prev_screen.page.clone(),
                                content_state: ScrollViewState::with_offset(prev_screen.offset),
                            }
                        }
                    }
                    _ => (),
                },
                Screen::Reading {
                    page,
                    content_state,
                } => match code {
                    KeyCode::Char('i') | KeyCode::Char('I') => {
                        self.current_screen = Screen::Info {
                            toc_state: ListState::default(),
                            prev_screen: Some(ReadingRecord {
                                page: page.clone(),
                                offset: content_state.offset(),
                            }),
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => {
                        content_state.scroll_up();
                    }
                    KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => {
                        content_state.scroll_down();
                    }
                    KeyCode::Left | KeyCode::Char('h') | KeyCode::Char('H') => {
                        let current_index = self.book.order.iter().position(|x| x == page).unwrap();

                        if current_index > 0 {
                            self.current_screen = Screen::Reading {
                                page: self.book.order.get(current_index - 1).unwrap().to_string(),
                                content_state: ScrollViewState::new(),
                            };
                        }
                    }
                    KeyCode::Right | KeyCode::Char('l') | KeyCode::Char('L') => {
                        let current_index = self.book.order.iter().position(|x| x == page).unwrap();

                        if current_index < self.book.order.len() - 1 {
                            self.current_screen = Screen::Reading {
                                page: self.book.order.get(current_index + 1).unwrap().to_string(),
                                content_state: ScrollViewState::new(),
                            };
                        }
                    }
                    _ => (),
                },
            },
        }
    }
}
