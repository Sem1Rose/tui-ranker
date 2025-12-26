use std::fmt::format;

use crate::helpers::{add_padding, popup};
use anyhow::Ok;
use ranker::Ranker;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{
        Modifier, Style, Stylize,
        palette::{material, tailwind},
    },
    text::{Line, Span, Text},
    widgets::{Block, Gauge, Padding},
};

#[derive(Default)]
pub enum Phase {
    #[default]
    Selecting,
    Done,
}

#[derive(Default)]
pub struct ProjectSelect {
    pub phase: Phase,

    project_list_alignment_bottom: bool,
    scroll_pos: usize,
    selected: usize,
}

impl ProjectSelect {
    pub fn render(&mut self, frame: &mut Frame, ranker: &Ranker<String>) -> anyhow::Result<()> {
        let popup_area = popup(
            frame,
            Constraint::Percentage(50),
            Constraint::Percentage(70),
            tailwind::BLUE.c950,
            "  Project Select  ",
            Style::new().fg(material::YELLOW.c800),
            Alignment::Center,
            Style::new().fg(tailwind::VIOLET.c950),
        );

        let [left, right] =
            Layout::horizontal([Constraint::Percentage(40), Constraint::Min(0)]).areas(popup_area);
        let right = add_padding(
            right,
            Padding {
                left: 0,
                right: 1,
                top: 4,
                bottom: 0,
            },
        );

        frame.render_widget(Block::new().bg(material::BLUE_GRAY.c900), right);

        let num_visible_projects = right.height as usize / 5;
        let partially_visible_project_height = right.height as usize - num_visible_projects * 5;
        let render_partially_visible_project = partially_visible_project_height > 0;

        let num_projects = num_visible_projects
            + if render_partially_visible_project {
                1
            } else {
                0
            };

        if ranker.get_num_projects() <= num_visible_projects {
            self.project_list_alignment_bottom = false;
        } else if self.selected - self.scroll_pos == 0 {
            self.project_list_alignment_bottom = false;
        } else if self.selected - self.scroll_pos == num_projects - 1 {
            self.project_list_alignment_bottom = true;
        }

        let mut remaining_area = right;
        for i in 0..num_projects {
            let [area, remaining] =
                if render_partially_visible_project && i == 0 && self.project_list_alignment_bottom
                {
                    Layout::vertical([
                        Constraint::Length(partially_visible_project_height as u16),
                        Constraint::Min(0),
                    ])
                } else if render_partially_visible_project
                    && i == num_projects - 1
                    && !self.project_list_alignment_bottom
                {
                    Layout::vertical([
                        Constraint::Length(partially_visible_project_height as u16),
                        Constraint::Min(0),
                    ])
                } else {
                    Layout::vertical([Constraint::Length(5), Constraint::Min(0)])
                }
                .areas(remaining_area);

            if self.scroll_pos + i < ranker.get_num_projects() {
                self.render_project_widget(i, frame, area, ranker);
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

        Ok(())
    }

    fn render_project_widget(
        &mut self,
        index: usize,
        frame: &mut Frame,
        area: Rect,
        ranker: &Ranker<String>,
    ) {
        let project = ranker
            .get_project_by_index(self.scroll_pos + index)
            .unwrap();
        let partially_visible = area.height < 5;

        let alternate = index & 1 == 1;
        let selected = self.selected == index + self.scroll_pos;
        // if partially_visible {
        //     if self.project_list_alignment_bottom {
        //         frame.render_widget(Block::new().bg(material::RED.c400), area);
        //     } else {
        //         frame.render_widget(Block::new().bg(material::GREEN.c400), area);
        //     }
        // } else {
        //     frame.render_widget(Block::new().bg(material::BLUE.c400), area);
        // }
        frame.render_widget(
            Block::new().bg(if selected {
                tailwind::TEAL.c500
            } else if !alternate {
                tailwind::GRAY.c700
            } else {
                tailwind::SLATE.c700
            }),
            area,
        );

        let areas = Layout::vertical(vec![Constraint::Length(1); area.height as usize]).split(area);

        let range = if partially_visible {
            if self.project_list_alignment_bottom {
                (5 - area.height)..5
            } else {
                0..area.height
            }
        } else {
            0..5
        };

        for i in range {
            if i == 0 {
                frame.render_widget(
                    Line::from("▔".repeat(area.width as usize)).style(Style::new().fg(
                        if selected {
                            tailwind::EMERALD.c700
                        } else if !alternate {
                            tailwind::GRAY.c600
                        } else {
                            tailwind::SLATE.c600
                        },
                    )),
                    areas[i as usize],
                );
            } else if i == 1 {
                frame.render_widget(
                    Text::from(project.name.as_str())
                        .alignment(Alignment::Left)
                        .style(
                            Style::new()
                                .fg(if selected {
                                    tailwind::SLATE.c900
                                } else {
                                    material::ORANGE.c700
                                })
                                .add_modifier(if selected {
                                    Modifier::BOLD
                                } else {
                                    Modifier::empty()
                                }),
                        ),
                    add_padding(areas[i as usize], Padding::left(2)),
                );
            } else if i == 3 {
                frame.render_widget(
                    Gauge::default()
                        .use_unicode(true)
                        .ratio(
                            project.num_rated_items as f64
                                / if project.total_ratings > 0 {
                                    project.total_ratings as f64
                                } else {
                                    1.0
                                },
                        )
                        .label(
                            Span::from(format!(
                                "{}/{}",
                                project.num_rated_items, project.total_ratings
                            ))
                            .style(Style::new().fg(tailwind::SLATE.c700)),
                        )
                        .gauge_style(Style::new().bg(tailwind::SLATE.c950).fg(if selected {
                            tailwind::INDIGO.c700
                        } else {
                            tailwind::INDIGO.c900
                        })),
                    add_padding(
                        areas[i as usize],
                        Padding {
                            left: 2,
                            right: 2,
                            top: 0,
                            bottom: 0,
                        },
                    ),
                );
            } else if i == 4 {
                frame.render_widget(
                    Line::from("▁".repeat(area.width as usize)).style(Style::new().fg(
                        if selected {
                            tailwind::EMERALD.c700
                        } else if !alternate {
                            tailwind::GRAY.c600
                        } else {
                            tailwind::SLATE.c600
                        },
                    )),
                    areas[i as usize],
                );
            }
        }
    }
}
