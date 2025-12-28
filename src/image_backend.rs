use anyhow::bail;
use log::error;
use ratatui::{Frame, layout::Size, prelude::Rect};
use ratatui_image::{Image, Resize, picker::Picker, protocol::Protocol};
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

type LoadResult = (String, anyhow::Result<Protocol>);

enum LoadResize {
    Load(String),
    Resize(Size),
    CacheDir(PathBuf),
}

pub struct RatatuiImage {
    hashed_images: HashMap<String, Option<Protocol>>,
    preload_images: Vec<String>,

    tx_load: Sender<LoadResize>,
    rx_main: Receiver<LoadResult>,

    size: Option<Size>,
}

impl RatatuiImage {
    pub fn new() -> Self {
        let (tx_main, rx_main) = mpsc::channel();

        let tx_load = Self::start_load_thread(&tx_main);

        Self {
            hashed_images: HashMap::new(),
            preload_images: vec![],
            rx_main,
            tx_load,
            size: None,
        }
    }

    fn start_load_thread(tx_main: &Sender<LoadResult>) -> Sender<LoadResize> {
        let (tx_load, rx_load) = mpsc::channel::<LoadResize>();

        let tx_main = tx_main.clone();
        let picker = Picker::from_query_stdio().unwrap_or_else(|_| {
            error!("error querying graphics capabilities");
            Picker::from_fontsize((7, 14))
        });

        thread::spawn(move || {
            let mut cache_dir: Option<PathBuf> = None;
            let mut size: Option<Size> = None;

            for path in rx_load.iter() {
                match path {
                    LoadResize::Load(path) => {
                        let tx_main = tx_main.clone();

                        let name = PathBuf::from(&path)
                            .file_name()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .to_string();
                        let _picker = picker.clone();
                        let _cache_dir = cache_dir.clone();
                        thread::spawn(move || {
                            let result = (|| -> anyhow::Result<_> {
                                let mut decoded;
                                if _cache_dir.as_ref().unwrap().join(&name).exists() {
                                    let reader;
                                    let result = image::ImageReader::open(
                                        &_cache_dir.as_ref().unwrap().join(&name),
                                    );
                                    if let Err(err) = result {
                                        bail!(
                                            "Failed to open {}: {}",
                                            _cache_dir.as_ref().unwrap().join(&name).display(),
                                            err
                                        );
                                    } else {
                                        reader = result.unwrap();
                                    }

                                    let result = reader.decode();
                                    if let Err(err) = result {
                                        bail!("{}", err);
                                    } else {
                                        decoded = result.unwrap();
                                    }
                                } else {
                                    let reader;
                                    let result = image::ImageReader::open(&path);
                                    if let Err(err) = result {
                                        bail!("Failed to open {}: {}", path, err);
                                    } else {
                                        reader = result.unwrap();
                                    }

                                    let result = reader.decode();
                                    if let Err(err) = result {
                                        bail!("Failed to decode image {}: {}", path, err);
                                    } else {
                                        decoded = result.unwrap();
                                    }

                                    let resized = decoded.resize(
                                        1000000,
                                        240,
                                        ratatui_image::FilterType::CatmullRom,
                                    );

                                    if resized
                                        .save(_cache_dir.as_ref().unwrap().join(&name))
                                        .is_ok()
                                    {
                                        decoded = resized;
                                    }
                                }

                                let protocol = _picker.new_protocol(
                                    decoded,
                                    Rect {
                                        x: 0,
                                        y: 0,
                                        width: size.unwrap().width,
                                        height: size.unwrap().height,
                                    },
                                    Resize::Scale(Some(ratatui_image::FilterType::Triangle)),
                                )?;

                                Ok(protocol)
                            })();

                            tx_main.send((path, result))
                        });
                    }
                    LoadResize::Resize(_size) => {
                        size = Some(_size);
                    }
                    LoadResize::CacheDir(_cache_dir) => {
                        if !_cache_dir.is_dir() {
                            fs::create_dir(&_cache_dir).unwrap();
                        }
                        cache_dir = Some(_cache_dir);
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
            } else if let Err(e) = result {
                error!("{}", e);
                errored_ids.push(name);
            }
        }

        for id in errored_ids {
            self.remove_cached_image(&id);
        }
    }

    pub fn draw_image(&mut self, name: &str, area: Rect, frame: &mut Frame) {
        if self.size.is_none() {
            _ = self.tx_load.send(LoadResize::Resize(area.as_size()));
            self.size = Some(area.as_size());
        } else if self.size.unwrap() != area.as_size() {
            _ = self.tx_load.send(LoadResize::Resize(area.as_size()));
            self.size = Some(area.as_size());

            let mut paths = self.hashed_images.clone();
            self.hashed_images.clear();
            for (path, _) in paths.drain() {
                self.hash_image(path);
            }

            let preload_images = self.preload_images.clone();
            self.preload_images.clear();
            for path in preload_images {
                if let None = self.hashed_images.get(&path) {
                    self.hash_image(path);
                }
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

        let preload_images = self.preload_images.clone();
        self.preload_images.clear();
        for path in preload_images {
            if let None = self.hashed_images.get(&path) {
                self.hash_image(path);
            }
        }
    }

    pub fn filter_cached_images(&mut self, filter: &[&String]) {
        self.hashed_images.retain(|key, _| filter.contains(&key));
    }

    pub fn preload_images(&mut self, images: &[&String]) {
        self.preload_images = images.iter().map(|&x| x.clone()).collect()
    }

    pub fn change_root(&self, root: &PathBuf) {
        _ = self.tx_load.send(LoadResize::CacheDir(root.join(".cache")))
    }

    fn remove_cached_image(&mut self, name: &str) {
        self.hashed_images.remove(name);
    }
}
