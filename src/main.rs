mod cache_db;
use cache_db::Cache;
mod datatype;
mod logger;
use logger::*;
use serde::Deserialize;
use std::process;
mod parser;
mod parser_task_manager;
use parser::*;
mod stats;
use clap::Parser as ClapParser;
use clap::ValueEnum;
use stats::*;
use std::path::PathBuf;
use std::{fs, io};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, ValueEnum)]
pub enum Mode {
    UnFinished,
    Query,
    All,
}

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

    #[arg(
        short,
        long,
        value_enum,
        default_value_t = Mode::UnFinished,
        help = "We have three mode right now,\n\tunfinished(default): list all unifhished watching\n\tquery: list all matching watching with give query name\n\tall: list all watching.\n")]
    mode: Mode,

    #[arg(short, long, required_if_eq("mode", "query"))]
    query_name: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Config {
    reg_pattern_list: Vec<String>,
    finished_reg_pattern_list: Vec<String>,
    max_thread_num: Option<usize>,
    min_task_num_per_thread: Option<usize>,
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
        let our_cache_dir = cache_dir.join(our_program_name);
        if !our_cache_dir.exists() {
            fs::create_dir_all(&our_cache_dir)?;
        }
        our_cache_dir.join("cache.db")
    } else {
        eprintln!("We cannot find the system-level cache dir!");
        process::exit(1);
    };

    logger_init(args.log_level);

    let config_json_data = fs::read_to_string(config_path).unwrap();
    let config: Config = serde_yaml::from_str(&config_json_data)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let contents = fs::read_to_string(file_path).unwrap();

    let mut parser = Parser::new(
        config.reg_pattern_list,
        config.finished_reg_pattern_list,
        Cache::new(cache_path.to_str().unwrap()).ok(),
        config.max_thread_num.unwrap_or(1),
        config.min_task_num_per_thread.unwrap_or(1),
    );

    let lines: Vec<String> = contents.lines().map(|line| line.to_string()).collect();

    let metadata_list = parser.parse_metadata(&lines);
    let stats = Stats::new(metadata_list);
    if args.mode == Mode::UnFinished {
        let unfinished_wathcing_list = stats.stats_unfinished();
        for unfinished_watching in unfinished_wathcing_list {
            if unfinished_watching.season.is_some() {
                println!(
                    "{} season {}",
                    unfinished_watching.name,
                    unfinished_watching.season.unwrap()
                );
            } else {
                println!("{}", unfinished_watching.name);
            }
        }
    } else if args.mode == Mode::All {
        let all_wathcing_list = stats.stats_all();
        for watching in all_wathcing_list {
            if watching.season.is_some() {
                println!(
                    "{} season {} - {}",
                    watching.name,
                    watching.season.unwrap(),
                    if watching.b_finished {
                        "finished"
                    } else {
                        "unfinished"
                    }
                );
            } else {
                println!(
                    "{} - {}",
                    watching.name,
                    if watching.b_finished {
                        "finished"
                    } else {
                        "unfinished"
                    }
                );
            }
        }
    } else if args.mode == Mode::Query {
        let query_name = args.query_name.unwrap();
        let matching_watching_list = stats.query_by_name(&query_name);
        if matching_watching_list.len() > 1 {
            println!(
                "Found {} matching records for {}:",
                matching_watching_list.len(),
                query_name
            );
            for watching in matching_watching_list {
                if watching.season.is_some() {
                    println!(
                        "{} season {} - {}",
                        watching.name,
                        watching.season.unwrap(),
                        if watching.b_finished {
                            "finished"
                        } else {
                            "unfinished"
                        }
                    );
                } else {
                    println!(
                        "{} - {}",
                        watching.name,
                        if watching.b_finished {
                            "finished"
                        } else {
                            "unfinished"
                        }
                    );
                }
            }
        } else {
            println!("No record found for {}", query_name);
        }
    }

    Ok(())
}
