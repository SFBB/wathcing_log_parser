use std::collections::HashMap;

use super::datatype::Metadata;

#[derive(Clone)]
pub struct StatsInfo {
    pub name: String,
    pub season: Option<u16>,
    pub watched_times: u16,
    pub b_finished: bool,
    pub related_entry: Vec<Metadata>,
}

pub struct Stats {
    metadata_list: Vec<Metadata>,
    statsinfo_list: Vec<StatsInfo>,
    statsinfo_index_by_name: HashMap<String, usize>,
}

impl Stats {
    pub fn new(mut metadata_list: Vec<Metadata>) -> Self {
        metadata_list.sort_by(|a, b| a.index.cmp(&b.index));

        let mut statsinfo_list: Vec<StatsInfo> = Vec::new();
        let mut statsinfo_index_by_name: HashMap<String, usize> = HashMap::new();

        for metadata in &metadata_list {
            let title = format! {"{}-{:?}", &metadata.name, &metadata.season};
            if let std::collections::hash_map::Entry::Vacant(e) =
                statsinfo_index_by_name.entry(title.clone())
            {
                statsinfo_list.push(StatsInfo {
                    name: metadata.name.clone(),
                    season: metadata.season,
                    watched_times: 0,
                    b_finished: metadata.b_finished,
                    related_entry: vec![metadata.clone()],
                });
                e.insert(statsinfo_list.len() - 1);
            } else {
                let index: usize = statsinfo_index_by_name[&title];
                let statsinfo = &mut statsinfo_list[index];
                statsinfo.b_finished = metadata.b_finished || statsinfo.b_finished;
            }
        }
        Stats {
            metadata_list,
            statsinfo_list,
            statsinfo_index_by_name,
        }
    }

    pub fn stats_all(&self) -> Vec<StatsInfo> {
        self.statsinfo_list.clone()
    }

    pub fn stats_unfinished(&self) -> Vec<StatsInfo> {
        let mut result: Vec<StatsInfo> = Vec::new();
        for statsinfo in &self.statsinfo_list {
            if !statsinfo.b_finished {
                result.push(statsinfo.clone());
            }
        }
        result
    }

    pub fn query_by_name(&self, name: &str) -> Vec<StatsInfo> {
        let mut result = Vec::new();
        for statsinfo in &self.statsinfo_list {
            if statsinfo
                .name
                .to_lowercase()
                .contains(name.to_lowercase().as_str())
            {
                result.push(statsinfo.clone());
            }
        }
        result
    }
}
