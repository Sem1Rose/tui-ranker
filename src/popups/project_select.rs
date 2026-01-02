use crate::KeyEventHandler;
use crate::helpers::{add_padding, dynamic_popup};
use crate::key_event_handler::Data;
use crate::popups::Popups;
use ranker::Ranker;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{
        Modifier, Style, Stylize,
        palette::{material, tailwind},
    },
    symbols::{block, scrollbar::Set},
    text::{Line, Span, Text},
    widgets::{Block, Gauge, Padding, Scrollbar, ScrollbarState},
};
use std::ops::Add;
use tui_textarea::TextArea;

#[derive(Default)]
pub enum Phase {
    #[default]
    Selecting,
    Done,
}

#[derive(Default)]
pub struct ProjectSelect {
    pub phase: Phase,
    pub tab: usize,
    pub item: usize,
    pub project_list_selected_item: usize,

    project_list_visible_items: usize,
    project_list_alignment_bottom: bool,
    project_list_scroll_pos: usize,
    project_list_show_delete_confirmation: bool,

    add_project_input_disable: bool,
    add_project_name_input: TextArea<'static>,
    add_project_path_input: TextArea<'static>,
    edit_project: bool,
}

impl ProjectSelect {
    pub fn get_state(&self) -> (Option<usize>, Option<usize>) {
        (Some(self.tab), Some(self.item))
    }

    pub fn render(
        &mut self,
        frame: &mut Frame,
        ranker: &Ranker<String>,
        key_event_handler: &mut KeyEventHandler,
        exitable: bool,
    ) -> anyhow::Result<()> {
        key_event_handler.clear();

        key_event_handler.bind_tab((Some(0), None), |app, data| {
            if let Some(Popups::ProjectSelect(project_select)) = app.drawer.active_popup.as_mut() {
                project_select.item = 0;
                match data {
                    Data::Direction(_) => {
                        project_select.tab = 1;
                    }
                    _ => {}
                }
            }
        });

        let popup_area = dynamic_popup(
            frame,
            Some(25),
            3.5,
            tailwind::BLUE.c950,
            "  Project Select  ",
            Style::new().fg(material::YELLOW.c800),
            Alignment::Center,
            Style::new().fg(tailwind::VIOLET.c950),
        );

        let [left, right] =
            Layout::horizontal([Constraint::Percentage(40), Constraint::Min(0)]).areas(popup_area);

        self.render_projects_list(frame, right, ranker, key_event_handler, exitable);

        self.render_add_edit_project(frame, left, ranker, key_event_handler, exitable);

        Ok(())
    }

    fn render_projects_list(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        ranker: &Ranker<String>,
        key_event_handler: &mut KeyEventHandler,
        exitable: bool,
    ) {
        let tab_selected = self.tab == 0;

        let projects_list_area = add_padding(
            area,
            Padding {
                left: 0,
                right: 1,
                top: 1,
                bottom: 0,
            },
        );

        frame.render_widget(
            Block::new().bg(material::BLUE_GRAY.c900),
            projects_list_area,
        );

        let num_visible_projects = projects_list_area.height as usize / 5;
        let partially_visible_project_height =
            projects_list_area.height as usize - num_visible_projects * 5;
        let render_partially_visible_project = partially_visible_project_height > 0;

        self.project_list_visible_items = num_visible_projects
            + if render_partially_visible_project {
                1
            } else {
                0
            };

        key_event_handler.bind_vertical((Some(0), None), |app, data| {
            if let Some(Popups::ProjectSelect(project_select)) = app.drawer.active_popup.as_mut() {
                project_select.project_list_show_delete_confirmation = false;
                project_select.item = 0;

                match data {
                    Data::Direction(true) => {
                        project_select.project_list_selected_item = project_select
                            .project_list_selected_item
                            .add(1)
                            .min(app.ranker.get_num_projects().saturating_sub(1));
                        if project_select.project_list_selected_item
                            - project_select.project_list_scroll_pos
                            >= project_select.project_list_visible_items
                        {
                            project_select.project_list_scroll_pos += 1;
                        }
                    }
                    Data::Direction(false) => {
                        project_select.project_list_selected_item =
                            project_select.project_list_selected_item.saturating_sub(1);
                        if project_select.project_list_selected_item
                            < project_select.project_list_scroll_pos
                        {
                            project_select.project_list_scroll_pos -= 1;
                        }
                    }
                    _ => {}
                }
            }
        });
        key_event_handler.bind_enter((Some(0), Some(0)), |app, _| {
            app.select_project().unwrap();

            if let Some(Popups::ProjectSelect(project_select)) = app.drawer.active_popup.as_mut() {
                project_select.phase = Phase::Done;
            }
        });
        if exitable {
            key_event_handler.bind_esc((Some(0), Some(0)), |app, _| {
                app.select_project().unwrap();

                if let Some(Popups::ProjectSelect(project_select)) =
                    app.drawer.active_popup.as_mut()
                {
                    project_select.phase = Phase::Done;
                }
            });
            key_event_handler.bind_key((Some(0), Some(0)), 'q', |app, _| {
                app.select_project().unwrap();

                if let Some(Popups::ProjectSelect(project_select)) =
                    app.drawer.active_popup.as_mut()
                {
                    project_select.phase = Phase::Done;
                }
            });
        }
        key_event_handler.bind_esc((Some(0), Some(1)), |app, _| {
            if let Some(Popups::ProjectSelect(project_select)) = app.drawer.active_popup.as_mut() {
                project_select.item = 0;
            }
        });
        key_event_handler.bind_esc((Some(0), Some(2)), |app, _| {
            if let Some(Popups::ProjectSelect(project_select)) = app.drawer.active_popup.as_mut() {
                project_select.project_list_show_delete_confirmation = false;
                project_select.item = 0;
            }
        });
        key_event_handler.bind_key((Some(0), None), 'a', |app, _| {
            if let Some(Popups::ProjectSelect(project_select)) = app.drawer.active_popup.as_mut() {
                project_select.item = 0;
                project_select.tab = 1;
            }
        });
        key_event_handler.bind_key((Some(0), None), 'e', |app, _| {
            if let Some(Popups::ProjectSelect(project_select)) = app.drawer.active_popup.as_mut() {
                let name = &app
                    .ranker
                    .get_project_by_index(project_select.project_list_selected_item)
                    .unwrap()
                    .name;
                project_select.add_project_name_input = TextArea::from([name]);
                project_select.add_project_path_input =
                    TextArea::from([app.project_table.get(name).unwrap()]);

                project_select.edit_project = true;
                project_select.add_project_input_disable = false;
                project_select.tab = 1;
                project_select.item = 0;
            }
        });
        key_event_handler.bind_key((Some(0), None), 'd', |app, _| {
            if let Some(Popups::ProjectSelect(project_select)) = app.drawer.active_popup.as_mut() {
                project_select.project_list_show_delete_confirmation = true;
                project_select.item = 2;
            }
        });

        if tab_selected && ranker.get_num_projects() == 0 {
            self.tab = 1;
            self.item = 0;
        }

        if self.project_list_selected_item < self.project_list_scroll_pos {
            self.project_list_selected_item = self
                .project_list_selected_item
                .add(1)
                .min(ranker.get_num_projects().saturating_sub(1));
        } else if self.project_list_selected_item >= ranker.get_num_projects() {
            self.project_list_selected_item = ranker.get_num_projects().saturating_sub(1);
            self.project_list_scroll_pos = self
                .project_list_selected_item
                .saturating_sub(self.project_list_visible_items + 1);
        } else if self.project_list_selected_item - self.project_list_scroll_pos
            >= self.project_list_visible_items
        {
            self.project_list_scroll_pos = self
                .project_list_selected_item
                .saturating_sub(self.project_list_visible_items + 1);
        }

        if ranker.get_num_projects() <= num_visible_projects {
            self.project_list_alignment_bottom = false;
        } else if self.project_list_selected_item - self.project_list_scroll_pos == 0 {
            self.project_list_alignment_bottom = false;
        } else if self.project_list_selected_item - self.project_list_scroll_pos
            == self.project_list_visible_items - 1
        {
            self.project_list_alignment_bottom = true;
        }

        let mut remaining_area = projects_list_area;
        for i in 0..self.project_list_visible_items {
            let [area, remaining] =
                if render_partially_visible_project && i == 0 && self.project_list_alignment_bottom
                {
                    Layout::vertical([
                        Constraint::Length(partially_visible_project_height as u16),
                        Constraint::Min(0),
                    ])
                } else if render_partially_visible_project
                    && i == self.project_list_visible_items - 1
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

            if self.project_list_scroll_pos + i < ranker.get_num_projects() {
                self.render_project_widget(i, tab_selected, frame, area, ranker, key_event_handler);
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
        let mut scrollbar_state = ScrollbarState::new(
            ranker
                .get_num_projects()
                .saturating_sub(self.project_list_visible_items - 1),
        )
        .position(self.project_list_scroll_pos);

        frame.render_stateful_widget(
            scrollbar,
            add_padding(
                Layout::horizontal([Constraint::Min(0), Constraint::Length(3)]).split(area)[1],
                Padding::top(1),
            ),
            &mut scrollbar_state,
        );

        // Ok(())
    }

    fn render_project_widget(
        &mut self,
        index: usize,
        tab_selected: bool,
        frame: &mut Frame,
        area: Rect,
        ranker: &Ranker<String>,
        key_event_handler: &mut KeyEventHandler,
    ) {
        let project = ranker
            .get_project_by_index(self.project_list_scroll_pos + index)
            .unwrap();
        let partially_visible = area.height < 5;

        let alternate = index & 1 == 1;
        let selected =
            tab_selected && self.project_list_selected_item == index + self.project_list_scroll_pos;
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
                tailwind::TEAL.c600
            } else if !alternate {
                tailwind::GRAY.c600
            } else {
                tailwind::SLATE.c700
            }),
            area,
        );

        key_event_handler.bind_horizontal((Some(0), None), |app, data| {
            if let Some(Popups::ProjectSelect(project_select)) = app.drawer.active_popup.as_mut() {
                project_select.project_list_show_delete_confirmation = false;
                match data {
                    Data::Direction(true) => {
                        project_select.item = project_select.item.add(1).min(2);
                    }
                    Data::Direction(false) => {
                        project_select.item = project_select.item.checked_sub(1).unwrap_or(0);
                    }
                    _ => {}
                }
            }
        });
        key_event_handler.bind_enter((Some(0), Some(1)), |app, _| {
            if let Some(Popups::ProjectSelect(project_select)) = app.drawer.active_popup.as_mut() {
                let name = &app
                    .ranker
                    .get_project_by_index(project_select.project_list_selected_item)
                    .unwrap()
                    .name;
                project_select.add_project_name_input = TextArea::from([name]);
                project_select.add_project_path_input =
                    TextArea::from([app.project_table.get(name).unwrap()]);

                project_select.edit_project = true;
                project_select.add_project_input_disable = false;
                project_select.tab = 1;
                project_select.item = 0;
            }
        });
        key_event_handler.bind_enter((Some(0), Some(2)), |app, _| {
            let mut delete = false;
            if let Some(Popups::ProjectSelect(project_select)) = app.drawer.active_popup.as_mut() {
                if project_select.project_list_show_delete_confirmation {
                    delete = true;

                    project_select.project_list_show_delete_confirmation = false;
                    project_select.item = 0;
                } else {
                    project_select.project_list_show_delete_confirmation = true;
                }
            }
            if delete {
                app.delete_project().unwrap();
            }
        });

        let areas = Layout::vertical(vec![Constraint::Length(1); area.height as usize]).split(area);

        for i in 0..area.height {
            let index = if partially_visible {
                if self.project_list_alignment_bottom {
                    i + (5 - area.height)
                } else {
                    i
                }
            } else {
                i
            };
            if index == 0 {
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
            } else if index == 1 {
                let [text, delete] = Layout::horizontal([
                    Constraint::Min(0),
                    Constraint::Length(if selected && self.project_list_show_delete_confirmation {
                        13
                    } else {
                        7
                    }),
                ])
                .areas(add_padding(
                    areas[i as usize],
                    Padding {
                        left: 2,
                        right: 2,
                        top: 0,
                        bottom: 0,
                    },
                ));
                frame.render_widget(
                    Text::from(project.name.as_str())
                        .alignment(Alignment::Left)
                        .style(
                            Style::new()
                                .fg(if selected {
                                    material::CYAN.c100
                                } else {
                                    material::ORANGE.c400
                                })
                                .add_modifier(if selected {
                                    Modifier::BOLD
                                } else {
                                    Modifier::empty()
                                }),
                        ),
                    text,
                );

                if selected {
                    if self.project_list_show_delete_confirmation {
                        frame.render_widget(
                            Line::from(vec![
                                Span::from(" E ").style(
                                    Style::new()
                                        .fg(material::BLUE.c700)
                                        .bg(tailwind::SLATE.c900),
                                ),
                                Span::from(" "),
                                Span::from(" Confirm ").style(
                                    Style::new()
                                        .fg(tailwind::SLATE.c400)
                                        .bg(material::RED.c800)
                                        .bold(),
                                ),
                            ]),
                            delete,
                        );
                    } else {
                        frame.render_widget(
                            Line::from(vec![
                                Span::from(" E ").style(
                                    Style::new()
                                        .fg(if self.item == 1 {
                                            tailwind::SLATE.c200
                                        } else {
                                            material::BLUE.c700
                                        })
                                        .bg(if self.item == 1 {
                                            material::BLUE.c700
                                        } else {
                                            tailwind::SLATE.c900
                                        })
                                        .add_modifier(if self.item == 1 {
                                            Modifier::BOLD
                                        } else {
                                            Modifier::empty()
                                        }),
                                ),
                                Span::from(" "),
                                Span::from(" D ").style(
                                    Style::new()
                                        .fg(if self.item == 2 {
                                            tailwind::SLATE.c400
                                        } else {
                                            material::RED.c900
                                        })
                                        .bg(if self.item == 2 {
                                            material::RED.c900
                                        } else {
                                            tailwind::SLATE.c900
                                        })
                                        .add_modifier(if self.item == 2 {
                                            Modifier::BOLD
                                        } else {
                                            Modifier::empty()
                                        }),
                                ),
                            ]),
                            delete,
                        );
                    }
                }
            } else if index == 3 {
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
                            .style(Style::new().fg(if selected {
                                tailwind::SLATE.c300
                            } else {
                                tailwind::SLATE.c500
                            })),
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
            } else if index == 4 {
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

    fn render_add_edit_project(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        ranker: &Ranker<String>,
        key_event_handler: &mut KeyEventHandler,
        exitable: bool,
    ) {
        let tab_selected = self.tab == 1;
        let name_valid = self.validate_name(ranker);
        let path_valid = self.validate_path();
        let valid = name_valid && path_valid;
        let num_projects = ranker.get_num_projects();

        key_event_handler.bind_vertical((Some(1), None), |app, data| {
            if let Some(Popups::ProjectSelect(project_select)) = app.drawer.active_popup.as_mut() {
                if project_select.add_project_input_disable {
                    project_select.add_project_input_disable = false;
                } else {
                    match data {
                        Data::Direction(true) => {
                            if !project_select.edit_project || project_select.item < 2 {
                                project_select.item = project_select.item.add(1).min(2);
                            }
                        }
                        Data::Direction(false) => {
                            if project_select.edit_project && project_select.item >= 2 {
                                project_select.item = 1;
                            } else {
                                project_select.item =
                                    project_select.item.checked_sub(1).unwrap_or(0);
                            }
                        }
                        _ => {}
                    }
                }
            }
        });

        if self.edit_project {
            if self.add_project_input_disable {
                key_event_handler.bind_horizontal((Some(1), Some(2)), |app, _| {
                    if let Some(Popups::ProjectSelect(project_select)) =
                        app.drawer.active_popup.as_mut()
                    {
                        project_select.add_project_input_disable = false;
                    }
                });
                key_event_handler.bind_horizontal((Some(1), Some(3)), |app, _| {
                    if let Some(Popups::ProjectSelect(project_select)) =
                        app.drawer.active_popup.as_mut()
                    {
                        project_select.add_project_input_disable = false;
                    }
                });
            } else {
                key_event_handler.bind_horizontal((Some(1), Some(2)), |app, data| {
                    if let Some(Popups::ProjectSelect(project_select)) =
                        app.drawer.active_popup.as_mut()
                    {
                        match data {
                            Data::Direction(true) => {
                                project_select.item += 1;
                            }
                            _ => {}
                        }
                    }
                });
                key_event_handler.bind_horizontal((Some(1), Some(3)), |app, data| {
                    if let Some(Popups::ProjectSelect(project_select)) =
                        app.drawer.active_popup.as_mut()
                    {
                        match data {
                            Data::Direction(false) => {
                                project_select.item -= 1;
                            }
                            _ => {}
                        }
                    }
                });
            }
        }

        if self.add_project_input_disable {
            key_event_handler.bind_tab((Some(1), None), |app, data| {
                if let Some(Popups::ProjectSelect(project_select)) =
                    app.drawer.active_popup.as_mut()
                {
                    project_select.item = 0;
                    project_select.add_project_input_disable = false;
                    if project_select.edit_project {
                        project_select.edit_project = false;

                        project_select.add_project_name_input = TextArea::default();
                        project_select.add_project_path_input = TextArea::default();
                    }

                    match data {
                        Data::Direction(_) => {
                            project_select.tab = 0;
                        }
                        _ => {}
                    }
                }
            });
        } else {
            if self.edit_project {
                key_event_handler.bind_tab((Some(1), Some(3)), |app, data| {
                    if let Some(Popups::ProjectSelect(project_select)) =
                        app.drawer.active_popup.as_mut()
                    {
                        match data {
                            // Data::Direction(true) => {
                            //     project_select.add_project_edit = false;

                            //     project_select.add_project_name_input = TextArea::default();
                            //     project_select.add_project_path_input = TextArea::default();

                            //     project_select.tab = 0;
                            //     project_select.item = 0;
                            // }
                            Data::Direction(false) => {
                                project_select.item -= 1;
                            }
                            _ => {}
                        }
                    }
                });
            } else {
                key_event_handler.bind_tab((Some(1), Some(2)), |app, data| {
                    if let Some(Popups::ProjectSelect(project_select)) =
                        app.drawer.active_popup.as_mut()
                    {
                        match data {
                            Data::Direction(true) => {
                                project_select.add_project_input_disable = false;

                                project_select.tab = 0;
                                project_select.item = 0;
                            }
                            Data::Direction(false) => {
                                project_select.item -= 1;
                            }
                            _ => {}
                        }
                    }
                });
            }
            key_event_handler.bind_tab((Some(1), None), |app, data| {
                if let Some(Popups::ProjectSelect(project_select)) =
                    app.drawer.active_popup.as_mut()
                {
                    match data {
                        Data::Direction(true) => {
                            project_select.item += 1;
                        }
                        Data::Direction(false) => {
                            project_select.item -= 1;
                        }
                        _ => {}
                    }
                }
            });
        }

        if self.edit_project {
            key_event_handler.bind_esc((Some(1), None), |app, _| {
                if let Some(Popups::ProjectSelect(project_select)) =
                    app.drawer.active_popup.as_mut()
                {
                    project_select.item = 3;
                }
            });
        } else {
            key_event_handler.bind_esc((Some(1), None), |app, _| {
                if let Some(Popups::ProjectSelect(project_select)) =
                    app.drawer.active_popup.as_mut()
                {
                    project_select.add_project_input_disable = true;
                }
            });
        }
        key_event_handler.bind_esc((Some(1), Some(2)), move |app, _| {
            if exitable && num_projects == 0 {
                app.drawer.close_popups();
            }
            if let Some(Popups::ProjectSelect(project_select)) = app.drawer.active_popup.as_mut() {
                project_select.add_project_name_input = TextArea::default();
                project_select.add_project_path_input = TextArea::default();

                project_select.tab = 0;
                project_select.item = 0;
                project_select.edit_project = false;
            }
        });
        key_event_handler.bind_esc((Some(1), Some(3)), move |app, _| {
            if exitable && num_projects == 0 {
                app.drawer.close_popups();
            }
            if let Some(Popups::ProjectSelect(project_select)) = app.drawer.active_popup.as_mut() {
                project_select.add_project_name_input = TextArea::default();
                project_select.add_project_path_input = TextArea::default();

                project_select.tab = 0;
                project_select.item = 0;
                project_select.edit_project = false;
            }
        });

        if !self.add_project_input_disable {
            key_event_handler.bind_input_field((Some(1), Some(1)), |app, data| {
                if let Some(Popups::ProjectSelect(project_select)) =
                    app.drawer.active_popup.as_mut()
                {
                    if let Data::Key(key_event) = data {
                        project_select.add_project_path_input.input(key_event);
                    }
                }
            });
            key_event_handler.bind_input_field((Some(1), Some(0)), |app, data| {
                if let Some(Popups::ProjectSelect(project_select)) =
                    app.drawer.active_popup.as_mut()
                {
                    if let Data::Key(key_event) = data {
                        project_select.add_project_name_input.input(key_event);
                    }
                }
            });
        }
        if self.add_project_input_disable {
            key_event_handler.bind_enter((Some(1), None), |app, _| {
                if let Some(Popups::ProjectSelect(project_select)) =
                    app.drawer.active_popup.as_mut()
                {
                    project_select.add_project_input_disable = false;
                }
            });
        } else {
            key_event_handler.bind_enter((Some(1), None), |app, _| {
                if let Some(Popups::ProjectSelect(project_select)) =
                    app.drawer.active_popup.as_mut()
                {
                    project_select.item = project_select.item.add(1).min(2);
                }
            });
            if valid {
                key_event_handler.bind_enter((Some(1), Some(2)), |app, _| {
                    if let Some(Popups::ProjectSelect(project_select)) =
                        app.drawer.active_popup.as_mut()
                    {
                        let name = project_select.add_project_name_input.lines()[0].to_string();
                        let path = project_select.add_project_path_input.lines()[0].to_string();

                        project_select.add_project_name_input = TextArea::default();
                        project_select.add_project_path_input = TextArea::default();

                        project_select.tab = 0;
                        project_select.item = 0;
                        let edit = project_select.edit_project;
                        project_select.edit_project = false;

                        if edit {
                            app.edit_project(name, path).unwrap();
                        } else {
                            app.create_project(name, path).unwrap();
                        }
                    }
                });
            }
            key_event_handler.bind_enter((Some(1), Some(3)), |app, _| {
                if let Some(Popups::ProjectSelect(project_select)) =
                    app.drawer.active_popup.as_mut()
                {
                    project_select.add_project_name_input = TextArea::default();
                    project_select.add_project_path_input = TextArea::default();

                    project_select.tab = 0;
                    project_select.item = 0;
                    project_select.edit_project = false;
                }
            });
        }

        self.add_project_name_input
            .set_style(Style::new().fg(if tab_selected {
                if self.item == 0 && !self.add_project_input_disable {
                    tailwind::SLATE.c300
                } else {
                    tailwind::STONE.c400
                }
            } else {
                tailwind::STONE.c500
            }));
        self.add_project_name_input.set_cursor_style(
            Style::new()
                .fg(if tab_selected {
                    if self.item == 0 && !self.add_project_input_disable {
                        tailwind::SLATE.c300
                    } else {
                        tailwind::STONE.c400
                    }
                } else {
                    tailwind::STONE.c500
                })
                .add_modifier(if tab_selected {
                    if self.item == 0 && !self.add_project_input_disable {
                        Modifier::REVERSED
                    } else {
                        Modifier::default()
                    }
                } else {
                    Modifier::default()
                }),
        );
        self.add_project_name_input.set_block(
            Block::bordered()
                .border_type(ratatui::widgets::BorderType::Thick)
                .style(Style::new().fg(if tab_selected {
                    if self.item == 0 && !self.add_project_input_disable {
                        if name_valid {
                            material::BLUE.c500
                        } else {
                            material::RED.c600
                        }
                    } else {
                        tailwind::SLATE.c500
                    }
                } else {
                    tailwind::STONE.c600
                }))
                .title(" name ")
                .title_style(Style::new().fg(if tab_selected {
                    if self.item == 0 && !self.add_project_input_disable {
                        material::BLUE.c600
                    } else {
                        if name_valid {
                            material::BLUE.c600
                        } else {
                            material::RED.c600
                        }
                    }
                } else {
                    tailwind::SLATE.c600
                })),
        );
        self.add_project_name_input
            .set_placeholder_text("Enter the project name");
        self.add_project_name_input
            .set_placeholder_style(Style::new().fg(material::GRAY.c700));

        self.add_project_path_input
            .set_style(Style::new().fg(if tab_selected {
                if self.item == 1 && !self.add_project_input_disable {
                    tailwind::SLATE.c300
                } else {
                    tailwind::STONE.c400
                }
            } else {
                tailwind::STONE.c500
            }));
        self.add_project_path_input.set_cursor_style(
            Style::new()
                .fg(if tab_selected {
                    if self.item == 1 && !self.add_project_input_disable {
                        tailwind::SLATE.c300
                    } else {
                        tailwind::STONE.c400
                    }
                } else {
                    tailwind::STONE.c500
                })
                .add_modifier(if tab_selected {
                    if self.item == 1 && !self.add_project_input_disable {
                        Modifier::REVERSED
                    } else {
                        Modifier::default()
                    }
                } else {
                    Modifier::default()
                }),
        );
        self.add_project_path_input.set_block(
            Block::bordered()
                .border_type(ratatui::widgets::BorderType::Thick)
                .style(Style::new().fg(if tab_selected {
                    if self.item == 1 && !self.add_project_input_disable {
                        if path_valid {
                            material::BLUE.c500
                        } else {
                            material::RED.c600
                        }
                    } else {
                        tailwind::SLATE.c500
                    }
                } else {
                    tailwind::STONE.c600
                }))
                .title(" path ")
                .title_style(Style::new().fg(if tab_selected {
                    if self.item == 1 && !self.add_project_input_disable {
                        material::BLUE.c600
                    } else {
                        if path_valid {
                            material::BLUE.c600
                        } else {
                            material::RED.c600
                        }
                    }
                } else {
                    tailwind::SLATE.c600
                })),
        );
        self.add_project_path_input
            .set_placeholder_text("Enter the project path");
        self.add_project_path_input
            .set_placeholder_style(Style::new().fg(material::GRAY.c700));

        let [name_input_area, _, path_input_area, _, actions_area] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(2),
            Constraint::Length(1),
        ])
        .areas(add_padding(
            area,
            Padding {
                left: 1,
                right: 2,
                top: 1,
                bottom: 0,
            },
        ));

        frame.render_widget(&self.add_project_name_input, name_input_area);
        frame.render_widget(&self.add_project_path_input, path_input_area);

        if self.edit_project {
            frame.render_widget(
                Line::from(vec![
                    Span::from(" Confirm ").style(
                        Style::new()
                            .fg(if tab_selected {
                                if valid {
                                    if self.item == 2 {
                                        tailwind::SLATE.c200
                                    } else {
                                        tailwind::SLATE.c300
                                    }
                                } else {
                                    tailwind::SLATE.c500
                                }
                            } else {
                                tailwind::SLATE.c600
                            })
                            .bg(if tab_selected && self.item == 2 {
                                if valid {
                                    material::BLUE.c600
                                } else {
                                    tailwind::SLATE.c800
                                }
                            } else {
                                tailwind::SLATE.c950
                            }),
                    ),
                    Span::from(" "),
                    Span::from(" No ").style(
                        Style::new()
                            .fg(if tab_selected && self.item == 3 {
                                tailwind::SLATE.c300
                            } else {
                                tailwind::RED.c500
                            })
                            .bg(if tab_selected && self.item == 3 {
                                material::RED.c500
                            } else {
                                tailwind::SLATE.c950
                            }),
                    ),
                ]),
                Layout::horizontal([Constraint::Length(14)])
                    .flex(ratatui::layout::Flex::Center)
                    .split(actions_area)[0],
            );
        } else {
            frame.render_widget(
                Line::from("Add").alignment(Alignment::Center).style(
                    Style::new()
                        .fg(if tab_selected {
                            if valid {
                                if self.item == 2 {
                                    tailwind::SLATE.c200
                                } else {
                                    tailwind::SLATE.c300
                                }
                            } else {
                                tailwind::SLATE.c500
                            }
                        } else {
                            tailwind::SLATE.c600
                        })
                        .bg(if tab_selected && self.item == 2 {
                            if valid {
                                material::BLUE.c700
                            } else {
                                tailwind::SLATE.c800
                            }
                        } else {
                            tailwind::SLATE.c950
                        }),
                ),
                Layout::horizontal([Constraint::Length(5)])
                    .flex(ratatui::layout::Flex::Center)
                    .split(actions_area)[0],
            );
        }
    }

    fn validate_name(&self, ranker: &Ranker<String>) -> bool {
        let name = self.add_project_name_input.lines();

        !name[0].is_empty()
            && !((!self.edit_project && ranker.get_project_names().contains(&name[0]))
                || (self.edit_project
                    && name[0]
                        != ranker
                            .get_project_by_index(self.project_list_selected_item)
                            .unwrap()
                            .name)
                    && ranker.get_project_names().contains(&name[0]))
    }
    fn validate_path(&self) -> bool {
        let path = self.add_project_path_input.lines();

        std::path::PathBuf::from(&path[0]).is_dir()
    }
}
