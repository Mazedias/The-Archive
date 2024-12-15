#[macro_use] extern crate rocket;

use file_manager::{file_structure::generate_file_structure, updater::{update_files, UpdateStatus}};
use html_generator::html_generator::{build_html_structure, markdown_to_html};
use rocket::{fs::{relative, FileServer}, State};
use rocket_dyn_templates::{Template, context};
use tokio::sync::RwLock;
use std::{env, path::PathBuf, sync::Arc, time::Duration};

mod file_manager;
mod html_generator;

#[derive(Clone)]
struct UpdateState {
    is_updating: bool,
}

impl UpdateState {
    fn new() -> Self {
        UpdateState { is_updating: false }
    }
}

type SharedUpdateState = Arc<RwLock<UpdateState>>;


#[get("/")]
async fn index(update_state: &State<SharedUpdateState>) -> Template {
    let is_updating = update_state.read().await.is_updating;

    if is_updating {
        return Template::render("base", context! {
            content: markdown_to_html("content/messages/Updating"),
        })
    }

    // Try to retrieve file structre, if not possible return an error screen
    let file_structure = match generate_file_structure("content/repo") {
        Ok(value) => value,
        Err(_) => return Template::render("base", context! {content: "Error occured while loading files!".to_string()}),
    };

    // Translate file_structure to vector of Json objects (serde_json::Value) to be able to parse it into context macro
    let file_structure: String = file_structure
        .iter()
        .map(|entry| build_html_structure(&entry))
        .collect::<Vec<String>>()
        .join("\n");

    Template::render("base", context! { table_of_content: file_structure, content: markdown_to_html("content/messages/TheArchive") })
}


#[get("/loadFile/<file..>")]
fn get_markdown(file: PathBuf) -> String {
    let file_path = file.to_str().unwrap_or_default();
    markdown_to_html(file_path)
}


async fn update_files_periodically(update_state: SharedUpdateState) {
    loop {
        tokio::time::sleep(Duration::from_secs(3600)).await;

        {
            let mut state = update_state.write().await;
            state.is_updating = true;
        }

        match update_files() {
            UpdateStatus::SuccessfullUpdated => println!("Files updated successfully"),
            UpdateStatus::NoUpdateNecessary => println!("Checked for update but was not neccessary"),
            UpdateStatus::UpdateNotPossible => println!("Tried to update but couldn't"),
        }

        {
            let mut state = update_state.write().await;
            state.is_updating = false;
        }
    }
}


#[launch]
fn rocket() -> _ {
    let update_state = Arc::new(RwLock::new(UpdateState::new()));
    let update_state_clone = update_state.clone();

    let repo_is_set = match env::var("GIT_REPO_URL") {
        Ok(value) if !value.is_empty() => true,
        Ok(_) => false,
        Err(_) => false,
    };

    if repo_is_set {
        tokio::spawn(async move {
            update_files_periodically(update_state_clone).await;
        });
    }

    rocket::build()
        .mount("/", routes![index, get_markdown])
        .mount("/static", FileServer::from(relative!("static")))
        .attach(Template::fairing())
        .manage(update_state)
}