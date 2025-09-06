use chrono::{NaiveDateTime, NaiveTime};

#[derive(Clone)]
pub struct Metadata {
    pub name: String,
    pub b_finished: bool,
    pub episode: Option<u16>,
    pub time_at_episode: Option<NaiveTime>,
    pub season: Option<u16>,
    pub logged_time: Option<NaiveDateTime>,
    pub note: Option<String>,
}
