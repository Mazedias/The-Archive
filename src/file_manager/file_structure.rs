use std::{cmp::Ordering, fs::{self, DirEntry}, io::ErrorKind, path::Path};

use rocket::serde::Serialize;

#[derive(PartialEq, Debug, Serialize)]
pub enum FileEntry {
    File {
        name: String,
        path: String,
        order: Option<usize>,
    },
    Directory {
        name: String, 
        order: Option<usize>,
        children: Vec<FileEntry>,
    }
}


/// Method to generate a nested `FileEntry` structure representing the file structure starting 
/// from directory `path`.
/// 
/// # Arguments
/// 
/// * `path` - The path to the starting directory
/// 
/// # Returns
/// Result containing a vector that contains directories and files of `path` or an `std::io::Error`.
/// If `path` does not lead to a directory a `std::io::Error` of kind `NotFound` is returned.
pub fn generate_file_structure<P: AsRef<Path>>(path: P) -> Result<Vec<FileEntry>, std::io::Error> {
    let path = path.as_ref();
    
    if !path.is_dir() {
        return Err(std::io::Error::new(
            ErrorKind::NotFound,
            format!("Error reading path {}. Not a directory!", path.to_string_lossy())
        ));
    }

    // Generate FileEntry hierachy
    let file_structure: Vec<FileEntry> = walk_dir(path)?;

    Ok(file_structure)
}


/// Method to recursively walk directories and generating a nested `FileEntry` structure
/// representing found files/folders.
/// The elements in each `children` of a `FileEntry::Directory` are sorted. Listed first are 
/// all the subdirectories and then all files. 
/// 
/// Directories are sorted using [dir_comparison] while the files are sorted by simple string comparison.
/// 
/// # Arguments
/// 
/// * `path` - The path to the directory that shall be walked
/// 
/// # Returns
/// Result containing a vector that contains directories and files of `path` or an `std::io::Error`
fn walk_dir<P: AsRef<Path>>(path: P) -> Result<Vec<FileEntry>, std::io::Error> {
    // Load all entries
    let entries: Vec<DirEntry> = fs::read_dir(path)?
        .map(|res| res)
        .collect::<Result<Vec<_>, std::io::Error>>()?;

    // Seperate directories and files
    let (mut directories,mut files): (Vec<DirEntry>, Vec<DirEntry>) = entries.into_iter()
        .partition(|e| e.file_type().unwrap().is_dir());

    // Reorder directories and files
    directories.sort_by(|a, b| dir_comparison(&a, &b));
    files.sort_by(|a, b| {
        a.file_name().cmp(&b.file_name())
    });

    // Generate FileEntry children vector
    let mut file_entries: Vec<FileEntry> = Vec::new();

    // Add all directories
    for (index, dir) in directories.iter().enumerate() {
        file_entries.push(FileEntry::Directory { 
            name: dir.file_name().to_string_lossy().into_owned(), 
            order: Some(index), 
            children: walk_dir(dir.path())? 
        });
    }

    // Add all files
    for (index, file) in files.iter().enumerate() {
        file_entries.push(FileEntry::File { 
            name: file.file_name().to_string_lossy().into_owned(), 
            path: file.path().to_string_lossy().into_owned(),
            order: Some(index)
        });
    }

    Ok(file_entries)
}


/// Custom comparison for ordering directory entries regarding a numeric prefix.
/// Tries to extract numeric prefix using [extract_number_prefix] and ordering
/// accoring to it, resulting in this order:
/// - "01 DirectoryName"
/// - "2 DirectoryName"
/// - "004 DirectoryName"
/// - "Directory Name"
/// 
/// # Arguments
/// 
/// * `a` - Reference to first DirEntry
/// * `b` - Reference to second DirEntry
fn dir_comparison(a: &DirEntry, b: &DirEntry) -> Ordering {
    let a_name = a.file_name();
    let b_name = b.file_name();

    let a_str = a_name.to_str().unwrap_or("");
    let b_str = b_name.to_str().unwrap_or("");

    match (extract_number_prefix(&a_str), extract_number_prefix(&b_str)) {
        (Some(num_a), Some(num_b)) => num_a.cmp(&num_b),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => a_str.to_lowercase().cmp(&b_str.to_lowercase())
    }
}


/// Extracts a numeric prefix of a string.
/// The prefix must be seperated from the rest by whitespaces
/// 
/// # Arguments
/// 
/// * `name` - String to extract the prefix from
/// 
/// # Returns
/// Option of extracted number or *None*
fn extract_number_prefix(name: &str) -> Option<u32> {
    let parts: Vec<&str> = name.split_whitespace().collect();

    if let Some(first) = parts.first() {
        first.parse().ok()
    } else {
        None
    }
}