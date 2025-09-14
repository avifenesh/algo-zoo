//! Sorting Race Visualization Library
//! 
//! A terminal-based sorting algorithm race visualization tool

pub mod models {
    pub mod traits;
    pub mod algorithm;
    pub mod metrics;
    pub mod config;
    pub mod markers;
}

pub mod services {
    pub mod sorters {
        pub mod bubble;
        pub mod insertion;
        pub mod selection;
        pub mod quick;
        pub mod heap;
        pub mod merge;
        pub mod shell;
    }
    
    pub mod fairness {
        pub mod comparison;
        pub mod weighted;
        pub mod walltime;
        pub mod adaptive;
    }
    
    pub mod generator;
    pub mod snapshot;
    pub mod memory;
}

pub mod lib {
    pub mod bar_chart;
    pub mod controller;
    pub mod input;
    pub mod memory_graph;
    pub mod progress;
    pub mod sparkline;
    pub mod visualization;
}