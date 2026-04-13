use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::symbols::Marker;
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Axis, Block, Borders, Chart, Dataset, Gauge, GraphType, Paragraph,
};

use crate::app::App;
use crate::collector::{human_count, human_rate};

const ALLOC_COLOR: Color = Color::Rgb(255, 220, 150);   // warm yellow
const FREE_COLOR: Color = Color::Rgb(180, 120, 255);    // purple
const SWAPIN_COLOR: Color = Color::Rgb(100, 200, 255);  // light blue
const SWAPOUT_COLOR: Color = Color::Rgb(255, 130, 130); // coral
const FAULT_COLOR: Color = Color::Rgb(150, 255, 150);   // green
const MAJOR_FAULT_COLOR: Color = Color::Rgb(255, 100, 100); // red
const PSI_SOME_COLOR: Color = Color::Rgb(255, 200, 100);  // orange
const PSI_FULL_COLOR: Color = Color::Rgb(255, 80, 80);    // bright red
const BORDER_COLOR: Color = Color::DarkGray;
const LABEL_COLOR: Color = Color::Gray;

pub fn render(frame: &mut Frame, app: &App) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),     // hardware header
            Constraint::Min(0),        // everything else
        ])
        .split(frame.area());

    draw_header(frame, outer[0], app);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(25), // RAM row
            Constraint::Percentage(25), // Swap row
            Constraint::Percentage(25), // Faults row
            Constraint::Percentage(25), // PSI row
        ])
        .split(outer[1]);

    draw_row(frame, rows[0], app, RowKind::Ram);
    draw_row(frame, rows[1], app, RowKind::Swap);
    draw_row(frame, rows[2], app, RowKind::Faults);
    draw_row(frame, rows[3], app, RowKind::Psi);
}

fn draw_header(frame: &mut Frame, area: Rect, app: &App) {
    let text = Paragraph::new(Line::from(vec![
        Span::styled(" RAM ", Style::default().fg(ALLOC_COLOR).add_modifier(Modifier::BOLD)),
        Span::styled(
            &app.hardware.summary,
            Style::default().fg(LABEL_COLOR),
        ),
    ]));
    frame.render_widget(text, area);
}

enum RowKind {
    Ram,
    Swap,
    Faults,
    Psi,
}

fn draw_row(frame: &mut Frame, area: Rect, app: &App, kind: RowKind) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(30),       // chart (takes remaining space)
            Constraint::Length(30),     // gauge sidebar
        ])
        .split(area);

    match kind {
        RowKind::Ram => {
            draw_throughput_chart(frame, cols[0], app);
            draw_ram_gauge(frame, cols[1], app);
        }
        RowKind::Swap => {
            draw_swap_io_chart(frame, cols[0], app);
            draw_swap_gauge(frame, cols[1], app);
        }
        RowKind::Faults => {
            draw_faults_chart(frame, cols[0], app);
            draw_dirty_gauge(frame, cols[1], app);
        }
        RowKind::Psi => {
            draw_psi_chart(frame, cols[0], app);
            draw_psi_gauge(frame, cols[1], app);
        }
    }
}

fn draw_throughput_chart(frame: &mut Frame, area: Rect, app: &App) {
    let mut alloc_data = Vec::new();
    let mut free_data = Vec::new();
    app.alloc_history.as_chart_data(&mut alloc_data);
    app.free_history.as_chart_data(&mut free_data);

    let alloc_label = app.latest_rates.as_ref().map_or_else(
        || "alloc: --".to_string(),
        |rates| format!("alloc: {}", human_rate(rates.alloc_mb_per_sec)),
    );
    let free_label = app.latest_rates.as_ref().map_or_else(
        || "free: --".to_string(),
        |rates| format!("free:  {}", human_rate(rates.free_mb_per_sec)),
    );

    let y_max = auto_scale_max(app.alloc_history.max().max(app.free_history.max()));

    let datasets = vec![
        make_dataset(&alloc_label, ALLOC_COLOR, &alloc_data),
        make_dataset(&free_label, FREE_COLOR, &free_data),
    ];

    render_chart(frame, area, " Throughput ", "MB/s", datasets, app, y_max,
        |v| human_rate(v));
}

fn draw_swap_io_chart(frame: &mut Frame, area: Rect, app: &App) {
    let mut swapin_data = Vec::new();
    let mut swapout_data = Vec::new();
    app.swapin_history.as_chart_data(&mut swapin_data);
    app.swapout_history.as_chart_data(&mut swapout_data);

    let swapin_label = app.latest_rates.as_ref().map_or_else(
        || "in: --".to_string(),
        |rates| format!("in:  {}", human_rate(rates.swapin_mb_per_sec)),
    );
    let swapout_label = app.latest_rates.as_ref().map_or_else(
        || "out: --".to_string(),
        |rates| format!("out: {}", human_rate(rates.swapout_mb_per_sec)),
    );

    let y_max = auto_scale_max(app.swapin_history.max().max(app.swapout_history.max()));

    let datasets = vec![
        make_dataset(&swapin_label, SWAPIN_COLOR, &swapin_data),
        make_dataset(&swapout_label, SWAPOUT_COLOR, &swapout_data),
    ];

    render_chart(frame, area, " Swap I/O ", "MB/s", datasets, app, y_max,
        |v| human_rate(v));
}

fn draw_faults_chart(frame: &mut Frame, area: Rect, app: &App) {
    let mut fault_data = Vec::new();
    let mut major_data = Vec::new();
    app.fault_history.as_chart_data(&mut fault_data);
    app.major_fault_history.as_chart_data(&mut major_data);

    let fault_label = app.latest_rates.as_ref().map_or_else(
        || "minor: --".to_string(),
        |rates| format!("minor: {}", human_count(rates.fault_per_sec)),
    );
    let major_label = app.latest_rates.as_ref().map_or_else(
        || "major: --".to_string(),
        |rates| format!("major: {}", human_count(rates.major_fault_per_sec)),
    );

    let y_max = auto_scale_max(app.fault_history.max().max(app.major_fault_history.max()));

    let datasets = vec![
        make_dataset(&fault_label, FAULT_COLOR, &fault_data),
        make_dataset(&major_label, MAJOR_FAULT_COLOR, &major_data),
    ];

    render_chart(frame, area, " Page Faults ", "/s", datasets, app, y_max,
        |v| human_count(v));
}

fn draw_psi_chart(frame: &mut Frame, area: Rect, app: &App) {
    let mut some_data = Vec::new();
    let mut full_data = Vec::new();
    app.psi_some_history.as_chart_data(&mut some_data);
    app.psi_full_history.as_chart_data(&mut full_data);

    let some_label = app.latest_psi.as_ref().map_or_else(
        || "some: --".to_string(),
        |psi| psi.some_label(),
    );
    let full_label = app.latest_psi.as_ref().map_or_else(
        || "full: --".to_string(),
        |psi| psi.full_label(),
    );

    let y_max = auto_scale_pct(app.psi_some_history.max().max(app.psi_full_history.max()));

    let datasets = vec![
        make_dataset(&some_label, PSI_SOME_COLOR, &some_data),
        make_dataset(&full_label, PSI_FULL_COLOR, &full_data),
    ];

    render_chart(frame, area, " Memory Pressure (PSI) ", "%", datasets, app, y_max,
        |v| format!("{:.0}%", v));
}

fn draw_ram_gauge(frame: &mut Frame, area: Rect, app: &App) {
    let (pct, label) = app.latest_info.as_ref().map_or_else(
        || (0.0, "RAM: --%".to_string()),
        |info| (info.ram_pct(), info.ram_label()),
    );
    draw_gauge(frame, area, &label, pct, ALLOC_COLOR);
}

fn draw_swap_gauge(frame: &mut Frame, area: Rect, app: &App) {
    let (pct, label) = app.latest_info.as_ref().map_or_else(
        || (0.0, "SWP: --%".to_string()),
        |info| (info.swap_pct(), info.swap_label()),
    );
    draw_gauge(frame, area, &label, pct, SWAPIN_COLOR);
}

fn draw_dirty_gauge(frame: &mut Frame, area: Rect, app: &App) {
    let label = app.latest_info.as_ref().map_or_else(
        || "Dirty+WB: --".to_string(),
        |info| info.dirty_label(),
    );
    let dirty_kb = app.latest_info.as_ref().map_or(0, |info| info.dirty_writeback_kb());
    let ram_total = app.latest_info.as_ref().map_or(1, |info| info.ram_total_kb.max(1));
    let pct = (dirty_kb as f64 / ram_total as f64) * 100.0;
    draw_gauge(frame, area, &label, pct, FAULT_COLOR);
}

fn draw_psi_gauge(frame: &mut Frame, area: Rect, app: &App) {
    let label = app.latest_psi.as_ref().map_or_else(
        || "PSI: --".to_string(),
        |psi| psi.summary_label(),
    );
    let pct = app.latest_psi.as_ref().map_or(0.0, |psi| psi.severity_pct());
    let color = if pct >= 10.0 {
        PSI_FULL_COLOR
    } else if pct >= 1.0 {
        PSI_SOME_COLOR
    } else {
        Color::Green
    };
    draw_gauge(frame, area, &label, pct, color);
}

fn draw_gauge(frame: &mut Frame, area: Rect, label: &str, pct: f64, color: Color) {
    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER_COLOR)),
        )
        .gauge_style(Style::default().fg(color).add_modifier(Modifier::BOLD))
        .label(Span::styled(
            label,
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ))
        .ratio(pct.clamp(0.0, 100.0) / 100.0);

    frame.render_widget(gauge, area);
}

fn make_dataset<'a>(label: &str, color: Color, data: &'a [(f64, f64)]) -> Dataset<'a> {
    Dataset::default()
        .name(Line::from(Span::styled(
            label.to_string(),
            Style::default().fg(color),
        )))
        .marker(Marker::HalfBlock)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(color))
        .data(data)
}

fn render_chart<F>(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    y_title: &str,
    datasets: Vec<Dataset<'_>>,
    app: &App,
    y_max: f64,
    format_y: F,
) where
    F: Fn(f64) -> String,
{
    let capacity = app.chart_capacity() as f64;

    let x_axis = Axis::default()
        .title(Span::styled("Time", Style::default().fg(LABEL_COLOR)))
        .style(Style::default().fg(BORDER_COLOR))
        .bounds([0.0, capacity - 1.0])
        .labels(vec![
            format!("{}s", app.scrollback_secs).bold(),
            "0s".to_string().bold(),
        ]);

    let y_axis = Axis::default()
        .title(Span::styled(y_title, Style::default().fg(LABEL_COLOR)))
        .style(Style::default().fg(BORDER_COLOR))
        .bounds([0.0, y_max])
        .labels(vec![
            "0".to_string().bold(),
            format_y(y_max).bold(),
        ]);

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER_COLOR)),
        )
        .x_axis(x_axis)
        .y_axis(y_axis);

    frame.render_widget(chart, area);
}

fn auto_scale_max(observed_max: f64) -> f64 {
    if observed_max <= 0.0 {
        return 10.0;
    }
    let padded = observed_max * 1.2;
    let steps: &[f64] = &[
        1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 100.0, 200.0, 500.0,
        1000.0, 2000.0, 5000.0, 10000.0, 50000.0, 100000.0,
    ];
    steps.iter()
        .find(|&&step| step >= padded)
        .copied()
        .unwrap_or(padded.ceil())
}

fn auto_scale_pct(observed_max: f64) -> f64 {
    if observed_max <= 0.0 {
        return 5.0;
    }
    let padded = observed_max * 1.2;
    let steps: &[f64] = &[1.0, 2.0, 5.0, 10.0, 25.0, 50.0, 100.0];
    steps.iter()
        .find(|&&step| step >= padded)
        .copied()
        .unwrap_or(100.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_scale_max() {
        assert_eq!(auto_scale_max(0.0), 10.0);
        assert_eq!(auto_scale_max(0.8), 1.0);
        assert_eq!(auto_scale_max(3.5), 5.0);
        assert_eq!(auto_scale_max(42.0), 100.0);
        assert_eq!(auto_scale_max(800.0), 1000.0);
    }

    #[test]
    fn test_auto_scale_pct() {
        assert_eq!(auto_scale_pct(0.0), 5.0);
        assert_eq!(auto_scale_pct(0.5), 1.0);
        assert_eq!(auto_scale_pct(3.0), 5.0);
        assert_eq!(auto_scale_pct(80.0), 100.0);
    }
}
