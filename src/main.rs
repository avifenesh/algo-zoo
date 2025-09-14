//! Sorting Race - Terminal-based sorting algorithm visualization

use anyhow::Result;

// Application constants
const DEFAULT_ARRAY_SIZE: usize = 50;
const DEFAULT_SEED: u64 = 12345;
const DEFAULT_BUDGET: usize = 16;
const DEFAULT_WALLTIME_SLICE_MS: u64 = 50;
const TARGET_FPS: u32 = 30;
const TICK_RATE_MS: u64 = 30;
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use sorting_race::{
    lib::{
        bar_chart::BarChart, memory_graph::MemoryGraph, progress::ProgressBars,
        sparkline::SparklineCollection,
    },
    models::{
        config::{Distribution, FairnessMode, RunConfiguration},
        traits::{FairnessModel, Sorter},
    },
    services::{
        fairness::{
            adaptive::AdaptiveFairness, comparison::ComparisonFairness, walltime::WallTimeFairness,
            weighted::WeightedFairness,
        },
        generator::ArrayGenerator,
        sorters::{
            bubble::BubbleSort, heap::HeapSort, insertion::InsertionSort, merge::MergeSort,
            quick::QuickSort, selection::SelectionSort, shell::ShellSort,
        },
    },
};
use std::{
    io,
    time::{Duration, Instant},
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Array size for sorting
    #[arg(short, long, default_value_t = DEFAULT_ARRAY_SIZE)]
    size: usize,

    /// Random seed for deterministic execution
    #[arg(short = 'S', long, default_value_t = DEFAULT_SEED)]
    seed: u64,

    /// Distribution type: shuffled, nearly-sorted, reversed, few-unique
    #[arg(short, long, default_value = "shuffled")]
    distribution: String,

    /// Fairness mode: comp, weighted, walltime, adaptive
    #[arg(short, long, default_value = "comp")]
    fair: String,

    /// Comparison budget for comp fairness mode
    #[arg(short = 'k', long, default_value_t = DEFAULT_BUDGET)]
    budget: usize,

    /// Alpha weight for weighted fairness (comparisons)
    #[arg(long, default_value_t = 1.0)]
    alpha: f32,

    /// Beta weight for weighted fairness (moves)
    #[arg(long, default_value_t = 1.0)]
    beta: f32,

    /// Learning rate for adaptive fairness (0.0-1.0)
    #[arg(long, default_value_t = 0.2)]
    learning_rate: f32,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Parse distribution
    let distribution = match args.distribution.as_str() {
        "nearly-sorted" => Distribution::NearlySorted,
        "reversed" => Distribution::Reversed,
        "few-unique" => Distribution::FewUnique,
        _ => Distribution::Shuffled,
    };

    // Parse fairness mode
    let fairness_mode = match args.fair.as_str() {
        "weighted" => FairnessMode::Weighted {
            alpha: args.alpha,
            beta: args.beta,
        },
        "walltime" => FairnessMode::WallTime {
            slice_ms: DEFAULT_WALLTIME_SLICE_MS,
        },
        "adaptive" => FairnessMode::Adaptive {
            learning_rate: args.learning_rate,
        },
        _ => FairnessMode::ComparisonBudget { k: args.budget },
    };

    // Create configuration
    let config = RunConfiguration {
        array_size: args.size,
        distribution,
        seed: args.seed,
        fairness_mode,
        target_fps: TARGET_FPS,
    };

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run the application
    let res = run_app(&mut terminal, config);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    config: RunConfiguration,
) -> Result<()> {
    // Generate initial array
    let generator = ArrayGenerator::new(config.seed);
    let array = generator.generate(config.array_size, &config.distribution);

    // Initialize sorting algorithms
    let mut algorithms: Vec<Box<dyn Sorter>> = vec![
        Box::new(BubbleSort::new()),
        Box::new(InsertionSort::new()),
        Box::new(SelectionSort::new()),
        Box::new(QuickSort::new()),
        Box::new(HeapSort::new()),
        Box::new(MergeSort::new()),
        Box::new(ShellSort::new()),
    ];

    // Reset all algorithms with the same array
    for algo in &mut algorithms {
        algo.reset(array.clone());
    }

    // Create fairness model based on configuration
    let fairness: Box<dyn FairnessModel> = match &config.fairness_mode {
        FairnessMode::ComparisonBudget { k } => Box::new(ComparisonFairness::new(*k)),
        FairnessMode::Weighted { alpha, beta } => Box::new(WeightedFairness::new(*alpha, *beta)),
        FairnessMode::WallTime { slice_ms } => Box::new(WallTimeFairness::new(*slice_ms)),
        FairnessMode::Adaptive { learning_rate } => Box::new(AdaptiveFairness::new(*learning_rate)),
        FairnessMode::EqualSteps => Box::new(ComparisonFairness::new(1)), // Fallback
    };

    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(TICK_RATE_MS);
    let mut paused = false;

    // Initialize visualization state
    let mut memory_graph = MemoryGraph::new();
    let mut sparklines = SparklineCollection::new(50, 1); // 50-point history
    let mut progress_bars = ProgressBars::new();

    loop {
        // Update visualization data
        for algo in &algorithms {
            let telemetry = algo.get_telemetry();
            let name = algo.name();

            // Update memory graph
            memory_graph.update_algorithm(name, telemetry.memory_current);

            // Update sparklines
            sparklines.update(
                &format!("{}_comparisons", name),
                telemetry.total_comparisons as f64,
            );
            sparklines.update(&format!("{}_moves", name), telemetry.total_moves as f64);

            // Update progress bars
            progress_bars.add_bar(name, telemetry.progress_hint);
        }

        terminal.draw(|f| {
            ui::<B>(
                f,
                &algorithms,
                &config,
                paused,
                &memory_graph,
                &sparklines,
                &progress_bars,
            )
        })?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char(' ') => paused = !paused,
                    KeyCode::Char('r') => {
                        // Reset with same seed
                        for algo in &mut algorithms {
                            algo.reset(array.clone());
                        }
                    },
                    _ => {},
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            if !paused {
                // Step all algorithms
                let budgets = fairness.allocate_budget(&algorithms);
                for (algo, budget) in algorithms.iter_mut().zip(budgets.iter()) {
                    if !algo.is_complete() {
                        algo.step(*budget);
                    }
                }
            }
            last_tick = Instant::now();
        }
    }
}

fn ui<B: ratatui::backend::Backend>(
    f: &mut Frame,
    algorithms: &[Box<dyn Sorter>],
    config: &RunConfiguration,
    paused: bool,
    memory_graph: &MemoryGraph,
    sparklines: &SparklineCollection,
    progress_bars: &ProgressBars,
) {
    // Main layout: header, body, footer
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Body
            Constraint::Length(3), // Footer
        ])
        .split(f.area());

    // Header
    let header = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(
                "Sorting Race",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" | "),
            Span::raw(format!(
                "Seed: {} | Size: {} | ",
                config.seed, config.array_size
            )),
            if paused {
                Span::styled("PAUSED", Style::default().fg(Color::Yellow))
            } else {
                Span::styled("RUNNING", Style::default().fg(Color::Green))
            },
        ]),
        Line::from("Press 'q' to quit, SPACE to pause/resume, 'r' to restart"),
    ])
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, main_chunks[0]);

    // Body layout: left (visualizations) and right (data)
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60), // Visualizations
            Constraint::Percentage(40), // Data panels
        ])
        .split(main_chunks[1]);

    // Left side: visualizations (split vertically)
    let vis_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50), // Bar chart area
            Constraint::Percentage(25), // Progress bars
            Constraint::Percentage(25), // Memory graph
        ])
        .split(body_chunks[0]);

    // Render bar chart for the first algorithm with data
    if let Some(first_algo) = algorithms.first() {
        let telemetry = first_algo.get_telemetry();
        let array_data = first_algo.get_array();

        let bar_chart = BarChart::from_array_with_colors(array_data, &telemetry.highlights)
            .scale_for_terminal(vis_chunks[0].width, vis_chunks[0].height)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Array View: {}", first_algo.name())),
            );

        f.render_widget(bar_chart, vis_chunks[0]);
    } else {
        let empty_chart = Block::default()
            .borders(Borders::ALL)
            .title("Array View: No Algorithm");
        f.render_widget(empty_chart, vis_chunks[0]);
    }

    // Progress bars
    let progress_widget = progress_bars
        .clone()
        .block(Block::default().borders(Borders::ALL).title("Progress"));
    f.render_widget(progress_widget, vis_chunks[1]);

    // Memory graph
    let memory_widget = memory_graph
        .clone()
        .block(Block::default().borders(Borders::ALL).title("Memory Usage"));
    f.render_widget(memory_widget, vis_chunks[2]);

    // Right side: data panels (split vertically)
    let data_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50), // Algorithm stats
            Constraint::Percentage(50), // Sparklines
        ])
        .split(body_chunks[1]);

    // Algorithm statistics list
    let items: Vec<ListItem> = algorithms
        .iter()
        .map(|algo| {
            let telemetry = algo.get_telemetry();
            let status = if algo.is_complete() { "✓" } else { "⟳" };

            let content = vec![
                Line::from(vec![
                    Span::styled(
                        format!("{:<12}", algo.name()),
                        if algo.is_complete() {
                            Style::default().fg(Color::Green)
                        } else {
                            Style::default().fg(Color::Cyan)
                        },
                    ),
                    Span::raw(format!(
                        " {} {:3.0}%",
                        status,
                        telemetry.progress_hint * 100.0
                    )),
                ]),
                Line::from(format!(
                    "    C:{:5} M:{:5} Mem:{:.1}KB",
                    telemetry.total_comparisons,
                    telemetry.total_moves,
                    telemetry.memory_current as f64 / 1024.0
                )),
            ];
            ListItem::new(content)
        })
        .collect();

    let algorithms_list =
        List::new(items).block(Block::default().borders(Borders::ALL).title("Statistics"));
    f.render_widget(algorithms_list, data_chunks[0]);

    // Sparklines area (simplified text display)
    let sparkline_text = if sparklines.len() > 0 {
        let mut text_lines = Vec::new();
        let algorithm_names: Vec<String> =
            algorithms.iter().map(|a| a.name().to_string()).collect();

        for name in &algorithm_names {
            if let Some(comp_sparkline) = sparklines.get(&format!("{}_comparisons", name)) {
                text_lines.push(Line::from(format!(
                    "{}: {}",
                    &name[..name.len().min(8)],
                    comp_sparkline.render_string()
                )));
            }
        }
        text_lines
    } else {
        vec![Line::from("No sparkline data yet")]
    };

    let sparklines_widget = Paragraph::new(sparkline_text).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Metrics History"),
    );
    f.render_widget(sparklines_widget, data_chunks[1]);

    // Footer
    let footer = Paragraph::new(format!(
        "Distribution: {:?} | Fairness: {:?}",
        config.distribution, config.fairness_mode
    ))
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, main_chunks[2]);
}
