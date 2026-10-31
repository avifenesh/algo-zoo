//! Sorting Race Visualization Library
//!
//! A terminal-based sorting algorithm race visualization tool

// Allow some clippy style lints that are too pedantic
#![allow(clippy::manual_clamp)]
#![allow(clippy::if_same_then_else)]

pub mod models;
pub mod services;

pub mod lib {
    pub mod bar_chart;
    pub mod controller;
    pub mod input;
    pub mod interactive;
    pub mod memory_graph;
    pub mod progress;
    pub mod sparkline;
    pub mod visualization;
}
