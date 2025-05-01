mod cli;

use cli::Cli;

use clap::Parser;

use std::path::PathBuf;

use tokio::task::JoinSet;

const CHARACTERS: [char; 1] = ['\u{200b}'];

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let args = Cli::parse();

    scan_folder(args.path).await;
}

async fn scan_folder(path: PathBuf) {
    let mut read_dir = tokio::fs::read_dir(&path).await.unwrap();
    let mut join_set = JoinSet::new();

    loop {
        let entry = read_dir.next_entry().await;
        let Ok(entry) = entry else { continue };

        if !entry.is_some() {
            break;
        }

        let entry = entry.unwrap();

        let Ok(file_type) = entry.file_type().await else {
            continue;
        };

        if file_type.is_dir() {
            let entry = entry.path().clone();
            spawn_process(&mut join_set, entry);
        } else if file_type.is_file() {
            let entry = entry.path();

            join_set.spawn(async move {
                let Ok(s) = tokio::fs::read_to_string(&entry).await else {
                    return;
                };

                let mut counter = 0;
                for c in s.chars() {
                    for zwp in CHARACTERS {
                        counter += (c == zwp) as usize;
                    }
                }

                if counter != 0 {
                    println!(
                        "found {} zero width whitespaces in {}",
                        counter,
                        entry.file_name().unwrap().to_str().unwrap()
                    );
                }
            });
        }
    }

    join_set.join_all().await;
}

// This function exists for a stupid reason
pub fn spawn_process(join_set: &mut JoinSet<()>, path: PathBuf) {
    join_set.spawn(scan_folder(path));
}
