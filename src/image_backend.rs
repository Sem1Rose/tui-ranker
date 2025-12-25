use anyhow::Context;
use log::error;
use ratatui::{Frame, prelude::Rect};
use ratatui_image::{Image, Resize, picker::Picker, protocol::Protocol};
use std::{
    collections::HashMap,
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

// struct LoadedImage(String, anyhow::Result<Protocol>);
type LoadedImage = (String, anyhow::Result<Protocol>);

enum LoadResize {
    Load(String),
    Resize(Rect),
}

pub struct RatatuiImage {
    hashed_images: HashMap<String, Option<Protocol>>,

    tx_load: Sender<LoadResize>,
    rx_main: Receiver<LoadedImage>,

    size: Option<Rect>,
}

impl RatatuiImage {
    pub fn new() -> Self {
        let (tx_main, rx_main) = mpsc::channel();

        let tx_load = Self::start_load_thread(&tx_main);

        Self {
            hashed_images: HashMap::new(),
            rx_main,
            tx_load,
            size: None,
        }
    }

    fn start_load_thread(tx_main: &Sender<LoadedImage>) -> Sender<LoadResize> {
        let (tx_load, rx_load) = mpsc::channel::<LoadResize>();

        let tx_main = tx_main.clone();
        let picker;
        if let Ok(p) = Picker::from_query_stdio() {
            picker = p;
            // picker = Picker::from_fontsize((7, 14));
        } else {
            error!("error querying graphics capabilities");
            picker = Picker::from_fontsize((7, 14));
        }
        thread::spawn(move || {
            let mut size = None;
            for path in rx_load.iter() {
                match path {
                    LoadResize::Load(name) => {
                        let tx_main = tx_main.clone();

                        let _picker = picker.clone();
                        thread::spawn(move || {
                            let result = (|| -> anyhow::Result<_> {
                                let reader = image::ImageReader::open(&name)
                                    .context("Failed to open image file")?;
                                let decoded =
                                    reader.decode().context("Failed to decode image data")?;

                                let protocol = _picker.new_protocol(
                                    decoded,
                                    size.unwrap(),
                                    Resize::Scale(Some(ratatui_image::FilterType::Triangle)),
                                )?;

                                Ok(protocol)
                            })();

                            tx_main.send((name, result))
                        });
                    }
                    LoadResize::Resize(_size) => {
                        size = Some(_size);
                    }
                }
            }
        });

        tx_load
    }

    fn hash_image(&mut self, path: String) {
        self.hashed_images.insert(path.clone(), None);

        _ = self.tx_load.send(LoadResize::Load(path));
    }

    pub fn update(&mut self) {
        let mut errored_ids = vec![];
        for (name, result) in self.rx_main.try_iter() {
            if let Ok(protocol) = result {
                if self.hashed_images.contains_key(&name) {
                    _ = self.hashed_images.get_mut(&name).unwrap().insert(protocol);
                }
            } else {
                errored_ids.push(name);
            }
        }

        for id in errored_ids {
            self.remove_cached_image(id);
        }
    }

    pub fn draw_image(&mut self, name: &str, area: Rect, frame: &mut Frame) {
        if self.size.is_none() {
            _ = self.tx_load.send(LoadResize::Resize(area));
            self.size = Some(area);
        } else if self.size.unwrap() != area {
            _ = self.tx_load.send(LoadResize::Resize(area));
            self.size = Some(area);

            let mut paths = self.hashed_images.clone();
            self.hashed_images.clear();
            for (path, _) in paths.drain() {
                self.hash_image(path);
            }

            return;
        }

        if let Some(value) = self.hashed_images.get(name) {
            if let Some(protocol) = value {
                frame.render_widget(Image::new(protocol), area);
            }
        } else {
            self.hash_image(name.to_string());
        }
    }

    pub fn filter_cached_images(&mut self, filter: &[String]) {
        self.hashed_images.retain(|key, _| filter.contains(key));
    }

    fn remove_cached_image(&mut self, name: String) {
        self.hashed_images.remove(&name);
    }
}
