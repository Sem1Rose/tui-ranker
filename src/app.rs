use crate::Drawer;
use crate::types::{Term, initialize_terminal};
use log::error;
use ranker::Ranker;
use ratatui::crossterm::event::{self, Event, KeyCode};

pub struct App {
    ranker: Ranker<String>,
    terminal: Term,
    drawer: Drawer,
}

impl App {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            ranker: Ranker::new(),
            terminal: initialize_terminal()?,
            drawer: Drawer::new(),
        })
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        loop {
            self.terminal
                .draw(|frame| {
                    let result = self.drawer.render_app(frame);
                    if let Err(err) = result {
                        error!("Error while drawing: {}", err);
                    }
                })
                .map(|_| ())?;

            if event::poll(std::time::Duration::from_millis(10))? {
                if let Ok(event) = event::read() {
                    if self.handle_event(event)? {
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    fn handle_event(&mut self, event: Event) -> anyhow::Result<bool> {
        match event {
            Event::Key(event) => {
                if event.code == KeyCode::Char('q') {
                    return Ok(true);
                } else if event.code == KeyCode::Char('p') {
                    panic!("PANIC");
                }
            }
            Event::FocusGained => (),
            Event::FocusLost => (),
            Event::Mouse(_) => (),
            Event::Paste(_) => (),
            Event::Resize(_, _) => (),
        }

        Ok(false)
    }
}
