use crate::Drawer;
use crate::types::{Term, initialize_terminal};
use log::error;
use ranker::Ranker;
use ratatui::crossterm::event::{self, Event, KeyCode};
use std::collections::HashMap;
use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

pub struct App {
    ranker: Ranker<String>,
    terminal: Term,
    drawer: Drawer,
    project_table: HashMap<String, String>,
}

impl App {
    pub fn new() -> anyhow::Result<Self> {
        let root = dirs::config_dir()
            .expect("Couldn't get user's config dir")
            .join("tui-ranker");
        if !root.exists() {
            fs::create_dir(&root)?;
        }
        let ranker = Ranker::new().load_projects_from(&root)?;

        Ok(Self {
            terminal: initialize_terminal()?,
            drawer: Drawer::new(),
            project_table: Self::load_project_table(root, &ranker)?,
            ranker,
        })
    }

    fn load_project_table(
        root: PathBuf,
        ranker: &Ranker<String>,
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
        for (k, v) in &self.project_table {
            eprintln!("{k}: {v}");
        }
        loop {
            self.terminal
                .draw(|frame| {
                    let result = self.drawer.render_app(frame, &mut self.ranker);
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
