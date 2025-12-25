use ratatui::crossterm::ExecutableCommand;
use ratatui::crossterm::{self, terminal::EnterAlternateScreen};
use ratatui::prelude::*;
use std::io::stdout;

// pub type OptionalResult<T> = anyhow::Result<T, Option<Errors>>;
type TermBackend = CrosstermBackend<std::io::Stdout>;
pub type Term = Terminal<TermBackend>;

pub fn initialize_terminal() -> anyhow::Result<Term> {
    set_panic_hook();

    crossterm::terminal::enable_raw_mode()?;

    let mut backend = TermBackend::new(stdout());
    backend.execute(EnterAlternateScreen)?;

    let mut term = Terminal::new(backend)?;
    term.hide_cursor()?;

    Ok(term)
}

fn set_panic_hook() {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        ratatui::restore();
        hook(info);
    }));
}
