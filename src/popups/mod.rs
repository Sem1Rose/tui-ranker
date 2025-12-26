pub mod project_select;

pub use project_select::{Phase as ProjectSelectPhase, ProjectSelect};

pub enum Popups {
    ProjectSelect(ProjectSelect),
}

impl Default for Popups {
    fn default() -> Self {
        Self::ProjectSelect(ProjectSelect::default())
    }
}
