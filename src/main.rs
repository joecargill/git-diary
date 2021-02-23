use std::env;
use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;

use chrono::{DateTime, Utc};
use git2::Repository;
use git2::{Commit, ObjectType};
use git2::{Oid, Signature};

fn main() -> Result<(), git2::Error> { 

  let now: DateTime<Utc> = Utc::now();
  let stdin = io::stdin();

  let root = Path::new("/");
  env::set_current_dir(&root);

  println!("Today is {}\n", now.format("%A %d %b %Y"));
  println!("What are you working on today? (Type 'done' to finish)\n");

  let mut tasks: Vec<String> = Vec::new();
  for line in stdin.lock().lines() {
    let line = match line {
        Ok(line) => line,
        Err(err) => panic!("failed to read line: {}", err)
    };
    if line.trim() == "done" { break; }
    tasks.push(line);
  }

  let file_path = format!("Users/joec/logbook/entries/{}.md", now.format("%F")).to_lowercase();
  let path = Path::new(&file_path);
  let display = path.display();

  let mut file = match File::create(&path) {
    Err(why) => panic!("couldn't create {}: {}", display, why),
    Ok(file) => file,
  };

  let page_title = format!("{}", now.format("%A %d %b %Y"));
  writeln!(file, "# {}\n", page_title);
  for task in &mut tasks {
    writeln!(file, " - {}", task);
  };

  let repo = match Repository::open("Users/joec/logbook") {
    Ok(repo) => repo,
    Err(e) => panic!("failed to open: {}", e),
  };
  let mut index = repo.index()?;
  index.add_path(path);
  index.write()?;

  let mut commit_msg = format!("{}\n", page_title.clone());
  for task in &mut tasks {
    commit_msg = format!("{}{}", commit_msg, format!("\n - {}", task));
  };

  let file_to_commit_path = format!("entries/{}.md", now.format("%F")).to_lowercase();
  let file_to_commit = Path::new(&file_to_commit_path);
  let _ = add_and_commit(&repo, &file_to_commit, &commit_msg).expect("Couldn't add file to repo");
  
  println!("File added and committed. Ready for push.");

  Ok(())
}

fn find_last_commit(repo: &Repository) -> Result<Commit, git2::Error> {
  let obj = repo.head()?.resolve()?.peel(ObjectType::Commit)?;
  obj.into_commit().map_err(|_| git2::Error::from_str("Couldn't find commit"))
}

fn add_and_commit(repo: &Repository, path: &Path, message: &str) -> Result<Oid, git2::Error> {
    let mut index = repo.index()?;
    index.add_path(path)?;
    let oid = index.write_tree()?;
    let signature = Signature::now("joecargill", "cargill3@live.com")?;
    let parent_commit = find_last_commit(&repo)?;
    let tree = repo.find_tree(oid)?;
    repo.commit(Some("HEAD"), //  point HEAD to our new commit
                &signature, // author
                &signature, // committer
                message, // commit message
                &tree, // tree
                &[&parent_commit]) // parents
}
