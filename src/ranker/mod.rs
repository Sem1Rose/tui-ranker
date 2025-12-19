use anyhow::{Ok, bail};
use log::{debug, error, info};
use rand::seq::{IteratorRandom, SliceRandom};
use std::{fs::File, panic, path::PathBuf, str::FromStr, usize, vec};
use types::BitMask;

mod elo;
mod files;
mod types;

pub const WINDOW_SIZE: usize = 9;

pub fn shuffle<T: Clone>(arr: &[T]) -> Vec<T> {
    let mut shuffled = arr.to_vec();
    shuffled.shuffle(&mut rand::rng());

    shuffled
}

#[derive(Default)]
pub struct Project<T> {
    name: String,
    dir: PathBuf,

    items: Vec<(T, f32)>,
    bitmasks: Vec<BitMask>,
    results: Vec<BitMask>,

    pub num_rated_items: usize,
}

// #[derive(Default)]
pub struct Ranker<T> {
    selected_project: usize,
    root: PathBuf,
    projects: Vec<Project<T>>,

    window: Vec<u16>,
    item_a: usize,
    item_b: usize,
    item_a_won: bool,

    pub window_rated_items: usize,
}

impl<T> Ranker<T>
where
    T: FromStr + ToString + PartialEq<T> + Clone + Default + std::fmt::Debug,
{
    pub fn new(custom_root: Option<PathBuf>, default_project_name: String) -> anyhow::Result<Self> {
        let root = if let Some(root) = custom_root {
            root
        } else {
            dirs::config_dir()
                .expect("Couldn't get user's config dir")
                .join("ranker")
        };
        if !root.exists() {
            std::fs::create_dir(&root)?;
        }

        let mut projects: Vec<Project<T>> = root
            .read_dir()
            .unwrap()
            .filter_map(|x| x.ok())
            .filter(|x| x.path().is_dir())
            .inspect(|x| info!("{}", x.path().file_name().unwrap().to_str().unwrap()))
            .filter_map(|x| {
                Project::<T>::new(
                    &root,
                    x.path().file_name().unwrap().to_str().unwrap().to_string(),
                )
                .ok()
            }) // `new` will return an error if it can't create the necessary files, so the object is skipped
            .collect();
        if projects.is_empty() {
            projects.push(Project::<T>::new(&root, default_project_name)?);
        }

        Ok(Self {
            projects,
            root,
            item_a_won: true,
            ..Default::default()
        })
    }

    pub fn init(&mut self) -> anyhow::Result<()> {
        if !self.get_new_window() {
            info!("all images has been rated, resetting..");

            self.projects[self.selected_project].reset_bitmasks_and_results();

            _ = self.get_new_window();
        }

        if self.window.len() < 2 {
            info!(
                "selected project has less than two items: {}",
                self.get_project().name
            );
        } else {
            info!("got window {:?}", self.window);

            self.item_a = *self.window.iter().choose(&mut rand::rng()).unwrap() as usize;
        }

        Ok(())
    }

    pub fn sync_project(&mut self, items: Vec<T>) -> anyhow::Result<()> {
        self.projects[self.selected_project].initialize(items)?;
        if self.window.is_empty() {
            self.init()?;
        }

        Ok(())
    }
    pub fn get_project(&self) -> &Project<T> {
        &self.projects[self.selected_project]
    }
    pub fn get_project_names(&self) -> Vec<String> {
        self.projects.iter().map(|p| p.name.clone()).collect()
    }
    pub fn try_find_project(&self, name: &str) -> Option<usize> {
        self.projects.iter().position(|p| p.name == name)
    }
    pub fn select_project(&mut self, project: usize) -> anyhow::Result<()> {
        if self.selected_project >= self.projects.len() {
            panic!(
                "Trying to access index {project} but the length is {}",
                self.projects.len()
            )
        }
        self.selected_project = project;
        self.window = vec![];

        self.init()
    }
    pub fn try_select_project_by_name(&mut self, name: &str) -> anyhow::Result<()> {
        if let Some(i) = self.try_find_project(name) {
            self.select_project(i)?;

            return Ok(());
        }

        bail!("Couldn't find project {}", name)
    }
    pub fn create_project(&mut self, name: String) -> anyhow::Result<()> {
        self.projects.push(Project::new(&self.root, name)?);
        self.selected_project = self.projects.len() - 1;
        self.window = vec![];

        Ok(())
    }

    pub fn next(&mut self) -> anyhow::Result<Option<(T, T)>> {
        if self.window.len() < 2 {
            error!(
                "selected project has less than two items: {}",
                self.get_project().name
            );
            bail!("Not enough items in project");
        }

        let mut refresh = false;
        if self.check_window_done() {
            info!("current window is done, updating elo and getting a new one...");
            self.update_results_elos()?;

            if !self.get_new_window() {
                info!("all images has been rated, quitting..");
                return Ok(None);
            }
            info!("got a new window {:?}", self.window);

            self.window_rated_items = 0;
            refresh = true;
        }

        if refresh {
            self.item_a = *self.window.iter().choose(&mut rand::rng()).unwrap() as usize;
            self.item_b = self.choose_random_opponent(self.item_a);
        } else if self.item_a_won {
            let mut a_rated_all_window = true;
            for i in &self.window {
                if !self.projects[self.selected_project].bitmasks[self.item_a].check_bit(*i) {
                    a_rated_all_window = false;
                    break;
                }
            }

            if a_rated_all_window {
                info!(
                    "{} rated all the window, getting a new item from the window...",
                    self.item_a
                );

                let mut new_item_found = false;
                for i in shuffle(&self.window) {
                    for j in &self.window {
                        if !self.projects[self.selected_project].bitmasks[i as usize].check_bit(*j)
                        {
                            self.item_a = i as usize;
                            new_item_found = true;
                            break;
                        }
                    }
                    if new_item_found {
                        break;
                    }
                }
            }

            info!("getting a new item_b");
            self.item_b = self.choose_random_opponent(self.item_a);
        } else {
            let mut b_rated_all_window = true;
            for i in &self.window {
                if !self.projects[self.selected_project].bitmasks[self.item_b].check_bit(*i) {
                    b_rated_all_window = false;
                    break;
                }
            }

            if b_rated_all_window {
                info!(
                    "{} rated all the window, getting a new item from the window...",
                    self.item_b
                );

                let mut new_item_found = false;
                for i in shuffle(&self.window) {
                    for j in &self.window {
                        if !self.projects[self.selected_project].bitmasks[i as usize].check_bit(*j)
                        {
                            self.item_b = i as usize;
                            new_item_found = true;
                            break;
                        }
                    }
                    if new_item_found {
                        break;
                    }
                }
            }

            info!("getting a new item_b");
            self.item_a = self.choose_random_opponent(self.item_b);
        }

        Ok(Some((
            self.projects[self.selected_project].items[self.item_a]
                .0
                .clone(),
            self.projects[self.selected_project].items[self.item_b]
                .0
                .clone(),
        )))
    }

    pub fn log_result(&mut self, a_won: bool) -> anyhow::Result<()> {
        self.projects[self.selected_project].bitmasks[self.item_a].set_bit(self.item_b as u16);
        self.projects[self.selected_project].bitmasks[self.item_b].set_bit(self.item_a as u16);
        self.projects[self.selected_project].num_rated_items += 1;

        self.window_rated_items += 1;
        self.item_a_won = a_won;

        if a_won {
            self.projects[self.selected_project].results[self.item_a].set_bit(self.item_b as u16);
        } else {
            self.projects[self.selected_project].results[self.item_b].set_bit(self.item_a as u16);
        }

        self.projects[self.selected_project].cache_results()?;
        self.projects[self.selected_project].cache_bitmasks()?;
        Ok(())
    }

    pub fn get_total_ratings(&self) -> usize {
        (self.projects[self.selected_project].items.len()
            * self.projects[self.selected_project].items.len()
            - self.projects[self.selected_project].items.len())
            / 2
    }

    pub fn get_item_scores(&self) -> Vec<(T, f32)> {
        let mut sorted = self.projects[self.selected_project]
            .items
            .iter()
            .filter(|x| x.1.is_normal())
            .map(|x| x.clone())
            .collect::<Vec<_>>();
        sorted.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        sorted.reverse();

        sorted
    }

    pub fn get_item_num_played_games(&self, item: usize) -> usize {
        self.projects[self.selected_project].bitmasks[item].get_num_ones() as usize - 1
    }

    pub fn get_item_index(&self, item: &T) -> usize {
        self.projects[self.selected_project]
            .items
            .iter()
            .position(|(x, _)| x == item)
            .unwrap()
    }

    fn get_new_window(&mut self) -> bool {
        let mut all_done = true;
        for bitmask in &self.projects[self.selected_project].bitmasks {
            if bitmask.get_num_ones() < self.projects[self.selected_project].bitmasks.len() as u16 {
                all_done = false;
                break;
            }
        }
        if all_done {
            return false;
        }

        self.window_rated_items = 0;
        self.window = vec![];
        info!("initializing a new window...");
        // info!(
        //     "{:#?}",
        //     shuffle(&(0..bitmasks.len() as u16).collect::<Vec<_>>())
        // );
        // info!(
        //     "{:#?}",
        //     shuffle(&(0..bitmasks.len() as u16).collect::<Vec<_>>())
        // );
        for i in shuffle(
            &(0..self.projects[self.selected_project].bitmasks.len() as u16).collect::<Vec<_>>(),
        ) {
            if self.projects[self.selected_project].bitmasks[i as usize].get_num_ones()
                < self.projects[self.selected_project].bitmasks.len() as u16
            {
                self.window = vec![i];
                break;
            }
        }

        for i in shuffle(
            &(0..self.projects[self.selected_project].bitmasks.len() as u16).collect::<Vec<_>>(),
        ) {
            if self.window.iter().any(|x| *x == i) {
                continue;
            }

            let mut skip = false;
            for j in &self.window {
                if self.projects[self.selected_project].bitmasks[i as usize].check_bit(*j) {
                    skip = true;
                    break;
                }
            }
            if skip {
                continue;
            }

            self.window.push(i);
            if self.window.len() == WINDOW_SIZE {
                break;
            }
        }

        true
    }

    fn check_window_done(&self) -> bool {
        for i in &self.window {
            let mask = &self.projects[self.selected_project].bitmasks[*i as usize];
            for j in &self.window {
                if i == j {
                    continue;
                }

                if !mask.check_bit(*j) {
                    return false;
                }
            }
        }

        true
    }

    fn choose_random_opponent(&self, item: usize) -> usize {
        for opponent_index in shuffle(&self.window) {
            if opponent_index != item as u16
                && !self.projects[self.selected_project].bitmasks[item].check_bit(opponent_index)
            {
                info!("got {opponent_index}");
                return opponent_index as usize;
            }
        }

        usize::MAX
    }

    fn update_results_elos(&mut self) -> anyhow::Result<()> {
        let mut new_ratings: Vec<(&u16, (T, f32))> = vec![];
        for i in &self.window {
            for j in &self.window {
                if i == j {
                    continue;
                }

                let score =
                    self.projects[self.selected_project].results[*i as usize].check_bit(*j) as u8;

                let mut i_rating = &self.projects[self.selected_project].items[*i as usize].1;
                let mut j_rating = &self.projects[self.selected_project].items[*j as usize].1;

                let i_new_rating_pos = new_ratings.iter().position(|x| x.0 == i);
                if let Some(pos) = i_new_rating_pos {
                    i_rating = &new_ratings[pos].1.1;
                } else if !i_rating.is_normal() {
                    i_rating = &1000.0;
                }

                let j_new_rating_pos = new_ratings.iter().position(|x| x.0 == j);
                if let Some(pos) = j_new_rating_pos {
                    j_rating = &new_ratings[pos].1.1;
                } else if !j_rating.is_normal() {
                    j_rating = &1000.0;
                }

                let new_rating = elo::calc_new_rating(*i_rating, *j_rating, score);
                info!("{}x{}={} {}->{}", i, j, score, i_rating, new_rating);

                if i_new_rating_pos.is_some() {
                    new_ratings[i_new_rating_pos.unwrap()].1.1 = new_rating;
                } else {
                    new_ratings.push((
                        i,
                        (
                            self.projects[self.selected_project].items[*i as usize]
                                .0
                                .clone(),
                            new_rating,
                        ),
                    ));
                }

                self.projects[self.selected_project].results[*i as usize].reset_bit(*j);
            }
        }

        for i in new_ratings {
            // info!("{} {}->{}", i.0, items[*i.0 as usize].1, i.1.1);
            self.projects[self.selected_project].items[*i.0 as usize] = i.1;
        }

        self.projects[self.selected_project].cache_items()?;
        self.projects[self.selected_project].cache_results()?;

        Ok(())
    }
}

impl<T> Default for Ranker<T>
where
    T: FromStr + ToString + PartialEq<T> + Clone + Default + std::fmt::Debug,
{
    fn default() -> Self {
        let root = dirs::config_dir()
            .expect("Couldn't get user's config dir")
            .join("ranker");
        if !root.exists() {
            std::fs::create_dir(&root).expect("Couldn't create root directory");
        }

        let mut projects: Vec<Project<T>> = root
            .read_dir()
            .unwrap()
            .filter_map(|x| x.ok())
            .filter(|x| x.path().is_dir())
            .inspect(|x| info!("{}", x.path().file_name().unwrap().to_str().unwrap()))
            .filter_map(|x| {
                Project::<T>::new(
                    &root,
                    x.path().file_name().unwrap().to_str().unwrap().to_string(),
                )
                .ok()
            }) // `new` will return an error if it can't create the necessary files, so the object is skipped
            .collect();
        if projects.is_empty() {
            projects
                .push(Project::<T>::new(&root, "0".into()).expect("IDK just figure it out gng"));
        }

        Self {
            projects,
            root,
            item_a_won: true,
            selected_project: Default::default(),
            window: Default::default(),
            item_a: Default::default(),
            item_b: Default::default(),
            window_rated_items: Default::default(),
        }
    }
}

impl<T> Project<T>
where
    T: FromStr + ToString + PartialEq<T> + Clone + Default + std::fmt::Debug,
{
    pub fn new(root: &PathBuf, name: String) -> anyhow::Result<Self> {
        Self {
            dir: root.join(&name),
            name,
            ..Default::default()
        }
        .ensure_files_exist()?
        .load()
    }

    fn ensure_files_exist(self) -> anyhow::Result<Self> {
        if !self.dir.exists() {
            std::fs::create_dir(&self.dir)?;
        }
        if !self.dir.join(".items").exists() {
            File::create_new(self.dir.join(".items"))?;
        }
        if !self.dir.join(".bitmasks").exists() {
            File::create_new(self.dir.join(".bitmasks"))?;
        }
        if !self.dir.join(".results").exists() {
            File::create_new(self.dir.join(".results"))?;
        }

        Ok(self)
    }

    fn load(mut self) -> anyhow::Result<Self> {
        self.items = files::get_cached_items(&self.dir)?;
        self.bitmasks = files::get_cached_bitmasks(&self.dir)?;
        self.results = files::get_cached_results(&self.dir)?;

        Ok(self)
    }

    fn initialize(&mut self, items: Vec<T>) -> anyhow::Result<()> {
        let cached_items = self.items.clone();
        self.items = vec![];

        if cached_items.is_empty() {
            self.items = items.into_iter().map(|x| (x, f32::NAN)).collect();
            self.bitmasks = vec![];
            self.results = vec![];

            for i in 0..self.items.len() as u16 {
                let mut new_bitmask: BitMask = vec![].into();
                new_bitmask.fit_to_number_of_bits(self.items.len() as u16 - 1);
                new_bitmask.set_bit(i);
                self.bitmasks.push(new_bitmask);

                let mut result: BitMask = vec![].into();
                result.fit_to_number_of_bits(self.items.len() as u16 - 1);
                self.results.push(result);
            }
        } else {
            for (i, cached_item) in cached_items.iter().enumerate().rev() {
                if !items.iter().any(|x| *x == cached_item.0) {
                    debug!("Removing item {:?}", cached_item.0);

                    self.bitmasks.remove(i);
                    for bm in self.bitmasks.iter_mut() {
                        bm.discard_bit(i as u16);
                    }

                    self.results.remove(i);
                    for rs in self.results.iter_mut() {
                        rs.discard_bit(i as u16);
                    }
                } else {
                    self.items.push(cached_item.clone());
                }
            }

            let mut new_items: Vec<_> = items
                .into_iter()
                .filter(|new_item| !cached_items.iter().any(|x| x.0 == *new_item))
                .collect();

            new_items.shuffle(&mut rand::rng());
            for new_item in new_items.into_iter() {
                debug!("Adding item {:?}", new_item);

                let mut new_bitmask: BitMask = vec![].into();
                new_bitmask.fit_to_number_of_bits(self.items.len() as u16 - 1);
                new_bitmask.set_bit(self.items.len() as u16);
                self.bitmasks.push(new_bitmask);

                self.items.push((new_item, f32::NAN));

                let mut new_result: BitMask = vec![].into();
                new_result.fit_to_number_of_bits(self.items.len() as u16 - 1);
                self.results.push(new_result);
            }
            // for bitmask in self.bitmasks.iter_mut() {
            // }

            self.num_rated_items = (self
                .bitmasks
                .iter()
                .fold(0, |acc, x| acc + x.get_num_ones())
                as usize
                - self.bitmasks.len())
                / 2;
        }

        self.cache_items()?;
        self.cache_bitmasks()?;
        self.cache_results()?;

        Ok(())
    }

    fn cache_items(&self) -> anyhow::Result<()> {
        files::cache_items(&self.dir, &self.items)?;
        Ok(())
    }
    fn cache_bitmasks(&self) -> anyhow::Result<()> {
        files::cache_bitmasks(&self.dir, &self.bitmasks)?;
        Ok(())
    }
    fn cache_results(&self) -> anyhow::Result<()> {
        files::cache_results(&self.dir, &self.results)?;
        Ok(())
    }

    fn reset_bitmasks_and_results(&mut self) {
        for (i, bitmask) in self.bitmasks.iter_mut().enumerate() {
            *bitmask = vec![].into();

            bitmask.set_bit(i as u16);
            bitmask.fit_to_number_of_bits(self.items.len() as u16 - 1);
        }
        for result in self.results.iter_mut() {
            *result = vec![].into();
            result.fit_to_bytes(self.items.len() as u16);
        }
    }
}
