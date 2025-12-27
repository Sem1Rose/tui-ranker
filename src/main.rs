mod app;
mod drawer;
mod helpers;
mod image_backend;
mod key_event_handler;
mod popups;
mod screens;
mod types;

use app::App;
use drawer::Drawer;
use key_event_handler::KeyEventHandler;

// const FRAME_RATE: f32 = 60.0;
// const TICK_RATE: f32 = 10.0;

fn main() -> anyhow::Result<()> {
    env_logger::Builder::new()
        .format_source_path(true)
        .format_timestamp_millis()
        .init();

    let mut app = App::new()?;
    app.run()?;

    ratatui::restore();

    Ok(())

    // if !PathBuf::from(".projects").exists() {
    //     File::create(".projects")?;
    // }
    // let mut projects = fs::read_to_string(".projects")
    //     .unwrap()
    //     .lines()
    //     .map(|x| {
    //         (
    //             x.split(": ").nth(0).unwrap().to_string(),
    //             x.split(": ").nth(1).unwrap().to_string(),
    //         )
    //     })
    //     .collect::<Vec<_>>();
    // if projects.is_empty() {
    //     projects.push(("test".into(), ".test".into()));
    // }

    // let mut ranker = Ranker::new()
    //     // .load_projects()?;
    //     .load_projects_from(dirs::config_dir().expect("Couldn't get user's config dir"))?;
    // if !ranker.try_select_project_by_name(&projects[0].0)? {
    //     ranker.create_project(projects[0].0.clone())?;
    // }

    // let entries = std::fs::read_dir(&projects[0].1)?;
    // let mut files = vec![];
    // for entry in entries {
    //     let dir = entry?;

    //     if dir.file_name().to_str().is_some_and(|x| {
    //         x.to_lowercase().ends_with("png")
    //             || x.to_lowercase().ends_with("jpg")
    //             || x.to_lowercase().ends_with("jpeg")
    //             || x.to_lowercase().ends_with("webp")
    //     }) {
    //         files.push(
    //             dir.path()
    //                 // .canonicalize()
    //                 // .unwrap()
    //                 .to_str()
    //                 .unwrap()
    //                 .to_string(),
    //         );
    //     }
    // }
    // files.shuffle(&mut rand::rng());

    // ranker.sync_project(files)?;

    // let mut total_ratings = ranker.get_total_ratings();
    // loop {
    //     let result = ranker.get_next();
    //     if let Err(ranker::ProjectsError::NoProjects) = result {
    //         println!("Creating a new project...");
    //         let name = input!("enter project name: ");
    //         let p = input!("enter project path: ");
    //         if p == "q" {
    //             continue;
    //         }
    //         projects.push((name.clone(), p));

    //         let mut file = File::create(".projects")?;
    //         for project in &projects {
    //             writeln!(file, "{}: {}", project.0, project.1)?;
    //         }

    //         ranker.create_project(name.clone())?;

    //         let entries =
    //             std::fs::read_dir(&projects[projects.iter().position(|x| name == x.0).unwrap()].1)?;
    //         let mut files = vec![];
    //         for entry in entries {
    //             let dir = entry?;

    //             if dir.file_name().to_str().is_some_and(|x| {
    //                 x.to_lowercase().ends_with("png")
    //                     || x.to_lowercase().ends_with("jpg")
    //                     || x.to_lowercase().ends_with("jpeg")
    //                     || x.to_lowercase().ends_with("webp")
    //             }) {
    //                 files.push(dir.path().to_str().unwrap().to_string());
    //             }
    //         }
    //         files.shuffle(&mut rand::rng());

    //         ranker.sync_project(files)?;

    //         total_ratings = ranker.get_total_ratings();

    //         continue;
    //     } else if let Err(ranker::ProjectsError::NotEnoughItems) = result {
    //         // println!(
    //         //     "Project {} contains less than two items, add more items or choose a new project",
    //         //     ranker.get_project()?.name
    //         // );
    //         for name in ranker.get_project_names() {
    //             print!("{name}\t");
    //         }
    //         let name = input!("\nenter project name: ");
    //         if !ranker.try_select_project_by_name(&name)? {
    //             let p = input!("enter project path: ");
    //             if p == "q" {
    //                 continue;
    //             }
    //             projects.push((name.clone(), p));

    //             let mut file = File::create(".projects")?;
    //             for project in &projects {
    //                 writeln!(file, "{}: {}", project.0, project.1)?;
    //             }

    //             ranker.create_project(name.clone())?;
    //         }

    //         let entries =
    //             std::fs::read_dir(&projects[projects.iter().position(|x| name == x.0).unwrap()].1)?;
    //         let mut files = vec![];
    //         for entry in entries {
    //             let dir = entry?;

    //             if dir.file_name().to_str().is_some_and(|x| {
    //                 x.to_lowercase().ends_with("png")
    //                     || x.to_lowercase().ends_with("jpg")
    //                     || x.to_lowercase().ends_with("jpeg")
    //                     || x.to_lowercase().ends_with("webp")
    //             }) {
    //                 files.push(dir.path().to_str().unwrap().to_string());
    //             }
    //         }
    //         files.shuffle(&mut rand::rng());

    //         ranker.sync_project(files)?;

    //         total_ratings = ranker.get_total_ratings();

    //         continue;
    //     }

    //     info!(
    //         "total: {}/{total_ratings}",
    //         ranker.get_project()?.num_rated_items
    //     );
    //     info!(
    //         "window: {}/{}",
    //         ranker.window_rated_items,
    //         ranker.get_num_ratings_to_end()
    //     );
    //     let next = result?;
    //     if next.is_none() {
    //         info!("all images has been rated, quitting..");
    //         break;
    //     }
    //     let (item_a, item_b) = next.unwrap();

    //     let a = String::from_utf8(
    //         std::process::Command::new("chafa")
    //             .args(["--polite", "on", "-s", "64x24", &item_a])
    //             .output()?
    //             .stdout,
    //     )?;
    //     let b = String::from_utf8(
    //         std::process::Command::new("chafa")
    //             .args(["--polite", "on", "-s", "64x24", &item_b])
    //             .output()?
    //             .stdout,
    //     )?;

    //     print!("a{a}");
    //     print!("b{b}");

    //     let mut a_won = false;
    //     let mut quit = false;
    //     let mut c = false;
    //     let mut d = false;
    //     let mut choice = input!("choose (a-b): ");
    //     loop {
    //         if choice.trim() == "a" {
    //             a_won = true;
    //             break;
    //         } else if choice.trim() == "b" {
    //             a_won = false;
    //             break;
    //         } else if choice.trim() == "q" {
    //             quit = true;
    //             break;
    //         } else if choice.trim() == "c" {
    //             c = true;
    //             break;
    //         } else if choice.trim() == "d" {
    //             d = true;
    //             break;
    //         }
    //         choice = input!("choose (a-b): ");
    //     }
    //     if quit {
    //         break;
    //     }
    //     if c {
    //         for name in ranker.get_project_names() {
    //             print!("{name}\t");
    //         }
    //         let name = input!("\nenter project name: ");
    //         if !ranker.try_select_project_by_name(&name)? {
    //             let p = input!("enter project path: ");
    //             if p == "q" {
    //                 continue;
    //             }
    //             projects.push((name.clone(), p));

    //             let mut file = File::create(".projects")?;
    //             for project in &projects {
    //                 writeln!(file, "{}: {}", project.0, project.1)?;
    //             }

    //             ranker.create_project(name.clone())?;
    //         }

    //         let entries =
    //             std::fs::read_dir(&projects[projects.iter().position(|x| name == x.0).unwrap()].1)?;
    //         let mut files = vec![];
    //         for entry in entries {
    //             let dir = entry?;

    //             if dir.file_name().to_str().is_some_and(|x| {
    //                 x.to_lowercase().ends_with("png")
    //                     || x.to_lowercase().ends_with("jpg")
    //                     || x.to_lowercase().ends_with("jpeg")
    //                     || x.to_lowercase().ends_with("webp")
    //             }) {
    //                 files.push(dir.path().to_str().unwrap().to_string());
    //             }
    //         }
    //         files.shuffle(&mut rand::rng());

    //         ranker.sync_project(files)?;

    //         total_ratings = ranker.get_total_ratings();

    //         continue;
    //     }
    //     if d {
    //         for name in ranker.get_project_names() {
    //             print!("{name}\t");
    //         }
    //         let name = input!("\nenter project name: ");
    //         if !ranker.try_delete_project_by_name(&name)? {
    //             println!("Project {name} not found!");
    //         } else {
    //             projects.remove(projects.iter().position(|x| x.0 == name).unwrap());

    //             let mut file = File::create(".projects")?;
    //             for project in &projects {
    //                 writeln!(file, "{}: {}", project.0, project.1)?;
    //             }
    //         }

    //         continue;
    //     }

    //     ranker.log_result(a_won)?;
    // }

    // let mut file = File::create(".projects")?;
    // for project in &projects {
    //     writeln!(file, "{}: {}", project.0, project.1)?;
    // }

    // for (i, item) in ranker.get_item_scores().iter().enumerate().rev() {
    //     let x = String::from_utf8(
    //         std::process::Command::new("chafa")
    //             .args(["--polite", "on", "-s", "48x12", &item.0])
    //             .output()?
    //             .stdout,
    //     )?;
    //     print!(
    //         "{}: {} {}\n{x}",
    //         i + 1,
    //         item.1,
    //         ranker.get_item_num_played_games(ranker.get_item_index(&item.0)),
    //     );
    // }

    // Ok(())
}
