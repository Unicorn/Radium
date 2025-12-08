//! Cost dashboard view for displaying cost analytics.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Table, BarChart, Bar},
};
use crossterm::event::KeyCode;

use crate::state::{CostDashboardState, DisplayRow};
use radium_core::analytics::CostAnalytics;

/// Renders the cost dashboard view.
pub fn render_cost_dashboard(
    frame: &mut Frame,
    area: Rect,
    state: &mut CostDashboardState,
    analytics: &CostAnalytics,
) {
    let theme = crate::theme::get_theme();

    // Create layout: header, filters, main content, footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Length(3),  // Filters
            Constraint::Min(10),    // Main content
            Constraint::Length(3),  // Footer
        ])
        .split(area);

    // Header
    render_header(frame, chunks[0], state, &theme);

    // Filter bar
    render_filters(frame, chunks[1], state, &theme);

    // Main content area
    render_main_content(frame, chunks[2], state, &theme);

    // Footer with shortcuts
    render_footer(frame, chunks[3], state, &theme);
}

/// Renders the header section.
fn render_header(
    frame: &mut Frame,
    area: Rect,
    state: &CostDashboardState,
    theme: &crate::theme::RadiumTheme,
) {
    let title = "Cost Analytics Dashboard";
    let date_range = state.date_range_filter.display_name();
    let header_text = format!("{} | {}", title, date_range);

    let widget = Paragraph::new(header_text)
        .style(Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border_active))
                .title(" Cost Dashboard "),
        );

    frame.render_widget(widget, area);
}

/// Renders the filter bar.
fn render_filters(
    frame: &mut Frame,
    area: Rect,
    state: &CostDashboardState,
    theme: &crate::theme::RadiumTheme,
) {
    let grouping = state.grouping_mode.display_name();
    let view_mode = state.view_mode.display_name();
    let filter_text = format!(
        "Grouping: {} | View: {} | Sort: {:?} {}",
        grouping,
        view_mode,
        state.sort_column,
        if state.sort_ascending { "↑" } else { "↓" }
    );

    let widget = Paragraph::new(filter_text)
        .style(Style::default().fg(theme.text))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .title(" Filters "),
        );

    frame.render_widget(widget, area);
}

/// Renders the main content area.
fn render_main_content(
    frame: &mut Frame,
    area: Rect,
    state: &CostDashboardState,
    theme: &crate::theme::RadiumTheme,
) {
    if state.loading {
        let loading_text = "Loading cost data...";
        let widget = Paragraph::new(loading_text)
            .style(Style::default().fg(theme.text_muted))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .title(" Loading "),
            );
        frame.render_widget(widget, area);
        return;
    }

    if let Some(ref error) = state.error {
        let widget = Paragraph::new(format!("Error: {}", error))
            .style(Style::default().fg(theme.error))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .title(" Error "),
            );
        frame.render_widget(widget, area);
        return;
    }

    // Split main content: table (70%) and summary (30%)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(70),
            Constraint::Percentage(30),
        ])
        .split(area);

    // Left: Table with cost breakdown
    render_cost_table(frame, chunks[0], state, theme);

    // Right: Summary panel
    render_summary_panel(frame, chunks[1], state, theme);
}

/// Renders the cost breakdown table.
fn render_cost_table(
    frame: &mut Frame,
    area: Rect,
    state: &CostDashboardState,
    theme: &crate::theme::RadiumTheme,
) {
    let display_data = state.get_display_data();

    if display_data.is_empty() {
        let empty_text = "No cost data available for selected period";
        let widget = Paragraph::new(empty_text)
            .style(Style::default().fg(theme.text_muted))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .title(" Cost Breakdown "),
            );
        frame.render_widget(widget, area);
        return;
    }

    // Create table rows
    let headers = vec!["Name", "Cost", "Tokens", "Count"];
    let header_row = Row::new(
        headers
            .iter()
            .map(|h| {
                Cell::from(*h)
                    .style(Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))
            }),
    )
    .height(1);

    let rows: Vec<Row> = display_data
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let is_selected = i == state.selected_row;
            let style = if is_selected {
                Style::default()
                    .fg(theme.bg_primary)
                    .bg(theme.primary)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text)
            };

            let cost_str = format!("${:.4}", row.cost);
            let tokens_str = format_number(row.tokens);
            let count_str = row.count.to_string();

            Row::new(vec![
                Cell::from(row.key.as_str()).style(style),
                Cell::from(cost_str).style(style),
                Cell::from(tokens_str).style(style),
                Cell::from(count_str).style(style),
            ])
            .height(1)
        })
        .collect();

    let widths = vec![
        Constraint::Percentage(40),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
    ];

    let table = Table::new(rows, &widths)
        .header(header_row)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .title(" Cost Breakdown "),
        )
        .highlight_style(
            Style::default()
                .fg(theme.bg_primary)
                .bg(theme.primary)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    // Create a table state for rendering
    let mut table_state = ratatui::widgets::TableState::default();
    table_state.select(Some(state.selected_row));

    frame.render_stateful_widget(table, area, &mut table_state);
}

/// Renders the summary panel.
fn render_summary_panel(
    frame: &mut Frame,
    area: Rect,
    state: &CostDashboardState,
    theme: &crate::theme::RadiumTheme,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),  // Total summary
            Constraint::Length(6),  // Top requirements
            Constraint::Min(5),     // Provider breakdown
        ])
        .split(area);

    // Total cost summary
    if let Some(ref summary) = state.summary {
        let total_text = format!(
            "Total Cost: ${:.4}\nTotal Tokens: {}\nEvents: {}",
            summary.total_cost,
            format_number(summary.total_tokens),
            summary.event_count
        );

        let widget = Paragraph::new(total_text)
            .style(Style::default().fg(theme.text))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .title(" Summary "),
            );

        frame.render_widget(widget, chunks[0]);
    }

    // Top 5 expensive requirements
    let top_requirements = state.get_top_requirements(5);
    if !top_requirements.is_empty() {
        let mut lines = vec!["Top Requirements:".to_string()];
        for (i, req) in top_requirements.iter().take(5).enumerate() {
            lines.push(format!("{}. {}: ${:.4}", i + 1, req.key, req.total_cost));
        }

        let widget = Paragraph::new(lines.join("\n"))
            .style(Style::default().fg(theme.text))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .title(" Top Requirements "),
            );

        frame.render_widget(widget, chunks[1]);
    }

    // Provider breakdown (if available)
    if let Some(ref summary) = state.summary {
        if !summary.breakdown_by_provider.is_empty() {
            render_provider_breakdown_chart(frame, chunks[2], summary, theme);
        }
    }
}

/// Renders provider breakdown as a bar chart.
fn render_provider_breakdown_chart(
    frame: &mut Frame,
    area: Rect,
    summary: &radium_core::analytics::CostSummary,
    theme: &crate::theme::RadiumTheme,
) {
    let total_cost: f64 = summary
        .breakdown_by_provider
        .iter()
        .map(|b| b.total_cost)
        .sum();
    let max_cost = summary
        .breakdown_by_provider
        .iter()
        .map(|b| b.total_cost)
        .fold(0.0, f64::max);

    if max_cost == 0.0 {
        let widget = Paragraph::new("No provider data")
            .style(Style::default().fg(theme.text_muted))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .title(" Provider Breakdown "),
            );
        frame.render_widget(widget, area);
        return;
    }

    let bars: Vec<Bar> = summary
        .breakdown_by_provider
        .iter()
        .enumerate()
        .map(|(i, breakdown)| {
            let percentage = if total_cost > 0.0 {
                (breakdown.total_cost / total_cost) * 100.0
            } else {
                0.0
            };
            let color = match i % 3 {
                0 => Color::Blue,    // OpenAI
                1 => Color::Magenta, // Anthropic
                2 => Color::Green,   // Gemini
                _ => Color::Yellow,
            };
            Bar::default()
                .value(breakdown.total_cost as u64)
                .label(format!("{} {:.1}%", breakdown.key, percentage).into())
                .style(Style::default().fg(color))
        })
        .collect();

    let chart = BarChart::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .title(" Provider Breakdown "),
        )
        .data(&bars)
        .bar_width(3)
        .bar_gap(1)
        .max(max_cost as u64);

    frame.render_widget(chart, area);
}

/// Renders the footer with keyboard shortcuts.
fn render_footer(
    frame: &mut Frame,
    area: Rect,
    _state: &CostDashboardState,
    theme: &crate::theme::RadiumTheme,
) {
    let shortcuts = "↑↓: Navigate | Tab: Group | v: View | r: Refresh | 1-3: Date Range | ESC: Back";
    let widget = Paragraph::new(shortcuts)
        .style(Style::default().fg(theme.text_muted))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(theme.border_subtle)),
        );

    frame.render_widget(widget, area);
}

/// Handles keyboard input for the cost dashboard.
pub fn handle_cost_dashboard_key(
    state: &mut CostDashboardState,
    key: KeyCode,
    analytics: &CostAnalytics,
) -> Result<(), String> {
    match key {
        KeyCode::Up => {
            state.move_selection_up();
            Ok(())
        }
        KeyCode::Down => {
            state.move_selection_down();
            Ok(())
        }
        KeyCode::Tab => {
            state.set_grouping(state.grouping_mode.next());
            state.refresh_data(analytics)?;
            Ok(())
        }
        KeyCode::Char('v') => {
            state.toggle_view_mode();
            Ok(())
        }
        KeyCode::Char('r') => {
            state.refresh_data(analytics)?;
            Ok(())
        }
        KeyCode::Char('1') => {
            state.set_date_range(crate::state::DateRangeFilter::Last7Days);
            state.refresh_data(analytics)?;
            Ok(())
        }
        KeyCode::Char('2') => {
            state.set_date_range(crate::state::DateRangeFilter::Last30Days);
            state.refresh_data(analytics)?;
            Ok(())
        }
        KeyCode::Char('3') => {
            state.set_date_range(crate::state::DateRangeFilter::ThisMonth);
            state.refresh_data(analytics)?;
            Ok(())
        }
        _ => Ok(()),
    }
}

/// Helper function to format numbers with commas.
fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

