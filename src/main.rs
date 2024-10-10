#[macro_use] extern crate rocket;

use rocket::fs::{FileServer, relative};
use rocket_dyn_templates::{Template, context};
use pulldown_cmark::{Parser, Options, html};
use std::fs;


#[get("/<file>")]
fn render_markdown(file: &str) -> Template {
    let path = format!("content/{}.md", file);
    let markdown = fs::read_to_string(path).unwrap_or_else(|_| "# 404\n\nFile not found.".to_string());

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);

    let parser = Parser::new_ext(&markdown, options);

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    Template::render("base", context! {content: html_output})
}

#[launch]
fn rocket() -> _ {  
    rocket::build()
        .mount("/", routes![render_markdown])
        .mount("/static", FileServer::from(relative!("static")))
        .attach(Template::fairing())
}