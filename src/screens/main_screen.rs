use crate::KeyEventHandler;
use crate::helpers::dynamic_area;
use crate::image_backend::RatatuiImage;
use crate::popups::{Popups, ProjectSelect};
use crate::screens::Screens;
use ranker::Ranker;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Offset, Rect};
use ratatui::style::Style;
use ratatui::style::palette::tailwind;
use ratatui::style::{Stylize, palette::material};
use ratatui::symbols::border;
use ratatui::text::Span;
use ratatui::widgets::{Block, Gauge};

#[derive(Default)]
pub struct MainScreen {
    pub items: Option<(String, String)>,
    pub refresh: u8,
    pub drawing_images: bool,
}
impl MainScreen {
    pub fn render(
        &mut self,
        frame: &mut Frame,
        ranker: &Ranker<String>,
        key_event_handler: &mut KeyEventHandler,
        image_backend: &mut RatatuiImage,
    ) -> anyhow::Result<()> {
        if self.items.is_some() {
            key_event_handler.bind_horizontal((None, None), |app, data| {
                if let Some(Screens::MainScreen) = app.drawer.current_screen {
                    let main_screen = &mut app.drawer.main_screen;

                    match data {
                        crate::key_event_handler::Data::Direction(dir) => {
                            app.ranker.log_result(!dir).unwrap();
                        }
                        _ => {}
                    }

                    let result = app.ranker.get_next();
                    if let Ok(x) = result {
                        main_screen.items = x;
                    }
                    if app.ranker.window_rated_items == 0 {
                        app.drawer
                            .image_backend
                            .preload_images(&app.ranker.get_window_items());
                    }
                }
            });
        }
        key_event_handler.bind_esc((None, None), |app, _| {
            if app.drawer.active_popup.is_none() {
                app.drawer.active_popup = Some(Popups::ProjectSelect(ProjectSelect::default()));

                app.key_event_handler.clear();
            }
        });

        let project = ranker.get_selected_project().unwrap();

        let area = dynamic_area(
            Some(38),
            3.5,
            ratatui::layout::Flex::Center,
            ratatui::layout::Flex::End,
            frame.area(),
        );
        frame.render_widget(Block::new().bg(tailwind::INDIGO.c950), area);

        let images_width = (area.width - 3) / 2;
        let images_height = (images_width as f64 * 10.0 / 32.0) as u16;

        let [images, _, rest] = Layout::vertical([
            Constraint::Length(images_height),
            Constraint::Length(1),
            Constraint::Min(1),
        ])
        .areas(area);
        let [controls, _, gauge] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Min(1),
            Constraint::Length(5),
        ])
        .areas(rest);

        let a_area = Rect {
            x: images.x + 1,
            y: images.y,
            width: images_width,
            height: images.height,
        };
        let b_area = a_area.clone().offset(Offset {
            x: images_width as i32 + if (images.width - 3) & 1 == 1 { 2 } else { 1 },
            y: 0,
        });

        let block = Block::bordered()
            .border_set(border::PROPORTIONAL_WIDE)
            .border_style(Style::new().fg(material::INDIGO.c700));

        frame.render_widget(&block, a_area);
        frame.render_widget(Block::new().bg(material::INDIGO.c700), block.inner(a_area));
        frame.render_widget(&block, b_area);
        frame.render_widget(Block::new().bg(material::INDIGO.c700), block.inner(b_area));

        if self.refresh < 1 {
            if let Some((item_a, item_b)) = &self.items {
                self.drawing_images =
                    !(image_backend.draw_image(item_a, block.inner(a_area), frame)
                        & image_backend.draw_image(item_b, block.inner(b_area), frame));
            }
        }
        self.refresh = self.refresh.saturating_sub(1);

        let gauge_ratio = project.num_rated_items as f64
            / if project.total_ratings > 0 {
                project.total_ratings as f64
            } else {
                1.0
            };
        frame.render_widget(
            Gauge::default()
                .block(
                    Block::bordered()
                        .border_set(border::PROPORTIONAL_WIDE)
                        .border_style(Style::new().fg(material::INDIGO.c900)),
                )
                .use_unicode(true)
                .ratio(gauge_ratio.min(1.0))
                .label(
                    Span::from(format!(
                        "{}/{}",
                        project.num_rated_items, project.total_ratings
                    ))
                    .style(
                        Style::new()
                            .fg(if gauge_ratio < 0.1 {
                                material::DEEP_ORANGE.c600
                            } else if gauge_ratio < 0.3 {
                                material::YELLOW.c600
                            } else if gauge_ratio < 0.5 {
                                material::LIME.c700
                            } else if gauge_ratio < 0.7 {
                                material::GREEN.c500
                            } else if gauge_ratio < 0.8 {
                                material::TEAL.c500
                            } else {
                                material::BLUE.c600
                            })
                            .bg(tailwind::SLATE.c950),
                    ),
                )
                .gauge_style(
                    Style::new()
                        .bg(tailwind::SLATE.c950)
                        .fg(if gauge_ratio < 0.1 {
                            material::DEEP_ORANGE.c800
                        } else if gauge_ratio < 0.3 {
                            material::YELLOW.c800
                        } else if gauge_ratio < 0.5 {
                            material::LIME.c900
                        } else if gauge_ratio < 0.7 {
                            material::GREEN.c800
                        } else if gauge_ratio < 0.8 {
                            material::TEAL.c800
                        } else {
                            material::BLUE.c800
                        }),
                ),
            gauge,
        );

        Ok(())
    }
}
