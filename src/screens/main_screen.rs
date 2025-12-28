use crate::KeyEventHandler;
use crate::image_backend::RatatuiImage;
use crate::screens::Screens;

use ratatui::Frame;
use ratatui::layout::Rect;

#[derive(Default)]
pub struct MainScreen {
    pub item_a: String,
    pub item_b: String,
}
impl MainScreen {
    pub fn render(
        &mut self,
        frame: &mut Frame,
        key_event_handler: &mut KeyEventHandler,
        image_backend: &mut RatatuiImage,
    ) -> anyhow::Result<()> {
        key_event_handler.bind_horizontal((None, None), |app, _| {
            if let Some(Screens::MainScreen) = app.drawer.current_screen {
                let main_screen = &mut app.drawer.main_screen;

                let result = app.ranker.get_next();
                if let Ok(Some(x)) = result {
                    (main_screen.item_a, main_screen.item_b) = x;
                }
            }
        });
        key_event_handler.bind_enter((None, None), |app, _| {
            if let Some(Screens::MainScreen) = app.drawer.current_screen {
                app.ranker.get_new_window();
                app.drawer
                    .image_backend
                    .filter_cached_images(&app.ranker.get_window_items());
                app.drawer
                    .image_backend
                    .preload_images(&app.ranker.get_window_items());

                let main_screen = &mut app.drawer.main_screen;
                let result = app.ranker.get_next();
                if let Ok(Some(x)) = result {
                    (main_screen.item_a, main_screen.item_b) = x;
                }
            }
        });

        let height = frame.area().height / 2;
        let a = Rect {
            x: 0,
            y: 0,
            width: frame.area().width,
            height,
        };
        let b = Rect {
            x: 0,
            y: height,
            width: frame.area().width,
            height,
        };

        image_backend.draw_image(&self.item_a, a, frame);
        image_backend.draw_image(&self.item_b, b, frame);

        Ok(())
    }
}
