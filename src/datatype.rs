use chrono::{NaiveDateTime, NaiveTime};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub id: u64,
    pub name: String,
    pub b_finished: bool,
    pub episode: Option<u16>,
    pub time_at_episode: Option<NaiveTime>,
    pub season: Option<u16>,
    pub logged_time: Option<NaiveDateTime>,
    pub note: Option<String>,
    pub raw_line: String,
    pub reg_pattern_matched: String,
    pub finished_reg_pattern_matched: Option<String>,
}
