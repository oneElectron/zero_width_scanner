mod cli;

use cli::Cli;

use clap::Parser;

use std::path::{Path, PathBuf};

use lopdf::Document;
use tokio::task::JoinSet;

const CHARACTERS: [char; 3] = ['\u{200B}', '\u{200C}', '\u{200D}'];

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let args = Cli::parse();

    scan_folder(args.path).await;
}

/// AI-Generated
fn scan_pdf_for_zero_width_whitespaces<P: AsRef<Path>>(file_path: &P) -> Result<Vec<u32>, String> {
    // Load the PDF document
    let doc = Document::load(file_path).map_err(|e| e.to_string())?;

    // List of zero-width whitespace characters
    let zero_width_chars = ['\u{200B}', '\u{200C}', '\u{200D}'];

    let mut pages_with_matches = Vec::new();

    // Iterate over all pages in the PDF
    for (page_number, (page_id, _)) in doc.get_pages() {
        if let Ok(page_content) = doc.extract_text(&[page_id]) {
            // Check if any zero-width character exists in the page content
            if zero_width_chars.iter().any(|&c| page_content.contains(c)) {
                pages_with_matches.push(page_number);
            }
        }
    }

    Ok(pages_with_matches)
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

            let _ = tokio::spawn(async move {
                let extension = entry.extension().map(|x| x.to_str().unwrap());
                if extension.is_some() && extension.unwrap() == "pdf" {
                    let s = scan_pdf_for_zero_width_whitespaces(&entry.as_path());

                    let Err(_) = s else {
                        return;
                    };

                    let s = s.unwrap();
                    if s.len() != 0 {
                        for e in s.iter() {
                            println!("{}, whitespace found at {}", entry.as_path().display(), e);
                        }
                    }

                    return;
                }
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
            })
            .await;
        }
    }

    join_set.join_all().await;
}

// This function exists for a stupid reason
pub fn spawn_process(join_set: &mut JoinSet<()>, path: PathBuf) {
    join_set.spawn(scan_folder(path));
}
