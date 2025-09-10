use super::cache_db::Cache;
use super::datatype::Metadata;
use super::parser_task_manager::*;
use super::{log_debug, log_error};
use chinese_number::from_chinese_to_u16;
use chrono::{NaiveDateTime, NaiveTime};
use regex::Regex;
use xxhash_rust::xxh3;

type DefaultParserCallback = fn(&str, u32, u64, &Vec<String>, &Vec<String>) -> Option<Metadata>;

pub struct Parser {
    // reg pattern should follow:
    //  - supoport named group
    //  - group name follows the Metadata structutre's field
    //  - sub set should have higher priority
    pub reg_pattern_list: Vec<String>,
    // This is another pattern, it will run this against each line to determine if this entry
    // represent the watching is finished or not.
    pub finished_reg_pattern_list: Vec<String>,

    pub cache: Option<Cache>,

    task_manager: ParserTaskManager<DefaultParserCallback>,
}

fn parse_number(number_str: &str) -> Option<u16> {
    match number_str.parse::<u16>() {
        Ok(number) => Some(number),
        Err(_) => from_chinese_to_u16(number_str).ok(),
    }
}

fn parse_datetime(datetime_str: &str) -> Option<NaiveDateTime> {
    NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%d %H:%M").ok()
}

fn parse_time(time_str: &str) -> Option<NaiveTime> {
    let parts: Vec<&str> = time_str.split(":").collect();
    if parts.len() == 3 {
        let hours = parse_number(parts[0]);
        let minutes = parse_number(parts[1]);
        let seconds = parse_number(parts[2]);
        if let (Some(h), Some(m), Some(s)) = (hours, minutes, seconds) {
            return NaiveTime::from_hms_opt(h.into(), m.into(), s.into());
        }
    } else if parts.len() == 2 {
        let minutes = parse_number(parts[0]);
        let seconds = parse_number(parts[1]);
        if let (Some(m), Some(s)) = (minutes, seconds) {
            return NaiveTime::from_hms_opt(0, m.into(), s.into());
        }
    }
    None
}

impl Parser {
    pub fn new(
        reg_pattern_list: Vec<String>,
        finished_reg_pattern_list: Vec<String>,
        cache: Option<Cache>,
        max_thread_num: usize,
        min_task_per_thread: usize,
    ) -> Self {
        Parser {
            reg_pattern_list,
            finished_reg_pattern_list,
            cache,
            task_manager: ParserTaskManager::new(max_thread_num, min_task_per_thread),
        }
    }

    pub fn parse_metadata(&mut self, lines: &Vec<String>) -> Vec<Metadata> {
        let mut result = Vec::<Metadata>::new();

        let reg_pool_string = [
            &self.reg_pattern_list[..],
            &self.finished_reg_pattern_list[..],
        ]
        .concat()
        .join("###")
        .to_string();

        let mut index: u32 = 0;
        for line in lines {
            let hash_value = xxh3::xxh3_64((line.clone() + reg_pool_string.as_str()).as_bytes());
            let metadata = if let Some(cache) = &self.cache {
                cache.query_cache(hash_value)
            } else {
                None
            };
            if let Some(mut m) = metadata {
                m.index = index;
                result.push(m);
                continue;
            }

            self.task_manager.add_task(ParserTask {
                index,
                hash_value,
                line: line.clone(),
                reg_pattern_list: self.reg_pattern_list.clone(),
                finished_reg_pattern_list: self.finished_reg_pattern_list.clone(),
                callback: move |line,
                index,
                hash_value,
                reg_pattern_list,
                finished_reg_pattern_list| {
                    for reg in reg_pattern_list {
                        let re = Regex::new(reg).unwrap();
                        if re.is_match(line) {
                            if let Some(caps) = re.captures(line) {
                                let name = String::from(caps.name("name").unwrap().as_str());

                                // if name.contains("") {
                                //     println!("{}", reg);
                                // }

                                let mut b_finished = false;
                                let mut matched_finished_reg_pattern: Option<String> = None;
                                for finished_reg_pattern in finished_reg_pattern_list {
                                    let finished_re = Regex::new(finished_reg_pattern).unwrap();
                                    if finished_re.is_match(line) {
                                        b_finished = true;
                                        matched_finished_reg_pattern = Some(finished_reg_pattern.to_string().clone());
                                        break;
                                    }
                                }

                                let episode: Option<u16> =
                                caps.name("episode").and_then(|s| parse_number(s.as_str()));
                                let time_at_episode: Option<NaiveTime> = caps
                                    .name("time_at_episode")
                                    .and_then(|s| parse_time(s.as_str()));
                                let season: Option<u16> =
                                caps.name("season").and_then(|s| parse_number(s.as_str()));
                                let logged_time: Option<NaiveDateTime> = caps
                                    .name("logged_time")
                                    .and_then(|s| parse_datetime(s.as_str()));
                                let note: Option<String> =
                                caps.name("note").map(|m| String::from(m.as_str()));
                                log_debug!(
                                    "hash_value: {}, name: {}, b_finished: {}, season: {:?}, episode: {:?}, time_at_episode: {:?}, logged_time: {:?}, note: {:?}, raw: {}, reg: {}",
                                    hash_value,
                                    name,
                                    b_finished.to_string(),
                                    season,
                                    episode,
                                    time_at_episode,
                                    logged_time,
                                    note,
                                    line,
                                    reg
                                );
                                return Some(Metadata {
                                    index,
                                    id: hash_value,
                                    name,
                                    b_finished,
                                    episode,
                                    time_at_episode,
                                    season,
                                    logged_time,
                                    note,
                                    raw_line: line.to_string().clone(),
                                    reg_pattern_matched: reg.to_string().clone(),
                                    finished_reg_pattern_matched: matched_finished_reg_pattern,
                                });
                            }
                        }
                    }

                    log_error! {"This line cannot match any regex patterns:\n{}", line};

                    None
                },
            });

            index += 1;
        }

        match self.task_manager.run() {
            Ok(result_list) => {
                for metadata in result_list.into_iter().flatten() {
                    if let Some(cache) = &self.cache {
                        match cache.add_cache(&metadata) {
                            Ok(_r) => {}
                            Err(e) => {
                                log_error!("{}", e);
                            }
                        }
                    }
                    result.push(metadata);
                }
            }
            Err(e) => {
                log_error!("{}", e);
            }
        }

        result
    }
}

#[cfg(test)]
mod parser_tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_standard_parsing() {
        let file_path = "tests/standard.txt";
        let contents = fs::read_to_string(file_path).unwrap();

        let mut parser = Parser::new(Vec::<String>::new(), Vec::<String>::new(), None, 1, 1);

        let lines: Vec<String> = contents.lines().map(String::from).collect();

        let metadata_list = parser.parse_metadata(&lines);

        // println!("File constents:\n{}", contents);
        assert_eq! {lines.len(), metadata_list.len()};
    }
}
