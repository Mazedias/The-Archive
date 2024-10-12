use std::env;
use std::path::Path;

use git2::Repository;

pub enum UpdateStatus {
    NoUpdateNecessary,
    SuccessfullUpdated,
    UpdateNotPossible
}


pub fn update_files() -> UpdateStatus {
    // Check for repository URL
    let git_url = match env::var("GIT_REPO_URL") {
        Ok(value) if !value.is_empty() => value,
        Ok(_) => {
            println!("GIT_REPO_URL is empty! No repository linked, no update of files possible.");
            return UpdateStatus::UpdateNotPossible;
        }
        Err(_) => {
            println!("GIT_REPO_URL not found! No repository linked, no update of files possible.");
            return UpdateStatus::UpdateNotPossible;
        }
    };


    let repository_path = Path::new("content");

    // Check if the repostory already exists, if not clone it
    let repostory = match Repository::open(repository_path) {
        Ok(repo) => repo,
        Err(_) => {
            match Repository::clone(&git_url, repository_path) {
                Ok(repo) => {
                    println!("Repository cloned.");
                    repo
                }
                Err(e) => {
                    println!("Repository couldn't be cloned! Error: {}", e);
                    return UpdateStatus::UpdateNotPossible;
                }
            }
        }
    };

    // TODO: Update files 

    UpdateStatus::SuccessfullUpdated
}