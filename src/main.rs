mod cache_db;
use cache_db::Cache;
mod datatype;
mod logger;
use logger::*;
use serde::Deserialize;
use std::process;
mod parser;
use parser::*;
mod stats;
use clap::Parser as ClapParser;
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

    #[arg(
        short,
        long,
        value_enum, default_value_t = LogLevel::Warn,
        help = "If not set, we will use warning leve. The options are: error, warn, info, debug."
    )]
    log_level: LogLevel,
}

#[derive(Deserialize, Debug)]
struct Config {
    reg_pattern_list: Vec<String>,
    finished_reg_pattern_list: Vec<String>,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let our_program_name = "watching_log_parser";

    let file_path = PathBuf::from(args.filename);

    let config_path = if let Some(specified_config_path) = args.config_path {
        PathBuf::from(specified_config_path)
    } else if let Some(config_dir) = dirs_2::config_dir() {
        config_dir.join(our_program_name).join("config")
    } else {
        eprintln!(
            "We cannot find a config file, you can specify one with --config_path or put one on system-level config path."
        );
        process::exit(1);
    };

    let cache_path = if let Some(cache_dir) = dirs_2::cache_dir() {
        cache_dir.join(our_program_name).join("cache.db")
    } else {
        eprintln!("We cannot find the system-level cache dir!");
        process::exit(1);
    };
    if !cache_path.exists() {
        fs::create_dir_all(&cache_path)?;
    }

    logger_init(args.log_level);

    let config_json_data = fs::read_to_string(config_path).unwrap();
    let config: Config = serde_yaml::from_str(&config_json_data)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let contents = fs::read_to_string(file_path).unwrap();

    let parser = Parser::new(
        config.reg_pattern_list,
        config.finished_reg_pattern_list,
        Cache::new(cache_path.to_str().unwrap()).ok(),
    );

    let lines: Vec<String> = contents.lines().map(|line| line.to_string()).collect();

    let metadata_list = parser.parse_metadata(&lines);
    let stats = Stats::new(metadata_list);
    let unfinished_wathcing_list = stats.stats_unfinished();
    for unfinished_watching in unfinished_wathcing_list {
        println!("name: {}", unfinished_watching.name);
    }

    Ok(())
}
