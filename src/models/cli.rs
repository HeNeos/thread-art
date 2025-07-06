use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    pub image_path: PathBuf,

    #[arg(short, long)]
    output: Option<PathBuf>,
}
