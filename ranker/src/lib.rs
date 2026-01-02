// use anyhow::bail;
use bitfield::BitField;
use log::{debug, info, warn};
use rand::seq::{IteratorRandom, SliceRandom};
use std::{
    fmt,
    fs::{self, File},
    panic,
    path::PathBuf,
    str::FromStr,
    usize, vec,
};

mod bitfield;
mod elo;
mod files;

pub const DEFUALT_WINDOW_SIZE: usize = 9;

pub fn shuffle<T: Clone>(arr: &[T]) -> Vec<T> {
    let mut shuffled = arr.to_vec();
    shuffled.shuffle(&mut rand::rng());

    shuffled
}

type Result<T> = anyhow::Result<T, ProjectsError>;
#[derive(Debug)]
pub enum ProjectsError {
    NoProjects,
    NotEnoughItems,
    ProjectExists,
    Other(anyhow::Error),
}
impl fmt::Display for ProjectsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoProjects => write!(f, "No projects were loaded or no projects found!"),
            Self::NotEnoughItems => write!(f, "Selected project has less than two items"),
            Self::ProjectExists => write!(f, "Project already exists"),
            Self::Other(err) => write!(f, "{err}"),
        }
    }
}
impl std::error::Error for ProjectsError {}
impl From<anyhow::Error> for ProjectsError {
    fn from(err: anyhow::Error) -> Self {
        Self::Other(err)
    }
}
impl From<std::io::Error> for ProjectsError {
    fn from(err: std::io::Error) -> Self {
        Self::Other(err.into())
    }
}

#[derive(Default)]
pub struct Ranker<T> {
    selected_project: usize,
    root: PathBuf,
    projects: Vec<Project<T>>,
    no_projects: bool,

    window_size: usize,
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
    pub fn new() -> Self {
        Self {
            item_a_won: true,
            window_size: DEFUALT_WINDOW_SIZE,
            no_projects: true,
            ..Default::default()
        }
    }

    pub fn with_window_size(mut self, custom_window_size: usize) -> Self {
        self.window_size = custom_window_size;
        self
    }

    pub fn load_projects(self) -> Result<Self> {
        self.load_projects_from(&".".into())
    }
    pub fn load_projects_from(mut self, root: &PathBuf) -> Result<Self> {
        self.root = root.join("ranker");
        if !self.root.exists() {
            std::fs::create_dir(&self.root)?;
        }

        self.projects = self
            .root
            .read_dir()
            .unwrap()
            .filter_map(|x| x.ok())
            .filter(|x| x.path().is_dir())
            .filter_map(|x| {
                Project::<T>::new(
                    &self.root,
                    x.path().file_name().unwrap().to_str().unwrap().to_string(),
                )
                .ok()
            }) // `new` will return an error if it can't create the necessary files, so the object is skipped
            .collect();
        self.projects.sort_by(|a, b| a.name.cmp(&b.name));
        if !self.projects.is_empty() {
            self.no_projects = false;
        }

        Ok(self)
    }

    pub fn init(&mut self) -> Result<()> {
        if self.no_projects {
            return Err(ProjectsError::NoProjects);
        }

        if !self.get_new_window() {
            info!("all images has been rated, resetting..");

            self.projects[self.selected_project].reset_bitmasks_and_results();

            _ = self.get_new_window();
        }

        if self.window.len() < 2 {
            warn!(
                "selected project has less than two items: {}",
                self.get_selected_project()?.name
            );
        } else {
            debug!("got window {:?}", self.window);

            self.item_a = *self.window.iter().choose(&mut rand::rng()).unwrap() as usize;
        }

        Ok(())
    }

    pub fn get_selected_project(&self) -> Result<&Project<T>> {
        if self.no_projects {
            Err(ProjectsError::NoProjects)
        } else {
            Ok(&self.projects[self.selected_project])
        }
    }
    pub fn get_project_by_index(&self, index: usize) -> Result<&Project<T>> {
        if index >= self.projects.len() {
            Err(ProjectsError::NotEnoughItems)
        } else {
            Ok(&self.projects[index])
        }
    }
    pub fn get_num_projects(&self) -> usize {
        self.projects.len()
    }
    pub fn get_project_names(&self) -> Vec<String> {
        self.projects.iter().map(|p| p.name.clone()).collect()
    }
    pub fn try_find_project(&self, name: &str) -> Option<usize> {
        self.projects.iter().position(|p| p.name == name)
    }
    pub fn sync_project(&mut self, items: Vec<T>) -> Result<()> {
        self.projects[self.selected_project].initialize(items)?;
        if self.window.is_empty() {
            self.init()?;
        }

        Ok(())
    }
    pub fn select_project(&mut self, project: usize) -> Result<()> {
        if project >= self.projects.len() {
            panic!(
                "Trying to access index {project} but the length is {}",
                self.projects.len()
            )
        }
        self.selected_project = project;
        self.window = vec![];

        self.init()
    }
    pub fn try_select_project_by_name(&mut self, name: &str) -> Result<bool> {
        if let Some(i) = self.try_find_project(name) {
            self.select_project(i)?;

            return Ok(true);
        }

        // bail!("Couldn't find project {}", name)
        Ok(false)
    }
    pub fn create_project(&mut self, name: &str) -> Result<()> {
        if self.projects.iter().any(|x| x.name == name) {
            return Err(ProjectsError::ProjectExists);
        }

        self.projects
            .push(Project::new(&self.root, name.to_string())?);
        self.selected_project = self.projects.len() - 1;
        self.window = vec![];
        self.no_projects = false;

        Ok(())
    }
    pub fn rename_project(&mut self, name: &str, new_name: &str) -> Result<()> {
        if name == new_name {
            return Ok(());
        }

        if self.projects.iter().any(|x| x.name == name) {
            return Err(ProjectsError::ProjectExists);
        }

        if let Some(index) = self.try_find_project(name) {
            self.projects[index].rename(&self.root, new_name)?;

            // self.select_project(index)?;

            Ok(())
        } else {
            Err(ProjectsError::NoProjects)
        }
    }
    pub fn delete_project(&mut self, project: usize) -> Result<()> {
        if project >= self.projects.len() {
            panic!(
                "Trying to delete index {project} but the length is {}",
                self.projects.len()
            )
        }

        fs::remove_dir_all(self.root.join(&self.projects[project].name))?;
        self.projects.remove(project);
        if self.projects.len() == 0 {
            self.no_projects = true;
        } else if self.selected_project == project {
            if self.selected_project == self.projects.len() {
                self.select_project(self.selected_project - 1)?;
            }
        }

        Ok(())
    }
    pub fn try_delete_project_by_name(&mut self, name: &str) -> Result<bool> {
        if let Some(i) = self.try_find_project(name) {
            self.delete_project(i)?;

            return Ok(true);
        }

        // bail!("Couldn't find project {}", name)
        Ok(false)
    }
    pub fn get_window_items(&self) -> Vec<&T> {
        self.window
            .iter()
            .map(|i| &self.get_selected_project().unwrap().items[*i as usize].0)
            .collect()
    }

    pub fn get_next(&mut self) -> Result<Option<(T, T)>> {
        if self.no_projects {
            return Err(ProjectsError::NoProjects);
        }

        if self.window.len() < 2 {
            warn!(
                "selected project has less than two items: {}",
                self.get_selected_project()?.name
            );
            return Err(ProjectsError::NotEnoughItems);
        }

        let mut refresh = false;
        if self.check_window_done() {
            info!("current window is done, updating elo and getting a new one...");
            self.update_results_elos()?;

            if !self.get_new_window() {
                info!("all images has been rated, quitting..");
                return Ok(None);
            }
            debug!("got a new window {:?}", self.window);

            refresh = true;
        }

        if refresh {
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
                debug!(
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

            debug!("getting a new item_b");
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

            debug!("getting a new item_b");
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

    pub fn log_result(&mut self, a_won: bool) -> Result<()> {
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

    pub fn get_num_ratings_in_window(&self) -> usize {
        (1..self.window_size).sum()
    }

    pub fn get_new_window(&mut self) -> bool {
        let mut all_done = true;
        for bitmask in &self.get_selected_project().unwrap().bitmasks {
            if bitmask.get_num_ones() < self.get_selected_project().unwrap().items.len() as u16 {
                all_done = false;
                break;
            }
        }
        if all_done {
            return false;
        }

        self.window_rated_items = 0;
        self.window = vec![];
        debug!("initializing a new window...");
        for i in shuffle(
            &(0..self.get_selected_project().unwrap().bitmasks.len() as u16).collect::<Vec<_>>(),
        ) {
            if self.get_selected_project().unwrap().bitmasks[i as usize].get_num_ones()
                < self.get_selected_project().unwrap().bitmasks.len() as u16
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
                if self.get_selected_project().unwrap().bitmasks[i as usize].check_bit(*j) {
                    skip = true;
                    break;
                }
            }
            if skip {
                continue;
            }

            self.window.push(i);
            if self.window.len() == self.window_size {
                break;
            }
        }

        self.item_a = *self.window.iter().choose(&mut rand::rng()).unwrap() as usize;

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
                debug!("got {opponent_index}");
                return opponent_index as usize;
            }
        }

        usize::MAX
    }

    fn update_results_elos(&mut self) -> Result<()> {
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
                debug!("{}x{}={} {}->{}", i, j, score, i_rating, new_rating);

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

#[derive(Default)]
pub struct Project<T> {
    pub name: String,
    pub dir: PathBuf,

    pub items: Vec<(T, f32)>,
    pub bitmasks: Vec<BitField>,
    pub results: Vec<BitField>,

    pub total_ratings: usize,
    pub num_rated_items: usize,
}

impl<T> Project<T>
where
    T: FromStr + ToString + PartialEq<T> + Clone + Default + std::fmt::Debug,
{
    pub fn new(root: &PathBuf, name: String) -> Result<Self> {
        Self {
            dir: root.join(&name),
            name,
            ..Default::default()
        }
        .ensure_files_exist()?
        .load()
    }

    pub fn rename(&mut self, root: &PathBuf, name: &str) -> Result<()> {
        fs::rename(&self.dir, root.join(name))?;
        self.dir = root.join(&name);
        self.name = name.to_string();

        Ok(())
    }

    fn ensure_files_exist(self) -> Result<Self> {
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

    fn load(mut self) -> Result<Self> {
        self.items = files::get_cached_items(&self.dir)?;
        self.bitmasks = files::get_cached_bitmasks(&self.dir)?;
        self.results = files::get_cached_results(&self.dir)?;
        if !self.items.is_empty() && (self.bitmasks.is_empty() || self.results.is_empty()) {
            self.reset_bitmasks_and_results();
        }

        self.num_rated_items = (self
            .bitmasks
            .iter()
            .fold(0, |acc, x| acc + x.get_num_ones()) as usize
            - self.bitmasks.len())
            / 2;
        self.total_ratings = (self.items.len() * self.items.len() - self.items.len()) / 2;

        Ok(self)
    }

    fn initialize(&mut self, items: Vec<T>) -> Result<()> {
        // let mut cached_items = self.items.clone();
        // self.items = vec![];

        if self.items.is_empty() || items.is_empty() {
            self.items = items.into_iter().map(|x| (x, f32::NAN)).collect();

            self.reset_bitmasks_and_results();
            // for i in 0..self.items.len() as u16 {
            //     let mut new_bitmask: BitField = vec![].into();
            //     new_bitmask.fit_to_number_of_bits(self.items.len() as u16);
            //     // new_bitmask.set_bit(i);
            //     self.bitmasks.push(new_bitmask);

            //     let mut result: BitField = vec![].into();
            //     result.fit_to_number_of_bits(self.items.len() as u16);
            //     self.results.push(result);
            // }
        } else {
            for i in (0..self.items.len()).rev() {
                if !items.iter().any(|x| *x == self.items[i].0) {
                    debug!("Removing item {:?}", self.items[i].0);
                    self.items.remove(i);

                    self.bitmasks.remove(i);
                    for bm in self.bitmasks.iter_mut() {
                        bm.discard_bit(i as u16);
                    }

                    self.results.remove(i);
                    for rs in self.results.iter_mut() {
                        rs.discard_bit(i as u16);
                    }
                }
            }

            let mut new_items: Vec<_> = items
                .into_iter()
                .filter(|new_item| !self.items.iter().any(|x| x.0 == *new_item))
                .collect();

            new_items.shuffle(&mut rand::rng());
            for new_item in new_items.into_iter() {
                debug!("Adding item {:?}", new_item);

                let mut new_bitmask: BitField = vec![].into();
                new_bitmask.set_bit(self.items.len() as u16);
                self.bitmasks.push(new_bitmask);

                self.items.push((new_item, f32::NAN));

                self.results.push(vec![].into());
            }

            for i in 0..self.items.len() {
                self.bitmasks[i].fit_to_number_of_bits(self.items.len() as u16);
                self.results[i].fit_to_number_of_bits(self.items.len() as u16);
            }
        }

        self.num_rated_items = (self
            .bitmasks
            .iter()
            .fold(0, |acc, x| acc + x.get_num_ones()) as usize
            - self.bitmasks.len())
            / 2;
        self.total_ratings = (self.items.len() * self.items.len() - self.items.len()) / 2;

        self.cache_items()?;
        self.cache_bitmasks()?;
        self.cache_results()?;

        Ok(())
    }

    fn cache_items(&self) -> Result<()> {
        files::cache_items(&self.dir, &self.items)?;
        Ok(())
    }
    fn cache_bitmasks(&self) -> Result<()> {
        files::cache_bitmasks(&self.dir, &self.bitmasks)?;
        Ok(())
    }
    fn cache_results(&self) -> Result<()> {
        files::cache_results(&self.dir, &self.results)?;
        Ok(())
    }

    fn reset_bitmasks_and_results(&mut self) {
        self.bitmasks.clear();
        self.results.clear();
        for i in 0..self.items.len() {
            let mut bitmask: BitField = vec![].into();

            bitmask.set_bit(i as u16);
            bitmask.fit_to_number_of_bits(self.items.len() as u16);
            self.bitmasks.push(bitmask);

            let mut result: BitField = vec![].into();

            result.fit_to_number_of_bits(self.items.len() as u16);
            self.results.push(result);
        }
        self.num_rated_items = 0;
    }
}
