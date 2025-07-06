use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    pub image_path: PathBuf,

    #[arg(short, long)]
    pub output: Option<PathBuf>,

    #[arg(short, long, default_value_t = 1024)]
    pub nodes: usize,

    #[arg(short, long, default_value_t = 1024)]
    pub max_lines: usize,

    #[arg(short, long, default_value_t = 1)]
    pub colors: usize,
}
