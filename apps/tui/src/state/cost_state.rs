//! Cost dashboard state management for TUI.

use radium_core::analytics::{CostAnalytics, CostBreakdown, CostHistorySummary, DateRange};
use std::cmp::Ordering;

/// Date range filter for cost queries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DateRangeFilter {
    /// Last 7 days
    Last7Days,
    /// Last 30 days
    Last30Days,
    /// This month
    ThisMonth,
    /// Custom date range
    Custom { start: i64, end: i64 },
}

impl DateRangeFilter {
    /// Converts to DateRange for queries.
    pub fn to_date_range(&self) -> DateRange {
        match self {
            DateRangeFilter::Last7Days => DateRange::last_days(7),
            DateRangeFilter::Last30Days => DateRange::last_days(30),
            DateRangeFilter::ThisMonth => DateRange::this_month(),
            DateRangeFilter::Custom { start, end } => DateRange::new(*start, *end),
        }
    }

    /// Returns display name.
    pub fn display_name(&self) -> String {
        match self {
            DateRangeFilter::Last7Days => "Last 7 Days".to_string(),
            DateRangeFilter::Last30Days => "Last 30 Days".to_string(),
            DateRangeFilter::ThisMonth => "This Month".to_string(),
            DateRangeFilter::Custom { start, end } => {
                format!("Custom: {} - {}", start, end)
            }
        }
    }
}

/// Grouping mode for cost data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupingMode {
    /// Group by requirement
    ByRequirement,
    /// Group by model
    ByModel,
    /// Group by provider
    ByProvider,
    /// Group by day
    ByDay,
    /// Group by week
    ByWeek,
    /// Group by month
    ByMonth,
}

impl GroupingMode {
    /// Returns display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            GroupingMode::ByRequirement => "By Requirement",
            GroupingMode::ByModel => "By Model",
            GroupingMode::ByProvider => "By Provider",
            GroupingMode::ByDay => "By Day",
            GroupingMode::ByWeek => "By Week",
            GroupingMode::ByMonth => "By Month",
        }
    }

    /// Cycles to next grouping mode.
    pub fn next(&self) -> Self {
        match self {
            GroupingMode::ByRequirement => GroupingMode::ByModel,
            GroupingMode::ByModel => GroupingMode::ByProvider,
            GroupingMode::ByProvider => GroupingMode::ByDay,
            GroupingMode::ByDay => GroupingMode::ByWeek,
            GroupingMode::ByWeek => GroupingMode::ByMonth,
            GroupingMode::ByMonth => GroupingMode::ByRequirement,
        }
    }
}

/// View mode for displaying data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    /// Show costs
    Cost,
    /// Show token usage
    Tokens,
}

impl ViewMode {
    /// Toggles between Cost and Tokens.
    pub fn toggle(&self) -> Self {
        match self {
            ViewMode::Cost => ViewMode::Tokens,
            ViewMode::Tokens => ViewMode::Cost,
        }
    }

    /// Returns display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            ViewMode::Cost => "Cost",
            ViewMode::Tokens => "Tokens",
        }
    }
}

/// Sort column for table display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    /// Sort by name/key
    Name,
    /// Sort by cost
    Cost,
    /// Sort by tokens
    Tokens,
    /// Sort by count
    Count,
}

/// Display row for table rendering.
#[derive(Debug, Clone)]
pub struct DisplayRow {
    /// Row key/name
    pub key: String,
    /// Cost value
    pub cost: f64,
    /// Token value
    pub tokens: u64,
    /// Event count
    pub count: u64,
}

/// Cost dashboard state.
#[derive(Debug, Clone)]
pub struct CostDashboardState {
    /// Current date range filter
    pub date_range_filter: DateRangeFilter,
    /// Current grouping mode
    pub grouping_mode: GroupingMode,
    /// Current view mode
    pub view_mode: ViewMode,
    /// Sort column
    pub sort_column: SortColumn,
    /// Sort ascending (true) or descending (false)
    pub sort_ascending: bool,
    /// Loaded cost summary
    pub summary: Option<CostHistorySummary>,
    /// Loaded breakdown data
    pub breakdown_data: Vec<CostBreakdown>,
    /// Whether data is currently loading
    pub loading: bool,
    /// Selected row index (for keyboard navigation)
    pub selected_row: usize,
    /// Error message if any
    pub error: Option<String>,
}

impl CostDashboardState {
    /// Creates a new cost dashboard state with default values.
    pub fn new() -> Self {
        Self {
            date_range_filter: DateRangeFilter::Last7Days,
            grouping_mode: GroupingMode::ByRequirement,
            view_mode: ViewMode::Cost,
            sort_column: SortColumn::Cost,
            sort_ascending: false,
            summary: None,
            breakdown_data: Vec::new(),
            loading: false,
            selected_row: 0,
            error: None,
        }
    }

    /// Sets the date range filter.
    pub fn set_date_range(&mut self, filter: DateRangeFilter) {
        self.date_range_filter = filter;
        self.selected_row = 0; // Reset selection when filter changes
    }

    /// Sets the grouping mode.
    pub fn set_grouping(&mut self, mode: GroupingMode) {
        self.grouping_mode = mode;
        self.selected_row = 0; // Reset selection when grouping changes
    }

    /// Toggles the view mode.
    pub fn toggle_view_mode(&mut self) {
        self.view_mode = self.view_mode.toggle();
    }

    /// Sets the sort column, toggling direction if same column.
    pub fn set_sort(&mut self, column: SortColumn) {
        if self.sort_column == column {
            self.sort_ascending = !self.sort_ascending;
        } else {
            self.sort_column = column;
            self.sort_ascending = false; // Default to descending
        }
    }

    /// Refreshes data from analytics service.
    pub fn refresh_data(&mut self, analytics: &CostAnalytics) -> Result<(), String> {
        self.loading = true;
        self.error = None;

        let range = self.date_range_filter.to_date_range();

        // Get summary
        let summary = analytics
            .total_cost_summary(&range)
            .map_err(|e| format!("Failed to load cost summary: {}", e))?;

        // Get breakdown based on grouping mode
        let breakdown = match self.grouping_mode {
            GroupingMode::ByRequirement => {
                analytics.group_by_requirement(&range).map_err(|e| {
                    format!("Failed to load requirement breakdown: {}", e)
                })?
            }
            GroupingMode::ByModel => {
                analytics.group_by_model(&range).map_err(|e| {
                    format!("Failed to load model breakdown: {}", e)
                })?
            }
            GroupingMode::ByProvider => {
                analytics.group_by_provider(&range).map_err(|e| {
                    format!("Failed to load provider breakdown: {}", e)
                })?
            }
            GroupingMode::ByDay => {
                analytics
                    .group_by_time_period(&range, radium_core::analytics::TimePeriod::Day)
                    .map_err(|e| format!("Failed to load daily breakdown: {}", e))?
            }
            GroupingMode::ByWeek => {
                analytics
                    .group_by_time_period(&range, radium_core::analytics::TimePeriod::Week)
                    .map_err(|e| format!("Failed to load weekly breakdown: {}", e))?
            }
            GroupingMode::ByMonth => {
                analytics
                    .group_by_time_period(&range, radium_core::analytics::TimePeriod::Month)
                    .map_err(|e| format!("Failed to load monthly breakdown: {}", e))?
            }
        };

        self.summary = Some(summary);
        self.breakdown_data = breakdown;
        self.loading = false;

        // Reset selection if out of bounds
        if self.selected_row >= self.breakdown_data.len() {
            self.selected_row = 0;
        }

        Ok(())
    }

    /// Gets display data formatted for table rendering.
    pub fn get_display_data(&self) -> Vec<DisplayRow> {
        let mut rows: Vec<DisplayRow> = self
            .breakdown_data
            .iter()
            .map(|b| DisplayRow {
                key: b.key.clone(),
                cost: b.total_cost,
                tokens: b.total_tokens,
                count: b.event_count,
            })
            .collect();

        // Sort based on current sort settings
        rows.sort_by(|a, b| {
            let ordering = match self.sort_column {
                SortColumn::Name => a.key.cmp(&b.key),
                SortColumn::Cost => {
                    a.cost.partial_cmp(&b.cost).unwrap_or(Ordering::Equal)
                }
                SortColumn::Tokens => a.tokens.cmp(&b.tokens),
                SortColumn::Count => a.count.cmp(&b.count),
            };

            if self.sort_ascending {
                ordering
            } else {
                ordering.reverse()
            }
        });

        rows
    }

    /// Moves selection up.
    pub fn move_selection_up(&mut self) {
        if self.selected_row > 0 {
            self.selected_row -= 1;
        }
    }

    /// Moves selection down.
    pub fn move_selection_down(&mut self) {
        let max_row = self.breakdown_data.len().saturating_sub(1);
        if self.selected_row < max_row {
            self.selected_row += 1;
        }
    }

    /// Gets top N expensive requirements from summary.
    pub fn get_top_requirements(&self, limit: usize) -> Vec<&CostBreakdown> {
        if let Some(ref summary) = self.summary {
            summary
                .breakdown_by_requirement
                .iter()
                .take(limit)
                .collect()
        } else {
            Vec::new()
        }
    }
}

impl Default for CostDashboardState {
    fn default() -> Self {
        Self::new()
    }
}

