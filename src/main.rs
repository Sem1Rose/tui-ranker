mod app;
mod ranker;
mod types;

use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use log::info;
use prompted::input;
use rand::seq::SliceRandom;
// use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent};
// use std::time::{Duration, Instant};
// use types::Term;

use crate::ranker::Ranker;

// const FRAME_RATE: f32 = 60.0;
// // const TICK_RATE: f32 = 10.0;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    // let terminal: Term = ratatui::init();
    // // terminal.hide_cursor()?;

    // run(terminal)?;

    // ratatui::restore();

    if !PathBuf::from(".projects").exists() {
        File::create(".projects")?;
    }
    let mut projects = fs::read_to_string(".projects")
        .unwrap()
        .lines()
        .map(|x| {
            (
                x.split(": ").nth(0).unwrap().to_string(),
                x.split(": ").nth(1).unwrap().to_string(),
            )
        })
        .collect::<Vec<_>>();
    if projects.is_empty() {
        projects.push(("test".into(), ".test".into()));
    }

    let mut ranker = Ranker::new(None, "test".into())?;
    // let mut ranker = Ranker::default(); // simpler
    ranker.try_select_project_by_name(&projects[0].0)?;

    let entries = std::fs::read_dir(&projects[0].1)?;
    let mut files = vec![];
    for entry in entries {
        let dir = entry?;

        if dir
            .file_name()
            .to_str()
            .is_some_and(|x| x.to_lowercase().ends_with("png"))
        {
            files.push(
                dir.path()
                    // .canonicalize()
                    // .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string(),
            );
        }
    }
    files.shuffle(&mut rand::rng());

    ranker.sync_project(files)?;

    let mut total_ratings = ranker.get_total_ratings();
    loop {
        info!(
            "total: {}/{total_ratings}",
            ranker.get_project().num_rated_items
        );
        info!(
            "window: {}/{}",
            ranker.window_rated_items,
            (1..ranker::WINDOW_SIZE).sum::<usize>()
        );

        let next = ranker.next()?;
        if next.is_none() {
            info!("all images has been rated, quitting..");
            break;
        }
        let (item_a, item_b) = next.unwrap();

        let x = String::from_utf8(
            std::process::Command::new("chafa")
                .args(["--polite", "on", "-s", "64x24", &item_a])
                .output()?
                .stdout,
        )?;
        print!("a{x}");

        let x = String::from_utf8(
            std::process::Command::new("chafa")
                .args(["--polite", "on", "-s", "64x24", &item_b])
                .output()?
                .stdout,
        )?;
        print!("b{x}");

        let mut a_won = false;
        let mut quit = false;
        let mut c = false;
        let mut choice = input!("choose (a-b): ");
        loop {
            if choice.trim() == "a" {
                a_won = true;
                break;
            } else if choice.trim() == "b" {
                a_won = false;
                break;
            } else if choice.trim() == "q" {
                quit = true;
                break;
            } else if choice.trim() == "c" {
                c = true;
                break;
            }
            choice = input!("choose (a-b): ");
        }
        if quit {
            break;
        }
        if c {
            for name in ranker.get_project_names() {
                print!("{name}\t");
            }
            let name = input!("\nenter project name: ");
            let id = ranker.try_find_project(&name);
            if let Some(id) = id {
                ranker.select_project(id)?;
            } else {
                let p = input!("enter project path: ");
                if p == "q" {
                    continue;
                }
                projects.push((name.clone(), p));

                let mut file = File::create(".projects")?;
                for project in &projects {
                    writeln!(file, "{}: {}", project.0, project.1)?;
                }

                ranker.create_project(name.clone())?;
            }

            let entries =
                std::fs::read_dir(&projects[projects.iter().position(|x| name == x.0).unwrap()].1)?;
            let mut files = vec![];
            for entry in entries {
                let dir = entry?;

                if dir.file_name().to_str().is_some_and(|x| {
                    x.to_lowercase().ends_with("png")
                        || x.to_lowercase().ends_with("jpg")
                        || x.to_lowercase().ends_with("jpeg")
                        || x.to_lowercase().ends_with("webp")
                }) {
                    files.push(dir.path().to_str().unwrap().to_string());
                }
            }
            files.shuffle(&mut rand::rng());

            ranker.sync_project(files)?;

            total_ratings = ranker.get_total_ratings();

            continue;
        }

        ranker.log_result(a_won)?;
    }

    for (i, item) in ranker.get_item_scores().iter().enumerate().rev() {
        let x = String::from_utf8(
            std::process::Command::new("chafa")
                .args(["--polite", "on", "-s", "48x12", &item.0])
                .output()?
                .stdout,
        )?;
        print!(
            "{}: {} {}\n{x}",
            i + 1,
            item.1,
            ranker.get_item_num_played_games(ranker.get_item_index(&item.0)),
        );
    }

    Ok(())
}

// pub fn run(mut terminal: Term) -> anyhow::Result<()> {
//     let frame_time = Duration::from_secs_f32(1.0 / FRAME_RATE);
//     // let frames_per_tick = (FRAME_RATE / TICK_RATE).floor() as u32;
//     let mut last_frame = Instant::now();
//     // let mut tick_counter = 0;

//     loop {
//         terminal.draw(|frame| {}).map(|_| ())?;

//         let timeout = frame_time
//             .checked_sub(last_frame.elapsed())
//             .unwrap_or_else(|| Duration::from_secs(0));

//         if event::poll(timeout)? {
//             if let Ok(event) = event::read() {
//                 if let Event::Key(KeyEvent {
//                     code: KeyCode::Char('q'),
//                     modifiers: _,
//                     kind: _,
//                     state: _,
//                 }) = event
//                 {
//                     return Ok(());
//                 } else if let Event::Key(KeyEvent {
//                     code: KeyCode::Char('p'),
//                     modifiers: _,
//                     kind: _,
//                     state: _,
//                 }) = event
//                 {
//                     panic!("PANIC");
//                 }
//             }
//         }

//         // if self.can_quit() {
//         //     return Ok(());
//         // }

//         // if last_frame.elapsed() >= frame_time {
//         //     last_frame = std::time::Instant::now();

//         //     tick_counter += 1;
//         //     if tick_counter >= frames_per_tick {
//         //         tick_counter = 0;
//         //     }
//         // }
//     }
// }
