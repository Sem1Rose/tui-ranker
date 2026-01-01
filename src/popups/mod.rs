pub mod project_select;
pub mod results;

pub use project_select::{Phase as ProjectSelectPhase, ProjectSelect};
pub use results::Results;

pub enum Popups {
    ProjectSelect(ProjectSelect),
    Results(Results),
}

impl Default for Popups {
    fn default() -> Self {
        Self::ProjectSelect(ProjectSelect::default())
    }
}
