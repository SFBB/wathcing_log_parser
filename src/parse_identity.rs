use super::datatype::Metadata;
use super::{log_debug, log_error, log_info, log_warn};
use chinese_number::from_chinese_to_u16;
use chrono::{NaiveDateTime, NaiveTime};
use regex::Regex;
use xxhash_rust::xxh3;

pub struct Parser {
    // reg pattern should follow:
    //  - supoport named group
    //  - group name follows the Metadata structutre's field
    //  - sub set should have higher priority
    pub reg_pattern_list: Vec<String>,
    // This is another pattern, it will run this against each line to determine if this entry
    // represent the watching is finished or not.
    pub finished_reg_pattern_list: Vec<String>,
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
        if hours.is_some() && minutes.is_some() && seconds.is_some() {
            return NaiveTime::from_hms_opt(
                hours.unwrap().into(),
                minutes.unwrap().into(),
                seconds.unwrap().into(),
            );
        }
    } else if parts.len() == 2 {
        let minutes = parse_number(parts[0]);
        let seconds = parse_number(parts[1]);
        if minutes.is_some() && seconds.is_some() {
            return NaiveTime::from_hms_opt(0, minutes.unwrap().into(), seconds.unwrap().into());
        }
    }
    None
}

impl Parser {
    pub fn new(reg_pattern_list: Vec<String>, finished_reg_pattern_list: Vec<String>) -> Self {
        Parser {
            reg_pattern_list,
            finished_reg_pattern_list,
        }
    }

    pub fn parse_metadata(&self, lines: &Vec<String>) -> Vec<Metadata> {
        let mut result = Vec::<Metadata>::new();

        for line in lines {
            let mut b_found: bool = false;
            for reg in &self.reg_pattern_list {
                let re = Regex::new(reg).unwrap();
                if re.is_match(line) {
                    if let Some(caps) = re.captures(line) {
                        let hash_value = xxh3::xxh3_64(line.as_bytes());

                        let name = String::from(caps.name("name").unwrap().as_str());

                        // if name.contains("") {
                        //     println!("{}", reg);
                        // }

                        let mut b_finished = false;
                        for finished_reg_pattern in &self.finished_reg_pattern_list {
                            let finished_re = Regex::new(finished_reg_pattern).unwrap();
                            if finished_re.is_match(line) {
                                b_finished = true;
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
                        result.push(Metadata {
                            name,
                            b_finished,
                            episode,
                            time_at_episode,
                            season,
                            logged_time,
                            note,
                        });

                        b_found = true;

                        break;
                    }
                }
            }

            if !b_found {
                log_error! {"This line cannot match any regex patterns:\n{}", line};
                log_error! {"This line cannot match any regex patterns:\n{}", line};
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

        let parser = Parser::new(Vec::<String>::new(), Vec::<String>::new());

        let lines: Vec<String> = contents
            .lines()
            .filter_map(|line| Some(line.to_string()))
            .collect();

        let metadata_list = parser.parse_metadata(&lines);

        // println!("File constents:\n{}", contents);
        assert_eq! {lines.len(), metadata_list.len()};
    }
}
