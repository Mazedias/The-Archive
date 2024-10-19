#[macro_use] extern crate rocket;

use file_manager::{file_structure::generate_file_structure, updater::{update_files, UpdateStatus}};
use rocket::{fs::{relative, FileServer}, State};
use rocket::serde::json::Json;
use rocket_dyn_templates::{Template, context};
use pulldown_cmark::{Parser, Options, html};
use serde_json::{json, Value};
use tokio::sync::RwLock;
use std::{env, fs, sync::Arc, time::Duration};

mod file_manager;

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
            content: get_markdown("messages/Updating"),
        })
    }

    let file_structure = match generate_file_structure("content/repo") {
        Ok(value) => value,
        Err(_) => json!(""),
    };

    Template::render("base", context! { files: file_structure, content: get_markdown("messages/TheArchive") })
}


#[get("/file_structure")]
fn get_file_structure() -> Json<Value> {
    let json = match generate_file_structure("content/repo") {
        Ok(value) => value,
        Err(_) => json!({ "error": "Could not generate file structure!" })
    };

    Json(json)
}


#[get("/<file>")]
fn get_markdown(file: &str) -> String {
    let path = format!("content/{}.md", file);
    let markdown = fs::read_to_string(path).unwrap_or_else(|_| "# 404\n\nFile not found.".to_string());

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);

    let parser = Parser::new_ext(&markdown, options);

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    html_output
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
        Ok(_) => true,
        Err(_) => false,
    };

    if repo_is_set {
        tokio::spawn(async move {
            update_files_periodically(update_state_clone).await;
        });
    }

    rocket::build()
        .mount("/", routes![index, get_markdown, get_file_structure])
        .mount("/static", FileServer::from(relative!("static")))
        .attach(Template::fairing())
        .manage(update_state)
}