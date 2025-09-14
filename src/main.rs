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
        bar_chart::BarChart, interactive::InteractiveConfigMenu, memory_graph::MemoryGraph, progress::ProgressBars,
        sparkline::SparklineCollection,
    },
    models::{
        config::{Distribution, FairnessMode, RunConfiguration},
        configuration::ConfigurationState,
        interactive_mode::ApplicationMode,
        session::SessionState,
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

fn create_fairness_model(fairness_mode: &FairnessMode) -> Box<dyn FairnessModel> {
    match fairness_mode {
        FairnessMode::ComparisonBudget { k } => Box::new(ComparisonFairness::new(*k)),
        FairnessMode::Weighted { alpha, beta } => Box::new(WeightedFairness::new(*alpha, *beta)),
        FairnessMode::WallTime { slice_ms } => Box::new(WallTimeFairness::new(*slice_ms)),
        FairnessMode::Adaptive { learning_rate } => Box::new(AdaptiveFairness::new(*learning_rate)),
        FairnessMode::EqualSteps => Box::new(ComparisonFairness::new(1)), // Fallback
    }
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    config: RunConfiguration,
) -> Result<()> {
    // Initialize interactive configuration menu
    let config_state = ConfigurationState::from_run_config(&config);
    let mut interactive_menu = InteractiveConfigMenu::new();
    interactive_menu.config_state = config_state;
    let mut session_state = SessionState::new();
    let mut current_config = config;

    // Force start in Configuration mode for interactive experience
    interactive_menu.interactive_mode.current_mode = ApplicationMode::Configuration;

    // Generate initial array
    let generator = ArrayGenerator::new(current_config.seed);
    let mut array = generator.generate(current_config.array_size, &current_config.distribution);

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
    let mut fairness: Box<dyn FairnessModel> = create_fairness_model(&current_config.fairness_mode);

    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(TICK_RATE_MS);
    let mut paused = false;
    // Remove this - use interactive_menu.interactive_mode.array_view_algorithm instead
    // let mut selected_algorithm_index = 0; // Track which algorithm to display

    // Initialize visualization state
    let mut memory_graph = MemoryGraph::new();
    let mut sparklines = SparklineCollection::new(50, 1); // 50-point history
    let mut progress_bars = ProgressBars::new();

    loop {
        // Update visualization data
        for algo in &algorithms {
            let telemetry = algo.get_telemetry();
            let name = algo.name();

            // Update memory graph with actual memory usage
            let actual_memory = algo.get_memory_usage();
            let memory_to_use = if actual_memory > 0 {
                actual_memory
            } else {
                // Fall back to telemetry if get_memory_usage returns 0
                telemetry.memory_current
            };
            memory_graph.update_algorithm(name, memory_to_use);

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
            // Check if we should render the interactive menu overlay
            if !interactive_menu.is_racing() {
                // Render the actual interactive menu
                let area = f.area();
                interactive_menu.render(area, f.buffer_mut());
            } else {
                ui(
                    f,
                    &algorithms,
                    &current_config,
                    paused,
                    interactive_menu.interactive_mode.array_view_algorithm,
                    &memory_graph,
                    &sparklines,
                    &progress_bars,
                );
            }
        })?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)?
            && let Event::Key(key) = event::read()? {
                // Always handle interactive menu events
                let menu_handled = interactive_menu.handle_key_event(key)?;

                // Check if we just transitioned to racing mode
                if interactive_menu.should_start_new_race()
                    && let Some(new_run_config) = interactive_menu.get_run_config() {
                            current_config = new_run_config;

                            // Regenerate array with new configuration
                            let generator = ArrayGenerator::new(current_config.seed);
                            array = generator.generate(current_config.array_size, &current_config.distribution);

                            // Reset all algorithms with new array
                            for algo in &mut algorithms {
                                algo.reset(array.clone());
                            }

                            // Create new fairness model
                            fairness = create_fairness_model(&current_config.fairness_mode);

                            // Reset visualization state
                            memory_graph.reset_all();  // Reset memory data but keep algorithm names
                            sparklines = SparklineCollection::new(50, 1);
                            progress_bars = ProgressBars::new();

                            // Start new race
                            let _ = session_state.start_new_race();

                            // Unpause to start the race
                            paused = false;
                    }

                    // Handle additional key events not handled by menu
                    if !menu_handled {
                        match key.code {
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Char('r') => {
                            // Reset with same seed
                            for algo in &mut algorithms {
                                algo.reset(array.clone());
                            }
                            // Reset memory tracking
                            memory_graph.reset_all();
                        },
                        KeyCode::Char('k') | KeyCode::Char('b') | KeyCode::Char('f') => {
                            // Enter configuration mode
                            interactive_menu.interactive_mode.current_mode = ApplicationMode::Configuration;
                            paused = true; // Pause the race

                            // Set specific focus based on key
                            use sorting_race::models::interactive_mode::ConfigurationField;
                            match key.code {
                                KeyCode::Char('k') => {
                                    interactive_menu.interactive_mode.set_config_focus(ConfigurationField::ArraySize)?;
                                },
                                KeyCode::Char('b') => {
                                    interactive_menu.interactive_mode.set_config_focus(ConfigurationField::Distribution)?;
                                },
                                KeyCode::Char('f') => {
                                    interactive_menu.interactive_mode.set_config_focus(ConfigurationField::FairnessMode)?;
                                },
                                _ => {}
                            }
                        },
                        _ => {},
                    }
                }
        }

        if last_tick.elapsed() >= tick_rate {
            if interactive_menu.is_racing() && !paused {
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

#[allow(clippy::too_many_arguments)]
fn ui(
    f: &mut Frame,
    algorithms: &[Box<dyn Sorter>],
    config: &RunConfiguration,
    paused: bool,
    selected_algorithm_index: usize,
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
        Line::from("Press 'q' to quit, SPACE to pause/resume, 'v' to switch array view, 'r' to restart"),
        Line::from("Interactive: 'k' for array size, 'b' for distribution, 'f' for fairness mode"),
    ])
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, main_chunks[0]);

    // Body layout: array view at top, progress in middle, bottom panels at bottom
    let body_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10),     // Array view (full width)
            Constraint::Length(8),      // Progress bars (full width)
            Constraint::Min(0),         // Bottom panels (stats, metrics, memory)
        ])
        .split(main_chunks[1]);

    // Render bar chart for the selected algorithm (full width at top)
    if let Some(selected_algo) = algorithms.get(selected_algorithm_index) {
        let telemetry = selected_algo.get_telemetry();
        let array_data = selected_algo.get_array();

        // Use viewport mode for large arrays
        let (bar_chart, viewport_indicator) = BarChart::from_array_with_viewport(
            array_data,
            &telemetry.highlights,
            body_chunks[0].width,
            telemetry.highlights.first().copied()  // Center on first highlight
        );

        let title = if viewport_indicator.is_empty() {
            format!("Array View: {} (Press 'v' to switch)", selected_algo.name())
        } else {
            format!("Array View: {} {} (Press 'v' to switch)",
                    selected_algo.name(), viewport_indicator)
        };

        let bar_chart = bar_chart
            .scale_for_terminal(body_chunks[0].width, body_chunks[0].height)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title),
            );

        f.render_widget(bar_chart, body_chunks[0]);
    } else {
        let empty_chart = Block::default()
            .borders(Borders::ALL)
            .title("Array View: No Algorithm");
        f.render_widget(empty_chart, body_chunks[0]);
    }

    // Progress bars (full width in middle)
    let progress_widget = progress_bars
        .clone()
        .block(Block::default().borders(Borders::ALL).title("Progress"));
    f.render_widget(progress_widget, body_chunks[1]);

    // Bottom panels: split horizontally into three sections
    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33), // Algorithm stats
            Constraint::Percentage(34), // Sparklines/Metrics
            Constraint::Percentage(33), // Memory graph
        ])
        .split(body_chunks[2]);

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
                Line::from({
                    let actual_memory = algo.get_memory_usage();
                    let memory_display = if actual_memory > 0 {
                        format!("{:.1}KB", actual_memory as f64 / 1024.0)
                    } else if telemetry.memory_current > 0 {
                        format!("{:.1}KB", telemetry.memory_current as f64 / 1024.0)
                    } else {
                        "N/A".to_string()
                    };
                    format!(
                        "    C:{:5} M:{:5} Mem:{}",
                        telemetry.total_comparisons,
                        telemetry.total_moves,
                        memory_display
                    )
                }),
            ];
            ListItem::new(content)
        })
        .collect();

    let algorithms_list =
        List::new(items).block(Block::default().borders(Borders::ALL).title("Statistics"));
    f.render_widget(algorithms_list, bottom_chunks[0]);

    // Sparklines area (simplified text display)
    let sparkline_text = if !sparklines.is_empty() {
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
    f.render_widget(sparklines_widget, bottom_chunks[1]);

    // Memory graph in the third bottom panel
    let memory_title = if memory_graph.is_empty() {
        "Memory Usage (No data yet - press Space to start race)"
    } else {
        "Memory Usage"
    };
    let memory_widget = memory_graph
        .clone()
        .block(Block::default().borders(Borders::ALL).title(memory_title));
    f.render_widget(memory_widget, bottom_chunks[2]);

    // Footer
    let footer = Paragraph::new(format!(
        "Distribution: {:?} | Fairness: {:?}",
        config.distribution, config.fairness_mode
    ))
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, main_chunks[2]);
}
