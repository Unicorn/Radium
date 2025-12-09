//! Budget analytics view for displaying budget trends, forecasts, and anomalies.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Gauge, Paragraph, Row, Table, Tabs},
};
use crate::theme::get_theme;

/// Tab selection for budget analytics view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalyticsTab {
    Overview,
    Trends,
    Forecast,
    Anomalies,
    Recommendations,
}

impl AnalyticsTab {
    pub fn next(&self) -> Self {
        match self {
            Self::Overview => Self::Trends,
            Self::Trends => Self::Forecast,
            Self::Forecast => Self::Anomalies,
            Self::Anomalies => Self::Recommendations,
            Self::Recommendations => Self::Overview,
        }
    }

    pub fn previous(&self) -> Self {
        match self {
            Self::Overview => Self::Recommendations,
            Self::Trends => Self::Overview,
            Self::Forecast => Self::Trends,
            Self::Anomalies => Self::Forecast,
            Self::Recommendations => Self::Anomalies,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Trends => "Trends",
            Self::Forecast => "Forecast",
            Self::Anomalies => "Anomalies",
            Self::Recommendations => "Recommendations",
        }
    }
}

/// Budget analytics view state.
pub struct BudgetAnalyticsView {
    /// Selected tab.
    pub selected_tab: AnalyticsTab,
    /// Budget analytics data (will be populated from BudgetManager).
    pub analytics_data: Option<String>, // Placeholder - would use actual BudgetAnalytics type
}

impl BudgetAnalyticsView {
    pub fn new() -> Self {
        Self {
            selected_tab: AnalyticsTab::Overview,
            analytics_data: None,
        }
    }

    /// Renders the budget analytics view.
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let theme = get_theme();

        // Split into tabs and content
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        // Render tabs
        let tab_titles = vec![
            AnalyticsTab::Overview.as_str(),
            AnalyticsTab::Trends.as_str(),
            AnalyticsTab::Forecast.as_str(),
            AnalyticsTab::Anomalies.as_str(),
            AnalyticsTab::Recommendations.as_str(),
        ];

        let tabs = Tabs::new(tab_titles)
            .block(Block::default().borders(Borders::ALL).title(" Budget Analytics "))
            .select(match self.selected_tab {
                AnalyticsTab::Overview => 0,
                AnalyticsTab::Trends => 1,
                AnalyticsTab::Forecast => 2,
                AnalyticsTab::Anomalies => 3,
                AnalyticsTab::Recommendations => 4,
            })
            .style(Style::default().fg(theme.text_muted))
            .highlight_style(Style::default().fg(theme.primary).bold());

        frame.render_widget(tabs, chunks[0]);

        // Render content based on selected tab
        match self.selected_tab {
            AnalyticsTab::Overview => self.render_overview(frame, chunks[1], &theme),
            AnalyticsTab::Trends => self.render_trends(frame, chunks[1], &theme),
            AnalyticsTab::Forecast => self.render_forecast(frame, chunks[1], &theme),
            AnalyticsTab::Anomalies => self.render_anomalies(frame, chunks[1], &theme),
            AnalyticsTab::Recommendations => self.render_recommendations(frame, chunks[1], &theme),
        }
    }

    fn render_overview(&self, frame: &mut Frame, area: Rect, theme: &crate::theme::RadiumTheme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),
                Constraint::Length(6),
                Constraint::Min(0),
            ])
            .split(area);

        // Budget gauge
        let budget_text = Paragraph::new("Budget: $100.00  Spent: $67.50 (67.5%)  Remaining: $32.50")
            .block(Block::default().borders(Borders::ALL).title(" Budget Status "))
            .style(Style::default().fg(theme.info));

        let utilization = 67.5;
        let gauge_color = if utilization < 70.0 {
            theme.success
        } else if utilization < 90.0 {
            theme.warning
        } else {
            theme.error
        };

        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title(" Utilization "))
            .gauge_style(Style::default().fg(gauge_color).bg(theme.bg_panel))
            .percent(utilization as u16)
            .label(format!("{:.1}%", utilization));

        frame.render_widget(budget_text, chunks[0]);
        frame.render_widget(gauge, chunks[1]);

        // Recent anomalies summary
        let anomalies_text = Paragraph::new("Recent Anomalies:\n• REQ-198: $12.50 (z=3.2) - Token spike\n• REQ-195: $9.80 (z=2.4) - Legitimate complexity")
            .block(Block::default().borders(Borders::ALL).title(" Recent Anomalies "))
            .style(Style::default().fg(theme.text_muted));

        frame.render_widget(anomalies_text, chunks[2]);
    }

    fn render_trends(&self, frame: &mut Frame, area: Rect, theme: &crate::theme::RadiumTheme) {
        let text = Paragraph::new(
            "30-Day Spend Trend:\n\n\
            $100 ┤                                          ╭───\n\
                 │                                    ╭─────╯\n\
             $50 ┤                          ╭─────────╯\n\
                 │                ╭─────────╯\n\
              $0 └────────────────┴──────────────────────────────\n\
                  Dec 1        Dec 15        Dec 30\n\n\
            (ASCII chart - would use ratatui Chart widget in production)"
        )
        .block(Block::default().borders(Borders::ALL).title(" Spend Trends "))
        .style(Style::default().fg(theme.info));

        frame.render_widget(text, area);
    }

    fn render_forecast(&self, frame: &mut Frame, area: Rect, theme: &crate::theme::RadiumTheme) {
        let text = Paragraph::new(
            "Budget Forecast:\n\n\
            Current velocity: $4.50/day\n\
            Projected exhaustion: Dec 15, 2025 (7 days)\n\
            Confidence interval: Dec 14 - Dec 17 (95%)\n\
            Velocity trend: ↑ 35% vs last week\n\n\
            ⚠️  WARNING: Budget will run out in 7 days at current rate"
        )
        .block(Block::default().borders(Borders::ALL).title(" Forecast "))
        .style(Style::default().fg(theme.warning));

        frame.render_widget(text, area);
    }

    fn render_anomalies(&self, frame: &mut Frame, area: Rect, theme: &crate::theme::RadiumTheme) {
        let rows = vec![
            Row::new(vec!["REQ-198", "$12.50", "3.2", "MAJOR", "TokenSpike"]),
            Row::new(vec!["REQ-195", "$9.80", "2.4", "MINOR", "LegitimateComplexity"]),
        ];

        let table = Table::new(
                rows,
                &[
                    Constraint::Percentage(25),
                    Constraint::Percentage(20),
                    Constraint::Percentage(15),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                ]
            )
            .block(Block::default().borders(Borders::ALL).title(" Cost Anomalies "))
            .header(Row::new(vec!["Requirement", "Cost", "Z-Score", "Severity", "Category"])
                .style(Style::default().fg(theme.primary).bold()))
            .style(Style::default().fg(theme.text));

        frame.render_widget(table, area);
    }

    fn render_recommendations(&self, frame: &mut Frame, area: Rect, theme: &crate::theme::RadiumTheme) {
        let text = Paragraph::new(
            "Budget Recommendations:\n\n\
            • Switch to eco models to extend runway by 10 days\n\
            • Reduce requirement scope: Current plan exceeds budget by 15%\n\
            • Schedule expensive requirements after budget renewal\n\
            • Increase budget allocation: Current pace requires $50 more"
        )
        .block(Block::default().borders(Borders::ALL).title(" Recommendations "))
        .style(Style::default().fg(theme.info));

        frame.render_widget(text, area);
    }
}

