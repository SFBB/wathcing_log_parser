use super::datatype::Metadata;
use std::any::Any;
use std::fmt::Display;
use std::num::NonZeroUsize;
use std::sync::Arc;
use thiserror::Error;

#[allow(dead_code)]
#[derive(Debug)]
struct ThreadErrorPayload(Box<dyn Any + Send>);

impl Display for ThreadErrorPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Thread panicked with a non-standard value")
    }
}

impl std::error::Error for ThreadErrorPayload {}

#[derive(Error, Debug)]
pub enum ParserTaskManagerError {
    #[error("Thread Error")]
    ThreadError {
        #[source]
        source: Box<dyn std::error::Error + Send + 'static>,
    },
}

impl From<Box<dyn Any + Send>> for ParserTaskManagerError {
    fn from(err: Box<dyn Any + Send>) -> Self {
        match err.downcast::<Box<dyn std::error::Error + Send + 'static>>() {
            Ok(standard_error) => ParserTaskManagerError::ThreadError {
                source: *standard_error,
            },
            Err(e) => ParserTaskManagerError::ThreadError {
                source: Box::new(ThreadErrorPayload(e)),
            },
        }
    }
}

pub type ParserTaskManagerResult<T> = Result<T, ParserTaskManagerError>;

pub trait ParserCallback:
    Fn(&str, u32, u64, &Vec<String>, &Vec<String>) -> Option<Metadata> + Send + Sync + 'static + Clone
{
}

impl<F> ParserCallback for F where
    F: Fn(&str, u32, u64, &Vec<String>, &Vec<String>) -> Option<Metadata>
        + Send
        + Sync
        + 'static
        + Clone
{
}

#[derive(Clone, Debug)]
pub struct ParserTask<F>
where
    F: ParserCallback,
{
    pub index: u32,
    pub hash_value: u64,
    pub line: String,
    pub reg_pattern_list: Vec<String>,
    pub finished_reg_pattern_list: Vec<String>,
    pub callback: F,
}

pub struct ParserTaskManager<F>
where
    F: ParserCallback,
{
    max_thread_num: usize,
    min_tasks_per_thread: usize,
    task_list: Vec<ParserTask<F>>,
}

impl<F> ParserTaskManager<F>
where
    F: ParserCallback,
{
    pub fn new(max_thread_num: usize, min_tasks_per_thread: usize) -> Self {
        Self {
            max_thread_num: std::cmp::max(
                std::cmp::min(
                    max_thread_num,
                    std::thread::available_parallelism()
                        .ok()
                        .unwrap_or(NonZeroUsize::new(1).unwrap())
                        .into(),
                ),
                1,
            ),
            min_tasks_per_thread,
            task_list: Vec::new(),
        }
    }

    pub fn add_task(&mut self, task: ParserTask<F>) {
        self.task_list.push(task);
    }

    pub fn get_task_count(&self) -> usize {
        self.task_list.len()
    }

    pub fn get_thread_count(&self) -> usize {
        let total_tasks = self.get_task_count();
        let calculated_threads = total_tasks.div_ceil(self.min_tasks_per_thread);
        std::cmp::min(calculated_threads, self.max_thread_num)
    }

    pub fn run(&self) -> ParserTaskManagerResult<Vec<Option<Metadata>>> {
        let mut final_result = vec![];

        let thread_num = self.get_thread_count();
        let task_num_per_thread = self.get_task_count();

        let task_list_arc = Arc::new(self.task_list.clone());
        let mut join_hanfle_list = vec![];

        for i in 0..thread_num {
            let tasks_arc = Arc::clone(&task_list_arc);
            let join_handle = std::thread::spawn(move || {
                let mut result = vec![];
                let start_index = i * task_num_per_thread;
                let end_index = std::cmp::min(start_index + task_num_per_thread, tasks_arc.len());
                for j in start_index..end_index {
                    let task = &tasks_arc[j];
                    let metadata = (task.callback)(
                        &task.line,
                        task.index,
                        task.hash_value,
                        &task.reg_pattern_list,
                        &task.finished_reg_pattern_list,
                    );
                    result.push(metadata);
                }
                result
            });
            join_hanfle_list.push(join_handle);
        }

        for join_handle in join_hanfle_list {
            final_result.extend(join_handle.join()?);
        }

        Ok(final_result)
    }
}
