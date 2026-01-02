use crate::KeyEventHandler;
use crate::helpers::{add_padding, dynamic_popup};
use crate::image_backend::RatatuiImage;
use crate::key_event_handler::Data;
use crate::popups::Popups;
use ratatui::symbols::border;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{
        Modifier, Style, Stylize,
        palette::{material, tailwind},
    },
    symbols::{block, scrollbar::Set},
    text::Span,
    widgets::{Block, Padding, Scrollbar, ScrollbarState},
};
use std::ops::Add;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

#[derive(Default)]
pub struct Results {
    pub selected_item: usize,
    pub image_renderer: RatatuiImage,
    pub drawing_images: bool,

    num_visible_items: usize,
    scroll_pos: usize,
    items: Vec<(String, f32)>,
    finished: bool,
}

impl Results {
    pub fn get_state(&self) -> (Option<usize>, Option<usize>) {
        (None, None)
        // (None, Some(self.item))
    }
    pub fn finished(mut self, finished: bool) -> Self {
        self.finished = finished;
        self
    }

    pub fn set_items(&mut self, items: &[(String, f32)]) {
        self.items = items.iter().map(|x| x.clone()).collect();
    }

    pub fn render(
        &mut self,
        frame: &mut Frame,
        key_event_handler: &mut KeyEventHandler,
    ) -> anyhow::Result<()> {
        let finished = self.finished;
        key_event_handler.clear();
        key_event_handler.bind_esc((None, None), move |app, _| {
            // app.drawer.close_popups();
            if finished {
                app.drawer.open_project_select_popup();
            } else {
                app.drawer.close_popups();

                // app.ranker
                //     .try_select_project_by_name(
                //         &app.ranker.get_selected_project().unwrap().name.clone(),
                //     )
                //     .unwrap();

                // app.drawer.open_main_screen(&mut app.ranker);
            }
        });
        key_event_handler.bind_key((None, None), 'q', move |app, _| {
            app.drawer.close_popups();
            if finished {
                app.quit = true;
            }
        });
        key_event_handler.bind_vertical((None, None), |app, data| {
            if let Some(Popups::Results(results_popup)) = app.drawer.active_popup.as_mut() {
                match data {
                    Data::Direction(true) => {
                        results_popup.selected_item = results_popup
                            .selected_item
                            .add(1)
                            .min(results_popup.items.len().saturating_sub(1));
                        if results_popup.selected_item - results_popup.scroll_pos
                            >= results_popup.num_visible_items
                        {
                            results_popup.scroll_pos += 1;
                        }
                    }
                    Data::Direction(false) => {
                        results_popup.selected_item = results_popup.selected_item.saturating_sub(1);
                        if results_popup.selected_item < results_popup.scroll_pos {
                            results_popup.scroll_pos -= 1;
                        }
                    }
                    _ => {}
                }
            }
        });
        key_event_handler.bind_enter((None, None), |app, _| {
            if let Some(Popups::Results(results_popup)) = app.drawer.active_popup.as_mut() {
                Command::new("xdg-open")
                    .arg(&results_popup.items[results_popup.selected_item].0)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                    .unwrap();
            }
        });

        let popup_area = dynamic_popup(
            frame,
            Some(35),
            2.5,
            tailwind::BLUE.c950,
            "  Results  ",
            Style::new().fg(material::YELLOW.c800),
            Alignment::Center,
            Style::new().fg(tailwind::VIOLET.c950),
        );

        self.num_visible_items = 4;

        if self.selected_item < self.scroll_pos {
            self.selected_item = self
                .selected_item
                .add(1)
                .min(self.items.len().saturating_sub(1));
        } else if self.selected_item >= self.items.len() {
            self.selected_item = self.items.len().saturating_sub(1);
            self.scroll_pos = self
                .selected_item
                .saturating_sub(self.num_visible_items + 1);
        } else if self.selected_item - self.scroll_pos >= self.num_visible_items {
            self.scroll_pos = self
                .selected_item
                .saturating_sub(self.num_visible_items + 1);
        }

        self.drawing_images = false;
        let mut remaining_area = add_padding(popup_area, Padding::right(1));
        for i in 0..self.num_visible_items {
            let [area, remaining] = Layout::vertical([
                Constraint::Length(popup_area.height / self.num_visible_items as u16),
                Constraint::Min(0),
            ])
            .areas(remaining_area);

            if self.scroll_pos + i < self.items.len() {
                self.render_item(i, frame, area);
            } else {
                frame.render_widget(
                    Block::new().bg(if i & 1 == 0 {
                        tailwind::SLATE.c950
                    } else {
                        tailwind::BLACK
                    }),
                    area,
                );
            }

            remaining_area = remaining;
        }

        let scrollbar = Scrollbar::new(ratatui::widgets::ScrollbarOrientation::VerticalRight)
            .symbols(Set {
                track: block::FULL,
                thumb: block::FULL, //"🮋",
                begin: "▲",
                end: "▼",
            })
            .begin_style(
                Style::new()
                    .bg(material::LIGHT_BLUE.c700)
                    .fg(tailwind::INDIGO.c900),
            )
            .end_style(
                Style::new()
                    .bg(material::LIGHT_BLUE.c700)
                    .fg(tailwind::INDIGO.c900),
            )
            .track_style(Style::new().fg(tailwind::SLATE.c900))
            .thumb_style(
                Style::new()
                    .fg(material::BLUE.c800)
                    .bg(tailwind::SLATE.c900),
            );
        let mut scrollbar_state =
            ScrollbarState::new(self.items.len().saturating_sub(self.num_visible_items - 1))
                .position(self.scroll_pos);

        frame.render_stateful_widget(
            scrollbar,
            Layout::horizontal([Constraint::Min(0), Constraint::Length(3)]).split(popup_area)[1],
            &mut scrollbar_state,
        );

        Ok(())
    }

    fn render_item(&mut self, index: usize, frame: &mut Frame, area: Rect) {
        let item = &self.items[self.scroll_pos + index];

        let alternate = (self.scroll_pos + index) & 1 == 1;
        let selected = self.selected_item == index + self.scroll_pos;

        frame.render_widget(
            Block::new().bg(if selected {
                tailwind::TEAL.c600
            } else if !alternate {
                tailwind::GRAY.c600
            } else {
                tailwind::SLATE.c700
            }),
            area,
        );

        let image_width = (area.height as f64 * 26.0 / 9.0) as u16;
        let [rank, image, details] = Layout::horizontal(vec![
            Constraint::Length(5),
            Constraint::Length(image_width),
            Constraint::Min(0),
        ])
        .areas(area);

        let rank_area = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .split(rank)[1];

        frame.render_widget(
            Span::from(format!("{}", self.scroll_pos + index + 1))
                .into_right_aligned_line()
                .style(Style::new().fg(if self.scroll_pos + index == 0 {
                    if selected {
                        material::YELLOW.c300
                    } else {
                        material::YELLOW.c600
                    }
                } else if self.scroll_pos + index == 1 {
                    if selected {
                        material::GREEN.c400
                    } else {
                        material::GREEN.c500
                    }
                } else if self.scroll_pos + index == 2 {
                    if selected {
                        material::DEEP_ORANGE.c200
                    } else {
                        material::DEEP_ORANGE.c300
                    }
                } else {
                    if selected {
                        material::GRAY.c300
                    } else {
                        material::GRAY.c500
                    }
                }))
                .add_modifier(if selected {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                }),
            add_padding(
                rank_area,
                Padding {
                    left: 1,
                    right: 1,
                    top: 0,
                    bottom: 0,
                },
            ),
        );

        let image_block = Block::bordered()
            .border_set(border::PROPORTIONAL_WIDE)
            .border_style(Style::new().fg(if self.scroll_pos + index == 0 {
                if selected {
                    material::YELLOW.c300
                } else {
                    material::YELLOW.c600
                }
            } else if self.scroll_pos + index == 1 {
                if selected {
                    material::GREEN.c300
                } else {
                    material::GREEN.c500
                }
            } else if self.scroll_pos + index == 2 {
                if selected {
                    material::DEEP_ORANGE.c200
                } else {
                    material::DEEP_ORANGE.c300
                }
            } else {
                if selected {
                    material::GRAY.c300
                } else {
                    material::GRAY.c500
                }
            }));
        let image_area = image_block.inner(image);
        frame.render_widget(&image_block, image);
        frame.render_widget(Block::new().bg(tailwind::SLATE.c800), image_area);
        self.drawing_images |= !self.image_renderer.draw_image(&item.0, image_area, frame);

        let [_, name, _, score, _] = Layout::vertical(vec![
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .areas(add_padding(details, Padding::left(2)));
        frame.render_widget(
            Span::from(
                PathBuf::from(&item.0)
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string(),
            )
            .into_left_aligned_line()
            .style(Style::new().fg(if selected {
                material::GRAY.c300
            } else {
                material::GRAY.c500
            }))
            .add_modifier(if selected {
                Modifier::BOLD
            } else {
                Modifier::empty()
            }),
            name,
        );
        frame.render_widget(
            Span::from(format!("{:.1}", item.1))
                .into_left_aligned_line()
                .style(Style::new().fg(if selected {
                    material::GRAY.c300
                } else {
                    material::GRAY.c500
                }))
                .add_modifier(if selected {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                }),
            score,
        );
    }
}
