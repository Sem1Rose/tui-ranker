use anyhow::Ok;
use ranker::Ranker;
use ratatui::Frame;
use ratatui::style::{Stylize, palette::tailwind::*};
use ratatui::widgets::Block;

use crate::image_backend::RatatuiImage;
use crate::popups::*;
use crate::screens::*;

pub struct Drawer {
    image_backend: RatatuiImage,

    current_screen: Option<Screens>,
    active_popup: Option<Popups>,

    main_screen: MainScreen,
}

impl Drawer {
    pub fn new() -> Self {
        Drawer {
            image_backend: RatatuiImage::new(),
            current_screen: None,
            active_popup: Some(Popups::default()),
            main_screen: MainScreen::default(),
        }
    }

    pub fn render_app(&mut self, frame: &mut Frame, ranker: &Ranker<String>) -> anyhow::Result<()> {
        self.image_backend.update();

        self.draw_current_screen(frame)?;

        self.check_popups()?;
        if self.active_popup.is_some() {
            self.draw_popup(frame, ranker)?;
        }

        Ok(())
    }

    fn draw_current_screen(&mut self, frame: &mut Frame) -> anyhow::Result<()> {
        if let Some(current_screen) = self.current_screen.as_ref() {
            match current_screen {
                Screens::MainScreen => {
                    // self.main_screen.render(frame)?;
                }
                _ => {}
            }
        } else {
            frame.render_widget(Block::new().bg(SLATE.c900), frame.area());
        }

        Ok(())
    }

    fn check_popups(&mut self) -> anyhow::Result<()> {
        if let Some(popup) = self.active_popup.as_mut() {
            match popup {
                Popups::ProjectSelect(project_select_popup) => match project_select_popup.phase {
                    ProjectSelectPhase::Done => {}
                    _ => {}
                },
                _ => {}
            }
        }

        Ok(())
    }

    fn draw_popup(&mut self, frame: &mut Frame, ranker: &Ranker<String>) -> anyhow::Result<()> {
        if let Some(active_popup) = self.active_popup.as_mut() {
            match active_popup {
                Popups::ProjectSelect(project_select) => {
                    project_select.render(frame, ranker)?;
                }
                _ => {}
            }
        } else {
        }

        Ok(())
    }

    fn close_popups(&mut self) {
        self.active_popup = None
    }

    fn open_main_screen(&mut self) {
        self.close_popups();

        self.current_screen = Some(Screens::MainScreen);
    }
}
