use ratatui::prelude::*;

// pub type OptionalResult<T> = anyhow::Result<T, Option<Errors>>;
pub type Term = Terminal<CrosstermBackend<std::io::Stdout>>;
