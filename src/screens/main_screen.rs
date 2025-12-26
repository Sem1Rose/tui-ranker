use ratatui::Frame;

#[derive(Default)]
pub struct MainScreen {}
impl MainScreen {
    pub fn render(&mut self, frame: &mut Frame) -> anyhow::Result<()> {
        Ok(())
    }
}
