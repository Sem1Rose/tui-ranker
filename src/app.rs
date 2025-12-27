use crate::Drawer;
use crate::KeyEventHandler;
use crate::types::{Term, initialize_terminal};
use log::error;
use rand::seq::SliceRandom;
use ranker::Ranker;
use ratatui::crossterm::event::{self, Event};
use std::collections::HashMap;
use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

pub struct App {
    terminal: Term,
    key_event_handler: KeyEventHandler,
    pub project_table: HashMap<String, String>,

    pub ranker: Ranker<String>,
    pub drawer: Drawer,
    pub quit: bool,
}

impl App {
    pub fn new() -> anyhow::Result<Self> {
        let root = dirs::config_dir()
            .expect("Couldn't get user's config dir")
            .join("tui-ranker");
        if !root.exists() {
            fs::create_dir(&root)?;
        }
        let mut ranker = Ranker::new().load_projects_from(&root)?;

        Ok(Self {
            terminal: initialize_terminal()?,
            drawer: Drawer::new(),
            project_table: Self::load_project_table(root, &mut ranker)?,
            key_event_handler: KeyEventHandler::default(),
            ranker,
            quit: false,
        })
    }

    fn load_project_table(
        root: PathBuf,
        ranker: &mut Ranker<String>,
    ) -> anyhow::Result<HashMap<String, String>> {
        if !root.join(".projects").exists() {
            File::create(root.join(".projects"))?;
        }
        let mut project_table = fs::read_to_string(root.join(".projects"))
            .unwrap()
            .lines()
            .map(|x| {
                (
                    x.split(": ").nth(0).unwrap().to_string(),
                    x.split(": ").nth(1).unwrap().to_string(),
                )
            })
            .collect::<HashMap<String, String>>();

        project_table.retain(|k, _| ranker.try_find_project(k).is_some());
        for i in ranker
            .get_project_names()
            .iter()
            .filter(|&x| project_table.get(x).is_none())
        {
            ranker.try_delete_project_by_name(i)?;
        }

        let mut file = File::create(root.join(".projects"))?;
        for project in &project_table {
            writeln!(file, "{}: {}", project.0, project.1)?;
        }

        Ok(project_table)
    }

    fn save_project_table(&self) -> anyhow::Result<()> {
        let mut file = File::create(
            dirs::config_dir()
                .expect("Couldn't get user's config dir")
                .join("tui-ranker")
                .join(".projects"),
        )?;
        for project in &self.project_table {
            writeln!(file, "{}: {}", project.0, project.1)?;
        }

        Ok(())
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        loop {
            self.key_event_handler.clear();
            self.key_event_handler
                .bind_key((None, None), 'q', |app, _| app.quit = true);
            self.key_event_handler
                .bind_key((None, None), 'p', |_, _| panic!("PANIC"));

            self.terminal
                .draw(|frame| {
                    let result = self.drawer.render_app(
                        frame,
                        &mut self.ranker,
                        &mut self.key_event_handler,
                    );
                    if let Err(err) = result {
                        error!("Error while drawing: {}", err);
                    }
                })
                .map(|_| ())?;

            if let Ok(event) = event::read() {
                self.handle_event(event)?;
            }

            if self.quit {
                break;
            }
        }

        Ok(())
    }

    pub fn create_project(&mut self, name: String, path: String) -> anyhow::Result<()> {
        let entries = std::fs::read_dir(&path)?;

        self.ranker.create_project(&name)?;
        self.project_table.insert(name, path);
        self.save_project_table()?;

        let mut files = vec![];
        for entry in entries {
            let dir = entry?;

            if dir.file_name().to_str().is_some_and(|x| {
                x.to_lowercase().ends_with("png")
                    || x.to_lowercase().ends_with("jpg")
                    || x.to_lowercase().ends_with("jpeg")
                    || x.to_lowercase().ends_with("webp")
            }) {
                files.push(
                    dir.path()
                        // .canonicalize()
                        // .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string(),
                );
            }
        }
        files.shuffle(&mut rand::rng());

        self.ranker.sync_project(files)?;

        Ok(())
    }
    pub fn edit_project(&mut self, name: String, path: String) -> anyhow::Result<()> {
        let entries = std::fs::read_dir(&path)?;

        if let Some(crate::popups::Popups::ProjectSelect(project_select)) =
            self.drawer.active_popup.as_ref()
        {
            let old_name =
                &self.ranker.get_project_names()[project_select.project_list_selected_item];

            self.project_table.remove(old_name);
            self.project_table.insert(name.clone(), path.clone());
            self.save_project_table()?;

            let mut files = vec![];
            for entry in entries {
                let dir = entry?;

                if dir.file_name().to_str().is_some_and(|x| {
                    x.to_lowercase().ends_with("png")
                        || x.to_lowercase().ends_with("jpg")
                        || x.to_lowercase().ends_with("jpeg")
                        || x.to_lowercase().ends_with("webp")
                }) {
                    files.push(
                        dir.path()
                            // .canonicalize()
                            // .unwrap()
                            .to_str()
                            .unwrap()
                            .to_string(),
                    );
                }
            }
            files.shuffle(&mut rand::rng());

            self.ranker.rename_project(old_name, &name)?;
            self.ranker
                .select_project(project_select.project_list_selected_item)?;
            self.ranker.sync_project(files)?;
        }

        Ok(())
    }
    pub fn delete_project(&mut self) -> anyhow::Result<()> {
        if let Some(crate::popups::Popups::ProjectSelect(project_select)) =
            self.drawer.active_popup.as_ref()
        {
            self.ranker
                .delete_project(project_select.project_list_selected_item)?;

            self.project_table
                .retain(|k, _| self.ranker.try_find_project(k).is_some());
            self.save_project_table()?;
        }

        Ok(())
    }

    fn handle_event(&mut self, event: Event) -> anyhow::Result<()> {
        match event {
            Event::Key(event) => {
                if let Some((callback, data)) = self
                    .key_event_handler
                    .handle_key_event(event, &self.drawer)?
                {
                    callback(self, data);
                }
            }
            Event::FocusGained => (),
            Event::FocusLost => (),
            Event::Mouse(_) => (),
            Event::Paste(_) => (),
            Event::Resize(_, _) => (),
        }

        Ok(())
    }
}
