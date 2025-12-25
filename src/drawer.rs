use anyhow::Ok;
use ratatui::Frame;

use crate::image_backend::RatatuiImage;
use crate::popups::*;
use crate::screens::*;

pub struct Drawer {
    image_backend: RatatuiImage,

    current_screen: Option<Screens>,
    active_popup: Option<Popups>,
}
impl Drawer {
    pub fn new() -> Self {
        Drawer {
            image_backend: RatatuiImage::new(),
            current_screen: None,
            active_popup: Some(Popups::ProjectSelect(ProjectSelect::new())),
        }
    }

    pub fn render_app(&mut self, frame: &mut Frame) -> anyhow::Result<()> {
        self.image_backend.update();

        self.draw_current_screen(frame)?;

        self.check_popups()?;
        if self.active_popup.is_some() {
            self.draw_popup(frame)?;
        }

        Ok(())
    }

    fn draw_current_screen(&mut self, frame: &mut Frame) -> anyhow::Result<()> {
        if self.current_screen.is_none() {
            return Ok(());
        }

        match self.current_screen.as_mut().unwrap() {
            Screens::MainScreen(main_screen) => {
                main_screen.render(frame)?;
            }
            _ => {}
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

    fn draw_popup(&mut self, frame: &mut Frame) -> anyhow::Result<()> {
        if self.active_popup.is_none() {
            return Ok(());
        }

        match self.active_popup.as_mut().unwrap() {
            Popups::ProjectSelect(project_select) => {
                project_select.render(frame)?;
            }
            _ => {}
        }

        Ok(())
    }
}
