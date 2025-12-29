use crate::{KeyEventHandler, image_backend::RatatuiImage, popups::*, screens::*};
use ranker::Ranker;
use ratatui::{
    Frame,
    layout::Constraint,
    style::{Stylize, palette::tailwind::*},
    text::{Line, Text},
    widgets::Block,
};

pub struct Drawer {
    pub image_backend: RatatuiImage,

    pub current_screen: Option<Screens>,
    pub active_popup: Option<Popups>,

    pub main_screen: MainScreen,

    show_term_size_warning: bool,

    refresh_immediate: u8,
}

const MINTERMSIZE: [u32; 2] = [100, 30];
impl Drawer {
    pub fn new() -> Self {
        Drawer {
            image_backend: RatatuiImage::new(),
            current_screen: None,
            active_popup: Some(Popups::default()),
            main_screen: MainScreen::default(),
            show_term_size_warning: false,
            refresh_immediate: 0,
        }
    }

    pub fn render_app(
        &mut self,
        frame: &mut Frame,
        ranker: &Ranker<String>,
        key_event_handler: &mut KeyEventHandler,
    ) -> anyhow::Result<()> {
        self.refresh_immediate = self.refresh_immediate.saturating_sub(1);

        self.check_term_size(frame);
        self.image_backend.update();

        self.draw_current_screen(frame, ranker, key_event_handler)?;

        self.check_popups()?;
        if !self.show_term_size_warning && self.active_popup.is_some() {
            self.draw_popup(frame, ranker, key_event_handler)?;
        }

        Ok(())
    }

    fn draw_current_screen(
        &mut self,
        frame: &mut Frame,
        ranker: &Ranker<String>,
        key_event_handler: &mut KeyEventHandler,
    ) -> anyhow::Result<()> {
        frame.render_widget(Block::new().bg(SLATE.c900), frame.area());

        if self.show_term_size_warning {
            self.render_term_size_warning(frame);
        } else if let Some(current_screen) = self.current_screen.as_ref() {
            match current_screen {
                Screens::MainScreen => {
                    self.main_screen.render(
                        frame,
                        ranker,
                        key_event_handler,
                        &mut self.image_backend,
                    )?;
                }
            }
        }

        Ok(())
    }

    fn check_popups(&mut self) -> anyhow::Result<()> {
        if let Some(popup) = self.active_popup.as_mut() {
            match popup {
                Popups::ProjectSelect(project_select_popup) => match project_select_popup.phase {
                    ProjectSelectPhase::Done => {
                        self.open_main_screen();
                    }
                    _ => {}
                },
            }
        }

        Ok(())
    }

    fn draw_popup(
        &mut self,
        frame: &mut Frame,
        ranker: &Ranker<String>,
        key_event_handler: &mut KeyEventHandler,
    ) -> anyhow::Result<()> {
        if let Some(active_popup) = self.active_popup.as_mut() {
            match active_popup {
                Popups::ProjectSelect(project_select) => {
                    project_select.render(
                        frame,
                        ranker,
                        key_event_handler,
                        self.current_screen.is_some(),
                    )?;
                }
            }
        } else {
        }

        Ok(())
    }

    pub fn close_popups(&mut self) {
        self.active_popup = None;

        self.refresh_immediate += 2;
    }

    pub fn open_main_screen(&mut self) {
        self.close_popups();

        self.current_screen = Some(Screens::MainScreen);
    }

    pub fn check_refresh_immediate(&mut self) -> bool {
        self.refresh_immediate > 0
    }
    pub fn check_refresh_delayed(&mut self) -> bool {
        self.main_screen.drawing_images
    }

    fn check_term_size(&mut self, frame: &Frame) {
        self.show_term_size_warning = (frame.area().width as u32) < MINTERMSIZE[0]
            || (frame.area().height as u32) < MINTERMSIZE[1];
    }

    fn render_term_size_warning(&mut self, frame: &mut Frame) {
        let frame_area = frame.area();
        let lines = vec![
            Line::from_iter([
                "Terminal is too small: ".into(),
                frame_area.width.to_string().red(),
                "x".into(),
                frame_area.height.to_string().red(),
            ]),
            Line::default(),
            Line::from_iter([
                "Minimum size is: ".into(),
                MINTERMSIZE[0].to_string().green(),
                "x".into(),
                MINTERMSIZE[1].to_string().green(),
            ]),
        ];
        let area = crate::helpers::center_rect(
            frame_area,
            Constraint::Min(0),
            Constraint::Length(lines.len() as u16),
        );
        let text = Text::from(lines).centered();

        frame.render_widget(text, area);
    }
}
