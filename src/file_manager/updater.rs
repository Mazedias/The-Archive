use std::{env, fmt::format};
use std::path::Path;

use git2::{FetchOptions, Repository};

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
    let repository = match Repository::open(repository_path) {
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

    let mut fetch_options = FetchOptions::new();

    // Update files
    match repository.find_remote("origin") {
        // Remote repositroy found
        Ok(mut remote) => {

            // Preform fetch operation
            match remote.fetch(&["main"], Some(&mut fetch_options), None) {
                Ok(_) => {
                    println!("Fetch completed successfully.");

                    let fetch_head = repository.find_reference("FETCH_HEAD").unwrap();
                    let fetch_commit = repository.reference_to_annotated_commit(&fetch_head).unwrap();
                    let analysis = repository.merge_analysis(&[&fetch_commit]).unwrap();

                    // Check if merge is neccessary
                    if analysis.0.is_up_to_date() {
                        println!("Repository is up to date.");
                        return UpdateStatus::NoUpdateNecessary;
                    } else if analysis.0.is_fast_forward() {
                        let refname = format!("refs/heads/{}", "main");
                        let mut reference = repository.find_reference(&refname).unwrap();
                        reference.set_target(fetch_commit.id(), "Fast-forward").unwrap();
                        repository.set_head(&refname).unwrap();
                        repository.checkout_head(Some(git2::build::CheckoutBuilder::default().force())).unwrap();
                        
                        println!("Fast-forward merge completed.");
                        return UpdateStatus::SuccessfullUpdated;
                    } else {
                        println!("No-Fast-Forward merge required. Not implemented yet!");
                        return UpdateStatus::UpdateNotPossible;
                    }
                }

                // Fetch failed
                Err(e) => {
                    println!("Failed to fetch! Error: {}", e);
                    return UpdateStatus::UpdateNotPossible;
                }
            }
        }

        // No remote repository found
        Err(e) => {
            println!("Failed to find remote 'origin'! Error: {}", e);
            return UpdateStatus::UpdateNotPossible
        }
    };
}