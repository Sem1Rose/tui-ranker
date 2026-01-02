use crate::{KeyEventHandler, popups::*, screens::*};
use ranker::Ranker;
use ratatui::{
    Frame,
    layout::Constraint,
    style::{Stylize, palette::tailwind::*},
    text::{Line, Text},
    widgets::Block,
};

pub struct Drawer {
    pub current_screen: Option<Screens>,
    pub active_popup: Option<Popups>,

    show_term_size_warning: bool,

    refresh_immediate: u8,
}

const MINTERMSIZE: [u32; 2] = [100, 30];
impl Drawer {
    pub fn new() -> Self {
        Drawer {
            current_screen: None,
            active_popup: Some(Popups::default()),
            show_term_size_warning: false,
            refresh_immediate: 0,
        }
    }

    pub fn render_app(
        &mut self,
        frame: &mut Frame,
        ranker: &mut Ranker<String>,
        key_event_handler: &mut KeyEventHandler,
    ) -> anyhow::Result<()> {
        self.refresh_immediate = self.refresh_immediate.saturating_sub(1);

        self.check_term_size(frame);
        self.update_image_renderers();

        self.draw_current_screen(frame, ranker, key_event_handler)?;

        self.check_popups(ranker)?;
        if !self.show_term_size_warning && self.active_popup.is_some() {
            self.draw_popup(frame, ranker, key_event_handler)?;
        }

        Ok(())
    }

    fn update_image_renderers(&mut self) {
        if let Some(Screens::MainScreen(main_screen)) = self.current_screen.as_mut() {
            main_screen.image_renderer.update();
        }
        if let Some(Popups::Results(results)) = self.active_popup.as_mut() {
            results.image_renderer.update();
        }
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
        } else if let Some(current_screen) = self.current_screen.as_mut() {
            match current_screen {
                Screens::MainScreen(main_screen) => {
                    main_screen.render(frame, ranker, key_event_handler)?;
                }
            }
        }

        Ok(())
    }

    fn check_popups(&mut self, ranker: &mut Ranker<String>) -> anyhow::Result<()> {
        if let Some(popup) = self.active_popup.as_mut() {
            match popup {
                Popups::ProjectSelect(project_select_popup) => match project_select_popup.phase {
                    ProjectSelectPhase::Done => {
                        self.open_main_screen(ranker);
                    }
                    _ => {}
                },
                Popups::Results(_) => (),
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
                Popups::Results(results_popup) => {
                    results_popup.render(frame, key_event_handler)?;
                }
            }
        } else {
        }

        Ok(())
    }

    pub fn open_results_popup(&mut self, ranker: &mut Ranker<String>, finished: bool) {
        let mut results_popup = Results::default().finished(finished);
        results_popup
            .image_renderer
            .change_root(&ranker.get_selected_project().unwrap().dir);
        results_popup.set_items(&ranker.get_item_scores());

        self.active_popup = Some(Popups::Results(results_popup));
    }
    pub fn open_project_select_popup(&mut self) {
        self.active_popup = Some(Popups::ProjectSelect(ProjectSelect::default()))
    }

    pub fn close_popups(&mut self) {
        self.active_popup = None;

        self.refresh_immediate += 2;
        if let Some(Screens::MainScreen(main_screen)) = self.current_screen.as_mut() {
            main_screen.redraw_images = 1;
        }
    }

    pub fn open_main_screen(&mut self, ranker: &mut Ranker<String>) {
        self.close_popups();

        let mut main_screen = MainScreen::default();
        main_screen
            .image_renderer
            .change_root(&ranker.get_selected_project().unwrap().dir);
        main_screen
            .image_renderer
            .filter_cached_images(ranker.get_window_items().as_slice());
        main_screen
            .image_renderer
            .preload_images(&ranker.get_window_items());

        let result = ranker.get_next();
        if let Ok(x) = result {
            main_screen.items = x;
        }

        self.current_screen = Some(Screens::MainScreen(main_screen));
    }

    pub fn check_refresh_immediate(&mut self) -> bool {
        self.refresh_immediate > 0
    }
    pub fn check_refresh_delayed(&mut self) -> bool {
        if let Some(Popups::Results(results_popup)) = self.active_popup.as_ref() {
            return results_popup.drawing_images;
        } else if let Some(Screens::MainScreen(main_screen)) = self.current_screen.as_ref() {
            return main_screen.drawing_images;
        }

        false
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
