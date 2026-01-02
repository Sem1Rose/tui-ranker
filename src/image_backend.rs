use anyhow::bail;
use log::error;
use ratatui::{
    Frame,
    layout::{Constraint, Rect, Size},
};
use ratatui_image::{Image, Resize, picker::Picker, protocol::Protocol};
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use crate::helpers::center_rect;

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

    cache_dir: PathBuf,
    pub loading: u8,
}
impl Default for RatatuiImage {
    fn default() -> Self {
        let (tx_main, rx_main) = mpsc::channel();

        let tx_load = Self::start_load_thread(&tx_main);

        Self {
            hashed_images: HashMap::new(),
            preload_images: vec![],
            rx_main,
            tx_load,
            size: None,
            cache_dir: PathBuf::new(),
            loading: 0,
        }
    }
}

impl RatatuiImage {
    fn start_load_thread(tx_main: &Sender<LoadResult>) -> Sender<LoadResize> {
        let (tx_load, rx_load) = mpsc::channel::<LoadResize>();

        let tx_main = tx_main.clone();
        let picker = Picker::from_query_stdio().unwrap_or_else(|_| {
            error!("error querying graphics capabilities");
            Picker::halfblocks()
        });

        thread::spawn(move || {
            let mut cache_dir: PathBuf = PathBuf::default();
            let mut size: Size = Size::default();

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
                                if _cache_dir.join(&name).exists() {
                                    let reader;
                                    let result = image::ImageReader::open(&_cache_dir.join(&name));
                                    if let Err(err) = result {
                                        bail!(
                                            "Failed to open {}: {}",
                                            _cache_dir.join(&name).display(),
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
                                        360,
                                        ratatui_image::FilterType::CatmullRom,
                                    );

                                    if resized.save(_cache_dir.join(&name)).is_ok() {
                                        decoded = resized;
                                    }
                                }

                                let protocol = _picker.new_protocol(
                                    decoded,
                                    Rect {
                                        x: 0,
                                        y: 0,
                                        width: size.width,
                                        height: size.height,
                                    },
                                    Resize::Scale(Some(ratatui_image::FilterType::Triangle)),
                                )?;

                                Ok(protocol)
                            })();

                            tx_main.send((path, result))
                        });
                    }
                    LoadResize::Resize(_size) => {
                        size = _size;
                    }
                    LoadResize::CacheDir(_cache_dir) => {
                        if !_cache_dir.is_dir() {
                            fs::create_dir(&_cache_dir).unwrap();
                        }
                        cache_dir = _cache_dir;
                    }
                }
            }
        });

        tx_load
    }

    fn hash_image(&mut self, path: String) {
        self.hashed_images.insert(path.clone(), None);

        _ = self.tx_load.send(LoadResize::Load(path));
        self.loading += 1;
    }

    pub fn update(&mut self) {
        // let mut errored_ids = vec![];
        for (name, result) in self.rx_main.try_iter() {
            if let Ok(protocol) = result {
                if self.hashed_images.contains_key(&name) {
                    _ = self.hashed_images.get_mut(&name).unwrap().insert(protocol);
                    self.loading -= 1;
                }
            } else if let Err(e) = result {
                error!("={:?}= {e}", PathBuf::from(&name).file_name().unwrap());

                if self
                    .cache_dir
                    .join(&PathBuf::from(&name).file_name().unwrap())
                    .is_file()
                {
                    _ = fs::remove_file(
                        self.cache_dir
                            .join(&PathBuf::from(&name).file_name().unwrap()),
                    );

                    _ = self.tx_load.send(LoadResize::Load(name));
                } else {
                    self.loading -= 1;
                }
            }
        }

        // for id in errored_ids {
        //     self.remove_cached_image(&id);
        // }
    }

    pub fn draw_image(&mut self, name: &str, area: Rect, frame: &mut Frame) -> bool {
        let mut drawn = false;
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

            return false;
        }

        if let Some(value) = self.hashed_images.get(name) {
            if let Some(protocol) = value {
                let Size { width, height } = protocol.area().as_size();

                frame.render_widget(
                    Image::new(protocol),
                    center_rect(area, Constraint::Length(width), Constraint::Length(height)),
                );
                drawn = true;
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

        return drawn;
    }

    pub fn filter_cached_images(&mut self, filter: &[&String]) {
        self.hashed_images.retain(|key, _| filter.contains(&key));
    }

    pub fn preload_images(&mut self, images: &[&String]) {
        self.preload_images = images.iter().map(|&x| x.clone()).collect()
    }

    pub fn change_root(&mut self, root: &PathBuf) {
        self.cache_dir = root.join(".cache");
        _ = self
            .tx_load
            .send(LoadResize::CacheDir(self.cache_dir.clone()));
    }
}
