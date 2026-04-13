use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style, Stylize};
use ratatui::symbols::Marker;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Axis, Block, Borders, Chart, Dataset, GraphType};

use crate::app::App;

const RAM_COLOR: Color = Color::Rgb(255, 220, 150); // warm yellow
const SWAP_COLOR: Color = Color::Rgb(180, 120, 255); // purple (matches dio)
const BORDER_COLOR: Color = Color::DarkGray;

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    draw_chart(frame, area, app);
}

fn draw_chart(frame: &mut Frame, area: Rect, app: &App) {
    let mut ram_data = Vec::new();
    let mut swap_data = Vec::new();
    app.ram_history.as_chart_data(&mut ram_data);
    app.swap_history.as_chart_data(&mut swap_data);

    let capacity = app.ram_history_capacity() as f64;

    let ram_label = app.latest_info.as_ref().map_or_else(
        || "RAM:  --%".to_string(),
        |info| info.ram_label(),
    );
    let swap_label = app.latest_info.as_ref().map_or_else(
        || "SWP:  --%".to_string(),
        |info| info.swap_label(),
    );

    let datasets = vec![
        Dataset::default()
            .name(Line::from(Span::styled(ram_label, Style::default().fg(RAM_COLOR))))
            .marker(Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(RAM_COLOR))
            .data(&ram_data),
        Dataset::default()
            .name(Line::from(Span::styled(swap_label, Style::default().fg(SWAP_COLOR))))
            .marker(Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(SWAP_COLOR))
            .data(&swap_data),
    ];

    let scrollback_secs = app.scrollback_secs;

    let x_axis = Axis::default()
        .style(Style::default().fg(BORDER_COLOR))
        .bounds([0.0, capacity - 1.0])
        .labels(vec![
            format!("{}s", scrollback_secs).bold(),
            "0s".to_string().bold(),
        ]);

    let y_axis = Axis::default()
        .style(Style::default().fg(BORDER_COLOR))
        .bounds([0.0, 100.0])
        .labels(vec![
            "0%".to_string().bold(),
            "100%".to_string().bold(),
        ]);

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .title(" Memory ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER_COLOR)),
        )
        .x_axis(x_axis)
        .y_axis(y_axis);

    frame.render_widget(chart, area);
}
