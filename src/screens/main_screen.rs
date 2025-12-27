use crate::KeyEventHandler;

use ratatui::Frame;

#[derive(Default)]
pub struct MainScreen {}
impl MainScreen {
    pub fn render(
        &mut self,
        frame: &mut Frame,
        key_event_handler: &mut KeyEventHandler,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}
