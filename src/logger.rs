use clap::ValueEnum;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, ValueEnum)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
}

#[derive(Debug)]
pub struct Logger {
    pub min_level: LogLevel,
}

pub static LOGGER: OnceLock<Logger> = OnceLock::new();

pub fn logger_init(min_level: LogLevel) {
    LOGGER
        .set(Logger { min_level })
        .expect("Logger is already initialized!");
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => ({
        if let Some(logger) = $crate::LOGGER.get() {
            if logger.min_level >= $crate::LogLevel::Error {
                eprintln!("ERROR: {}", format!($($arg)*));
            }
        }
    });
}
#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => ({
        if let Some(logger) = $crate::LOGGER.get() {
            if logger.min_level >= $crate::LogLevel::Warn {
                println!("WARN: {}", format!($($arg)*));
            }
        }
    });
}
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => ({
        if let Some(logger) = $crate::LOGGER.get() {
            if logger.min_level >= $crate::LogLevel::Info {
                println!("INFO: {}", format!($($arg)*));
            }
        }
    });
}
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => ({
        if let Some(logger) = $crate::LOGGER.get() {
            if logger.min_level >= $crate::LogLevel::Debug {
                println!("DEBUG: {}", format!($($arg)*));
            }
        }
    });
}
