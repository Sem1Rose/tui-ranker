use ratatui::Frame;

#[derive(Default)]
pub enum Phase {
    #[default]
    Selecting,
    Done,
}

pub struct ProjectSelect {
    pub phase: Phase,
}

impl ProjectSelect {
    pub fn new() -> Self {
        ProjectSelect {
            phase: Phase::default(),
        }
    }

    pub fn render(&mut self, frame: &mut Frame) -> anyhow::Result<()> {
        Ok(())
    }
}
