#[macro_use] extern crate rocket;

use rocket::fs::{FileServer, relative};
use rocket_dyn_templates::{Template, context};
use pulldown_cmark::{Parser, Options, html};
use std::fs;

mod file_manager;

#[get("/")]
fn index() -> Template {
    let files = fs::read_dir("content")
        .unwrap()
        .filter_map(|entry| {
            entry.ok().and_then(|e|
                e.path().file_stem()
                    .and_then(|n| n.to_str().map(String::from))   
            )
        })
        .collect::<Vec<String>>();

    Template::render("base", context! { files: files, content: get_markdown("TheArchive") })
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


#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, get_markdown])
        .mount("/static", FileServer::from(relative!("static")))
        .attach(Template::fairing())
}