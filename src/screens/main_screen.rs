use ratatui::Frame;

pub struct MainScreen {}
impl MainScreen {
    pub fn new() -> Self {
        MainScreen {}
    }

    pub fn render(&mut self, frame: &mut Frame) -> anyhow::Result<()> {
        Ok(())
    }
}
