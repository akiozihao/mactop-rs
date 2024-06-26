use std::{
    cmp::max,
    collections::HashMap,
    error,
    io::{BufReader, Read},
    process::{Command, Stdio},
};

use regex::Regex;

use crate::metrics::Metrics;

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,

    /// counter
    pub counter: u8,

    /// metrics
    pub metrics: Metrics,

    /// history
    pub cpu_w: Vec<f64>,
}

impl Default for App {
    fn default() -> Self {
        Self { running: true, counter: 0, metrics: Metrics::new(), cpu_w: vec![] }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        Self { metrics: Metrics::new(), ..Default::default() }
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn increment_counter(&mut self) {
        if let Some(res) = self.counter.checked_add(1) {
            self.counter = res;
        }
    }

    pub fn decrement_counter(&mut self) {
        if let Some(res) = self.counter.checked_sub(1) {
            self.counter = res;
        }
    }
}
