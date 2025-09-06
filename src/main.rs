mod datatype;
use serde::Deserialize;
use std::process;
mod parse_identity;
mod stats;
use clap::Parser as ClapParser;
use dirs_2;
use parse_identity::*;
use stats::*;
use std::path::PathBuf;
use std::{fs, io};

#[derive(ClapParser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    // config dir
    #[arg(
        short,
        long,
        help = "If not set, we will use your system's config path"
    )]
    config_path: Option<String>,
    // file
    #[arg(short, long)]
    filename: String,
}

#[derive(Deserialize, Debug)]
struct Config {
    reg_pattern_list: Vec<String>,
    finished_reg_pattern_list: Vec<String>,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let our_program_name = "watching_log_parser";

    let mut config_path = PathBuf::new();
    let mut cache_path = PathBuf::new();
    let mut file_path = PathBuf::from(args.filename);

    if let Some(specified_config_path) = args.config_path {
        config_path = PathBuf::from(specified_config_path);
    } else if let Some(config_dir) = dirs_2::config_dir() {
        config_path = config_dir.join(our_program_name).join("config.json");
    } else {
        eprintln!(
            "We cannot find a config file, you can specify one with --config_path or put one on system-level config path."
        );
        process::exit(1);
    }
    if let Some(cache_dir) = dirs_2::cache_dir() {
        cache_path = cache_dir.join(our_program_name).join("cache.db");
    }

    let config_json_data = fs::read_to_string(config_path).unwrap();
    let config: Config = serde_json::from_str(&config_json_data)?;
    let contents = fs::read_to_string(file_path).unwrap();

    let parser = Parser::new(config.reg_pattern_list, config.finished_reg_pattern_list);

    let lines: Vec<String> = contents.lines().map(|line| line.to_string()).collect();

    let metadata_list = parser.parse_metadata(&lines);
    let stats = Stats::new(metadata_list);
    let unfinished_wathcing_list = stats.stats_unfinished();
    for unfinished_watching in unfinished_wathcing_list {
        println!("name: {}", unfinished_watching.name);
    }

    Ok(())
}
