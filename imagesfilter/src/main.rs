use std::path::PathBuf;
use clap::Parser;
use walkdir::{WalkDir};
use indicatif::{ProgressBar, ProgressStyle};

#[derive(Parser, Debug)]
#[command(about, long_about = None)]
struct Args {
    #[arg(short, long, value_name = "FOLDER")]
    path: PathBuf,

    #[arg(short, long, value_name = "FILE")]
    out: PathBuf,
}

fn main() {
    let args = Args::parse();

    let mut images = vec![];
    let mut jsons = vec![];
    let files = WalkDir::new(args.path).into_iter().filter_map(|e| e.ok()).filter(|e| e.metadata().map(|e| !e.is_dir()).unwrap_or(false)).collect::<Vec<_>>();
    let files_pb = ProgressBar::new(files.len() as u64);
    files_pb.set_style(ProgressStyle::with_template("{prefix:.bold.dim} {spinner:.green} [{elapsed_precise}] {human_pos}/{human_len} (eta {eta}) {wide_msg}")
        .unwrap());
    files_pb.set_prefix("[1/2]");

    for (i, e) in files.into_iter().enumerate() {
        let file_path = e.path().to_str().map(|e| e.to_owned()).unwrap_or_else(|| panic!("File name contains invalid characters, at {:?}", e.path()));
        files_pb.set_message(file_path);
        match e.path().extension().and_then(|e| e.to_str()) {
            Some("json") => {
                let content = std::fs::read_to_string(e.path()).unwrap_or_else(|_| panic!("Failed to read file at {:?}", e.path()));
                jsons.push((e.path().to_owned(), content.to_lowercase()));
            }
            Some("png" | "jpeg" | "jpg") => {
                let file_name = e.path().file_name().and_then(|e| e.to_str()).unwrap_or_else(|| panic!("Failed to fetch name of file at {:?}", e.path()));
                images.push(file_name.to_lowercase())
            }
            _ => {}
        }
        files_pb.set_position(i as u64 + 1);
    }
    files_pb.abandon();

    let pb = ProgressBar::new(images.len() as u64);
    pb.set_style(ProgressStyle::with_template("{prefix:.bold.dim} {spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {human_pos}/{human_len} (eta {eta}) {msg:32!}")
        .unwrap()
        .progress_chars("#>-"));
    pb.set_prefix("[2/2]");

    let data = images.into_iter().enumerate().map(|(i, name)| {
        pb.set_message(name.clone());
        let used = jsons.iter().filter_map(|data| {
            if data.1.contains(&name) {
                data.0.to_str()
            } else { None }
        }).collect::<Vec<_>>().join(", ");
        pb.set_position(i as u64);
        if !used.is_empty() { format!("Image {} is used in files: {}", name, used) } else { format!("Image {} is unused", name) }
    }).collect::<Vec<String>>().join("\n");

    pb.abandon();

    println!("Writing file...");
    std::fs::write(args.out, data).expect("Failed to write to output file");
    println!("Done!");
}