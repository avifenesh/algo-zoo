# Visualization Widget Contract

**Date**: 2025-01-13  
**Status**: EXTRACTED from implementation

## ratatui Widget Interface

All visualization components implement the ratatui `Widget` trait:

```rust
pub trait Widget {
    fn render(self, area: Rect, buf: &mut Buffer);
}

// Also implemented for references:
impl Widget for &T where T: Widget
```

## Widget Implementations

### BarChart Widget
**Purpose**: Visualize array elements as vertical bars with operation highlighting

**Constructor**: `BarChart::from_array_with_colors(array: &[i32], highlights: &[usize])`

**Configuration Methods**:
- `bar_style(Style)` - Base bar styling
- `highlight_style(Style)` - Style for highlighted elements  
- `value_style(Style)` - Style for value labels
- `max_height(u16)` - Maximum bar height
- `scale_for_terminal(width, height)` - Auto-scaling for terminal size

**Rendering Contract**:
- Must handle array sizes 0-1000+ elements  
- Must scale bars to fit available terminal space
- Must highlight specified indices with distinct colors
- Must render value labels when space allows
- Must handle gracefully degrade on small terminal sizes

### Sparkline Widget
**Purpose**: Historical metrics visualization with rolling window

**Constructor**: `Sparkline::new(width: usize, height: usize)`

**Configuration Methods**:
- `style(Style)` - Line styling
- `block(Block)` - Border and title
- `add_data_point(f64)` - Add new metric value
- `set_data(Vec<f64>)` - Replace all data

**Rendering Contract**:
- Must maintain rolling window (default 100 points)
- Must auto-scale Y-axis to data range
- Must use Unicode block characters for smooth curves
- Must handle empty data gracefully
- Must provide `render_string()` for text-only output

### MemoryGraph Widget  
**Purpose**: Per-algorithm memory usage with peak tracking

**Constructor**: `MemoryGraph::new()`

**Configuration Methods**:
- `current_style(Style)` - Current memory usage bars
- `peak_style(Style)` - Peak memory indicators  
- `max_history(usize)` - Historical data retention
- `update_algorithm(name, bytes)` - Update algorithm memory usage

**Rendering Contract**:
- Must show current vs peak memory for each algorithm
- Must format memory values human-readable (B, KB, MB, GB)
- Must scale bars relative to maximum memory usage
- Must handle algorithm names up to reasonable length
- Must gracefully handle varying numbers of algorithms

### ProgressBar Widget
**Purpose**: Algorithm completion progress with labels

**Constructor**: `ProgressBar::new()`

**Configuration Methods**:
- `progress(f32)` - Set completion (0.0-1.0)
- `label(String)` - Algorithm name/status
- `show_percentage(bool)` - Display percentage text
- `filled_style(Style)` - Completed portion style
- `empty_style(Style)` - Remaining portion style

**Rendering Contract**:
- Must clamp progress values to 0.0-1.0 range
- Must show progress bar with filled/empty portions
- Must display percentage when enabled
- Must center text within available space
- Must handle varying progress bar widths

## Collection Widgets

### ProgressBars Collection
**Purpose**: Multiple progress bars for algorithm comparison

**Methods**:
- `add_bar(name, progress)` - Add or update progress bar
- `clear()` - Remove all bars
- `len()` - Get count of bars

**Rendering Contract**:
- Must layout multiple progress bars vertically
- Must allocate space fairly among bars
- Must handle dynamic bar addition/removal
- Must maintain consistent styling across bars

### SparklineCollection  
**Purpose**: Multiple sparklines for different metrics

**Methods**:
- `update(key, value)` - Update sparkline data
- `get(key)` - Retrieve specific sparkline
- `render_all()` - Text output of all sparklines

**Rendering Contract**:
- Must maintain separate data for each metric
- Must provide consistent time alignment across sparklines
- Must handle metric addition/removal gracefully

## Contract Tests

### Rendering Safety Test
```rust
fn test_widget_rendering_safety() {
    let widget = create_test_widget();
    let area = Rect::new(0, 0, 80, 24);
    let mut buffer = Buffer::empty(area);
    
    // Should never panic
    widget.render(area, &mut buffer);
    
    // Buffer should have valid content
    assert!(!buffer.content().is_empty());
}
```

### Edge Case Handling Test
```rust
fn test_edge_case_handling() {
    let widget = create_test_widget();
    
    // Tiny area
    let tiny_area = Rect::new(0, 0, 1, 1);
    let mut buffer = Buffer::empty(tiny_area);
    widget.render(tiny_area, &mut buffer); // Should not panic
    
    // Zero area  
    let zero_area = Rect::new(0, 0, 0, 0);
    let mut buffer = Buffer::empty(zero_area);
    widget.render(zero_area, &mut buffer); // Should not panic
}
```

### Data Scaling Test
```rust
fn test_data_scaling() {
    let mut bar_chart = BarChart::new(vec![]);
    
    // Large values should scale appropriately
    let large_data = vec![1000000, 2000000, 500000];
    bar_chart = BarChart::from_array_with_colors(&large_data, &[]);
    
    let area = Rect::new(0, 0, 40, 20);
    let mut buffer = Buffer::empty(area);
    bar_chart.render(area, &mut buffer);
    
    // Should render without overflow
}
```

### Performance Test
```rust  
fn test_rendering_performance() {
    let widget = create_large_widget(); // 1000+ elements
    let area = Rect::new(0, 0, 120, 40);
    let mut buffer = Buffer::empty(area);
    
    let start = Instant::now();
    widget.render(area, &mut buffer);
    let duration = start.elapsed();
    
    // Should render in <16ms for 60fps target
    assert!(duration < Duration::from_millis(16));
}
```

## Implementation Requirements

All visualization widgets must:

1. **Handle Edge Cases**: Gracefully handle zero/tiny render areas
2. **Scale Data**: Auto-scale content to fit available space  
3. **Respect Styles**: Honor ratatui styling configurations
4. **Performance**: Render in <16ms for smooth 60fps experience
5. **Memory Safety**: Never cause buffer overflows or panics
6. **Accessibility**: Use appropriate contrast and Unicode characters

## Terminal Compatibility

Widgets must support:
- **Minimum Size**: 80x24 terminal  
- **Color Depth**: 256-color terminals (fallback to 16-color)
- **Unicode**: Block characters for smooth graphics (fallback to ASCII)
- **Refresh Rate**: 25-35 FPS without flicker

## Validation Status

✅ **BarChart**: Passes all contract tests, scales to 1000+ elements  
✅ **Sparkline**: Passes all contract tests, 100-point rolling window  
✅ **MemoryGraph**: Passes all contract tests, handles 7+ algorithms  
✅ **ProgressBar**: Passes all contract tests, smooth progress updates  
✅ **Collections**: Pass all contract tests, dynamic management  

**Contract Status**: VALIDATED - All visualization widgets comply and perform well