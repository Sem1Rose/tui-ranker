pub mod elo;
pub mod files;
pub mod types;

use std::str::FromStr;
use std::usize;

use crate::ranker;
use anyhow::Ok;
use log::info;
use rand::seq::{IteratorRandom, SliceRandom};
use types::BitMask;

pub static WINDOW_SIZE: usize = 3;

pub fn shuffle<T: Clone>(arr: &[T]) -> Vec<T> {
    let mut shuffled = arr.to_vec();
    shuffled.shuffle(&mut rand::rng());

    shuffled
}

#[derive(Default)]
pub struct Ranker<T> {
    items: Vec<(T, f32)>,
    bitmasks: Vec<BitMask>,
    results: Vec<BitMask>,

    window: Vec<u16>,

    item_a: usize,
    item_b: usize,
    item_a_won: bool,

    pub num_rated_items: usize,
    pub window_rated_items: usize,
}

impl<T> Ranker<T>
where
    T: FromStr + ToString + PartialEq<T> + Clone + Default,
{
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            items: ranker::files::get_cached_items()?,
            bitmasks: ranker::files::get_cached_bitmasks()?,
            results: ranker::files::get_cached_results()?,
            item_a_won: true,
            ..Default::default()
        })
    }

    pub fn init(mut self, items: Vec<T>) -> anyhow::Result<Self> {
        let cached_items = self.items;
        self.items = vec![];

        if cached_items.is_empty() {
            self.items = items.into_iter().map(|x| (x, f32::NAN)).collect();
            self.bitmasks = vec![];
            self.results = vec![];

            for i in 0..self.items.len() as u16 {
                let mut new_bitmask: BitMask = vec![].into();
                new_bitmask.set_bit(i);
                self.bitmasks.push(new_bitmask);

                let mut result: BitMask = vec![].into();
                result.fit_to_bytes(2 * self.items.len() as u16);
                self.results.push(result);
            }
            for bitmask in self.bitmasks.iter_mut() {
                bitmask.fit_to_num(self.items.len() as u16 - 1);
            }
        } else {
            for (i, cached_item) in cached_items.iter().enumerate().rev() {
                if !items.iter().any(|x| *x == cached_item.0) {
                    self.bitmasks.remove(i);
                    for bm in self.bitmasks.iter_mut() {
                        bm.discard_bit(i as u16);
                    }

                    self.results.remove(i);
                    for rs in self.results.iter_mut() {
                        // rs.discard_byte(2 * i as u16 + 1);
                        rs.discard_byte(i as u16);
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
                let mut new_bitmask: BitMask = vec![].into();
                new_bitmask.set_bit(self.items.len() as u16);
                self.bitmasks.push(new_bitmask);

                self.items.push((new_item, f32::NAN));

                let mut new_result: BitMask = vec![].into();
                new_result.fit_to_bytes(self.items.len() as u16);
                self.results.push(new_result);
            }
            for bitmask in self.bitmasks.iter_mut() {
                bitmask.fit_to_num(self.items.len() as u16 - 1);
            }

            self.num_rated_items = (self
                .bitmasks
                .iter()
                .fold(0, |acc, x| acc + x.get_num_ones())
                as usize
                - self.bitmasks.len())
                / 2
                + 1;
        }

        ranker::files::cache_items(&self.items)?;
        ranker::files::cache_bitmasks(&self.bitmasks)?;
        ranker::files::cache_results(&self.results)?;

        info!("getting new window..");
        if !self.get_new_window() {
            info!("all images has been rated, resetting..");

            for (i, bitmask) in self.bitmasks.iter_mut().enumerate() {
                *bitmask = vec![].into();

                bitmask.set_bit(i as u16);
                bitmask.fit_to_num(self.items.len() as u16 - 1);
            }
            for result in self.results.iter_mut() {
                *result = vec![].into();
                result.fit_to_bytes(self.items.len() as u16);
            }

            _ = self.get_new_window();
        }
        info!("got window {:#?}", self.window);

        self.item_a = *self.window.iter().choose(&mut rand::rng()).unwrap() as usize;

        Ok(self)
    }

    pub fn next(&mut self) -> anyhow::Result<Option<(T, T)>> {
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
                if !self.bitmasks[self.item_a].check_bit(*i) {
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
                for i in ranker::shuffle(&self.window) {
                    for j in &self.window {
                        if !self.bitmasks[i as usize].check_bit(*j) {
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
                if !self.bitmasks[self.item_b].check_bit(*i) {
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
                for i in ranker::shuffle(&self.window) {
                    for j in &self.window {
                        if !self.bitmasks[i as usize].check_bit(*j) {
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
            self.items[self.item_a].0.clone(),
            self.items[self.item_b].0.clone(),
        )))
    }

    pub fn log_result(&mut self, a_won: bool) -> anyhow::Result<()> {
        self.bitmasks[self.item_a].set_bit(self.item_b as u16);
        self.bitmasks[self.item_b].set_bit(self.item_a as u16);
        self.window_rated_items += 1;
        self.num_rated_items += 1;
        self.item_a_won = a_won;

        if a_won {
            self.results[self.item_a].inc_byte(self.item_b as u16);
        } else {
            self.results[self.item_b].inc_byte(self.item_a as u16);
        }

        ranker::files::cache_results(&self.results)?;
        ranker::files::cache_bitmasks(&self.bitmasks)?;
        Ok(())
    }

    pub fn get_total_ratings(&self) -> usize {
        (self.items.len() * self.items.len() - self.items.len()) / 2
    }

    pub fn get_item_scores(&self) -> Vec<(T, f32)> {
        let mut sorted = self
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
        self.bitmasks[item].get_num_ones() as usize - 1
    }

    pub fn get_item_index(&self, item: &T) -> usize {
        self.items.iter().position(|(x, _)| x == item).unwrap()
    }

    fn get_new_window(&mut self) -> bool {
        let mut all_done = true;
        for bitmask in &self.bitmasks {
            if bitmask.get_num_ones() < self.bitmasks.len() as u16 {
                all_done = false;
                break;
            }
        }
        if all_done {
            return false;
        }

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
        for i in shuffle(&(0..self.bitmasks.len() as u16).collect::<Vec<_>>()) {
            if self.bitmasks[i as usize].get_num_ones() < self.bitmasks.len() as u16 {
                self.window = vec![i];
                break;
            }
        }

        info!("filling the new window...");
        for i in shuffle(&(0..self.bitmasks.len() as u16).collect::<Vec<_>>()) {
            if self.window.iter().any(|x| *x == i) {
                continue;
            }

            let mut skip = false;
            for j in &self.window {
                if self.bitmasks[i as usize].check_bit(*j) {
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
            let mask = &self.bitmasks[*i as usize];
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
            if opponent_index != item as u16 && !self.bitmasks[item].check_bit(opponent_index) {
                info!("got {opponent_index}");
                return opponent_index as usize;
            }
        }

        usize::MAX
    }

    fn update_results_elos(&mut self) -> anyhow::Result<()> {
        let mut new_ratings: Vec<(&u16, (T, f32))> = vec![];
        for i in &self.window {
            let result = &mut self.results[*i as usize];
            for j in &self.window {
                if i == j {
                    continue;
                }

                let score = result.get_byte(*j);

                let mut i_rating = self.items[*i as usize].1;
                let mut j_rating = self.items[*j as usize].1;

                let i_new_rating_pos = new_ratings.iter().position(|x| x.0 == i);
                if let Some(pos) = i_new_rating_pos {
                    i_rating = new_ratings[pos].1.1;
                } else if !i_rating.is_normal() {
                    i_rating = 1000.0;
                }

                let j_new_rating_pos = new_ratings.iter().position(|x| x.0 == j);
                if let Some(pos) = j_new_rating_pos {
                    j_rating = new_ratings[pos].1.1;
                } else if !j_rating.is_normal() {
                    j_rating = 1000.0;
                }

                let new_rating = elo::calc_new_rating(i_rating, j_rating, score);
                info!("{}x{}={} {}->{}", i, j, score, i_rating, new_rating);

                if i_new_rating_pos.is_some() {
                    new_ratings[i_new_rating_pos.unwrap()].1.1 = new_rating;
                } else {
                    new_ratings.push((i, (self.items[*i as usize].0.clone(), new_rating)));
                }

                result.set_byte(*j, 0);
            }
        }

        for i in new_ratings {
            // info!("{} {}->{}", i.0, items[*i.0 as usize].1, i.1.1);
            self.items[*i.0 as usize] = i.1;
        }

        ranker::files::cache_items(&self.items)?;
        ranker::files::cache_results(&self.results)?;
        ranker::files::cache_bitmasks(&self.bitmasks)?;
        Ok(())
    }
}
