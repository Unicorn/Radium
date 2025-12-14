//! Compilation Profiler
//!
//! Stage-based profiling for workflow compilation to identify
//! bottlenecks and track performance metrics.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;
use std::time::{Duration, Instant};

/// Compilation stages for profiling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CompilationStage {
    /// Parsing the workflow JSON/YAML
    Parsing,
    /// Validating the workflow schema
    Validation,
    /// Type checking and inference
    TypeChecking,
    /// Optimizing the workflow graph
    Optimization,
    /// Generating TypeScript code
    CodeGeneration,
    /// Formatting the output
    Formatting,
    /// Writing to output files
    Output,
}

impl CompilationStage {
    /// Get all stages in execution order
    pub fn all() -> &'static [CompilationStage] {
        &[
            CompilationStage::Parsing,
            CompilationStage::Validation,
            CompilationStage::TypeChecking,
            CompilationStage::Optimization,
            CompilationStage::CodeGeneration,
            CompilationStage::Formatting,
            CompilationStage::Output,
        ]
    }

    /// Get the display name for this stage
    pub fn name(&self) -> &'static str {
        match self {
            CompilationStage::Parsing => "Parsing",
            CompilationStage::Validation => "Validation",
            CompilationStage::TypeChecking => "Type Checking",
            CompilationStage::Optimization => "Optimization",
            CompilationStage::CodeGeneration => "Code Generation",
            CompilationStage::Formatting => "Formatting",
            CompilationStage::Output => "Output",
        }
    }
}

/// Timing data for a single stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageTiming {
    /// Duration of this stage
    pub duration: Duration,
    /// Percentage of total time
    pub percentage: f64,
}

/// Profile of a single compilation run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationProfile {
    /// Workflow identifier
    pub workflow_id: String,
    /// Timing for each stage
    pub stages: HashMap<CompilationStage, StageTiming>,
    /// Total compilation time
    pub total_duration: Duration,
    /// When the compilation started
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// Size of input (bytes)
    pub input_size_bytes: usize,
    /// Size of output (bytes)
    pub output_size_bytes: usize,
}

impl CompilationProfile {
    /// Get the slowest stage
    pub fn slowest_stage(&self) -> Option<(CompilationStage, &StageTiming)> {
        self.stages
            .iter()
            .max_by(|a, b| a.1.duration.cmp(&b.1.duration))
            .map(|(stage, timing)| (*stage, timing))
    }

    /// Get stages sorted by duration (slowest first)
    pub fn stages_by_duration(&self) -> Vec<(CompilationStage, &StageTiming)> {
        let mut stages: Vec<_> = self.stages.iter().map(|(s, t)| (*s, t)).collect();
        stages.sort_by(|a, b| b.1.duration.cmp(&a.1.duration));
        stages
    }

    /// Calculate throughput in bytes per second
    pub fn throughput_bytes_per_sec(&self) -> f64 {
        let secs = self.total_duration.as_secs_f64();
        if secs > 0.0 {
            self.output_size_bytes as f64 / secs
        } else {
            0.0
        }
    }
}

/// Builder for creating compilation profiles
pub struct ProfileBuilder {
    workflow_id: String,
    started_at: chrono::DateTime<chrono::Utc>,
    stage_start: Option<Instant>,
    current_stage: Option<CompilationStage>,
    stages: HashMap<CompilationStage, Duration>,
    input_size_bytes: usize,
    output_size_bytes: usize,
}

impl ProfileBuilder {
    /// Create a new profile builder
    pub fn new(workflow_id: impl Into<String>) -> Self {
        Self {
            workflow_id: workflow_id.into(),
            started_at: chrono::Utc::now(),
            stage_start: None,
            current_stage: None,
            stages: HashMap::new(),
            input_size_bytes: 0,
            output_size_bytes: 0,
        }
    }

    /// Set the input size
    pub fn with_input_size(mut self, bytes: usize) -> Self {
        self.input_size_bytes = bytes;
        self
    }

    /// Start timing a stage
    pub fn start_stage(&mut self, stage: CompilationStage) {
        // End previous stage if any
        self.end_current_stage();

        self.current_stage = Some(stage);
        self.stage_start = Some(Instant::now());
    }

    /// End the current stage
    pub fn end_current_stage(&mut self) {
        if let (Some(stage), Some(start)) = (self.current_stage.take(), self.stage_start.take()) {
            let duration = start.elapsed();
            self.stages.insert(stage, duration);
        }
    }

    /// Set the output size
    pub fn set_output_size(&mut self, bytes: usize) {
        self.output_size_bytes = bytes;
    }

    /// Build the final profile
    pub fn build(mut self) -> CompilationProfile {
        // End any remaining stage
        self.end_current_stage();

        // Calculate total duration
        let total_duration: Duration = self.stages.values().copied().sum();
        let total_nanos = total_duration.as_nanos() as f64;

        // Convert to StageTiming with percentages
        let stages = self
            .stages
            .into_iter()
            .map(|(stage, duration)| {
                let percentage = if total_nanos > 0.0 {
                    (duration.as_nanos() as f64 / total_nanos) * 100.0
                } else {
                    0.0
                };
                (stage, StageTiming { duration, percentage })
            })
            .collect();

        CompilationProfile {
            workflow_id: self.workflow_id,
            stages,
            total_duration,
            started_at: self.started_at,
            input_size_bytes: self.input_size_bytes,
            output_size_bytes: self.output_size_bytes,
        }
    }
}

/// Aggregated statistics across multiple compilations
#[derive(Debug, Default)]
pub struct ProfilerStats {
    /// Total compilations profiled
    pub total_compilations: AtomicU64,
    /// Total time spent compiling (microseconds)
    pub total_time_us: AtomicU64,
    /// Total input bytes processed
    pub total_input_bytes: AtomicU64,
    /// Total output bytes generated
    pub total_output_bytes: AtomicU64,
}

/// Snapshot of profiler statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilerStatsSnapshot {
    /// Total compilations profiled
    pub total_compilations: u64,
    /// Total time spent compiling
    pub total_duration: Duration,
    /// Total input bytes processed
    pub total_input_bytes: u64,
    /// Total output bytes generated
    pub total_output_bytes: u64,
    /// Average compilation time
    pub avg_compilation_time: Duration,
    /// Average input size
    pub avg_input_size: f64,
    /// Average output size
    pub avg_output_size: f64,
    /// Average throughput (output bytes per second)
    pub avg_throughput: f64,
}

/// Compilation profiler for tracking performance
pub struct CompilationProfiler {
    /// Recent profiles (ring buffer)
    profiles: RwLock<Vec<CompilationProfile>>,
    /// Maximum profiles to keep
    max_profiles: usize,
    /// Aggregated statistics
    stats: ProfilerStats,
}

impl CompilationProfiler {
    /// Create a new profiler
    pub fn new(max_profiles: usize) -> Self {
        Self {
            profiles: RwLock::new(Vec::with_capacity(max_profiles)),
            max_profiles,
            stats: ProfilerStats::default(),
        }
    }

    /// Record a compilation profile
    pub fn record(&self, profile: CompilationProfile) {
        // Update stats
        self.stats
            .total_compilations
            .fetch_add(1, Ordering::Relaxed);
        self.stats.total_time_us.fetch_add(
            profile.total_duration.as_micros() as u64,
            Ordering::Relaxed,
        );
        self.stats
            .total_input_bytes
            .fetch_add(profile.input_size_bytes as u64, Ordering::Relaxed);
        self.stats
            .total_output_bytes
            .fetch_add(profile.output_size_bytes as u64, Ordering::Relaxed);

        // Store profile
        let mut profiles = self.profiles.write().unwrap();
        if profiles.len() >= self.max_profiles {
            profiles.remove(0); // Remove oldest
        }
        profiles.push(profile);
    }

    /// Get recent profiles
    pub fn recent_profiles(&self) -> Vec<CompilationProfile> {
        self.profiles.read().unwrap().clone()
    }

    /// Get the most recent profile
    pub fn last_profile(&self) -> Option<CompilationProfile> {
        self.profiles.read().unwrap().last().cloned()
    }

    /// Get aggregated statistics
    pub fn stats(&self) -> ProfilerStatsSnapshot {
        let total_compilations = self.stats.total_compilations.load(Ordering::Relaxed);
        let total_time_us = self.stats.total_time_us.load(Ordering::Relaxed);
        let total_input_bytes = self.stats.total_input_bytes.load(Ordering::Relaxed);
        let total_output_bytes = self.stats.total_output_bytes.load(Ordering::Relaxed);

        let total_duration = Duration::from_micros(total_time_us);

        let (avg_compilation_time, avg_input_size, avg_output_size, avg_throughput) =
            if total_compilations > 0 {
                let avg_time = Duration::from_micros(total_time_us / total_compilations);
                let avg_in = total_input_bytes as f64 / total_compilations as f64;
                let avg_out = total_output_bytes as f64 / total_compilations as f64;
                let secs = total_duration.as_secs_f64();
                let throughput = if secs > 0.0 {
                    total_output_bytes as f64 / secs
                } else {
                    0.0
                };
                (avg_time, avg_in, avg_out, throughput)
            } else {
                (Duration::ZERO, 0.0, 0.0, 0.0)
            };

        ProfilerStatsSnapshot {
            total_compilations,
            total_duration,
            total_input_bytes,
            total_output_bytes,
            avg_compilation_time,
            avg_input_size,
            avg_output_size,
            avg_throughput,
        }
    }

    /// Get average timing per stage across all profiles
    pub fn average_stage_timings(&self) -> HashMap<CompilationStage, Duration> {
        let profiles = self.profiles.read().unwrap();
        if profiles.is_empty() {
            return HashMap::new();
        }

        let mut totals: HashMap<CompilationStage, Duration> = HashMap::new();
        let mut counts: HashMap<CompilationStage, usize> = HashMap::new();

        for profile in profiles.iter() {
            for (stage, timing) in &profile.stages {
                *totals.entry(*stage).or_default() += timing.duration;
                *counts.entry(*stage).or_default() += 1;
            }
        }

        totals
            .into_iter()
            .map(|(stage, total)| {
                let count = counts.get(&stage).copied().unwrap_or(1);
                (stage, total / count as u32)
            })
            .collect()
    }

    /// Clear all profiles and reset stats
    pub fn clear(&self) {
        let mut profiles = self.profiles.write().unwrap();
        profiles.clear();
        self.stats
            .total_compilations
            .store(0, Ordering::Relaxed);
        self.stats.total_time_us.store(0, Ordering::Relaxed);
        self.stats.total_input_bytes.store(0, Ordering::Relaxed);
        self.stats.total_output_bytes.store(0, Ordering::Relaxed);
    }
}

impl Default for CompilationProfiler {
    fn default() -> Self {
        Self::new(100) // Keep last 100 profiles
    }
}

/// RAII guard for timing a stage
pub struct StageTimer<'a> {
    builder: &'a mut ProfileBuilder,
    stage: CompilationStage,
    start: Instant,
}

impl<'a> StageTimer<'a> {
    /// Create a new stage timer
    pub fn new(builder: &'a mut ProfileBuilder, stage: CompilationStage) -> Self {
        Self {
            builder,
            stage,
            start: Instant::now(),
        }
    }
}

impl Drop for StageTimer<'_> {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        self.builder.stages.insert(self.stage, duration);
    }
}

/// Macro for timing a stage
#[macro_export]
macro_rules! time_stage {
    ($builder:expr, $stage:expr, $body:expr) => {{
        $builder.start_stage($stage);
        let result = $body;
        $builder.end_current_stage();
        result
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_profile_builder_basic() {
        let mut builder = ProfileBuilder::new("test-workflow").with_input_size(1000);

        builder.start_stage(CompilationStage::Parsing);
        thread::sleep(Duration::from_millis(10));
        builder.end_current_stage();

        builder.start_stage(CompilationStage::CodeGeneration);
        thread::sleep(Duration::from_millis(20));
        builder.end_current_stage();

        builder.set_output_size(2000);

        let profile = builder.build();

        assert_eq!(profile.workflow_id, "test-workflow");
        assert_eq!(profile.input_size_bytes, 1000);
        assert_eq!(profile.output_size_bytes, 2000);
        assert_eq!(profile.stages.len(), 2);
        assert!(profile.stages.contains_key(&CompilationStage::Parsing));
        assert!(profile.stages.contains_key(&CompilationStage::CodeGeneration));
    }

    #[test]
    fn test_profile_percentages() {
        let mut builder = ProfileBuilder::new("test");

        // Manually insert stages with known durations
        builder
            .stages
            .insert(CompilationStage::Parsing, Duration::from_millis(25));
        builder
            .stages
            .insert(CompilationStage::CodeGeneration, Duration::from_millis(75));

        let profile = builder.build();

        // Parsing should be ~25% and CodeGeneration ~75%
        let parsing = profile.stages.get(&CompilationStage::Parsing).unwrap();
        let codegen = profile
            .stages
            .get(&CompilationStage::CodeGeneration)
            .unwrap();

        assert!((parsing.percentage - 25.0).abs() < 1.0);
        assert!((codegen.percentage - 75.0).abs() < 1.0);
    }

    #[test]
    fn test_slowest_stage() {
        let mut builder = ProfileBuilder::new("test");
        builder
            .stages
            .insert(CompilationStage::Parsing, Duration::from_millis(10));
        builder
            .stages
            .insert(CompilationStage::Validation, Duration::from_millis(50));
        builder
            .stages
            .insert(CompilationStage::CodeGeneration, Duration::from_millis(30));

        let profile = builder.build();

        let (slowest, _) = profile.slowest_stage().unwrap();
        assert_eq!(slowest, CompilationStage::Validation);
    }

    #[test]
    fn test_profiler_recording() {
        let profiler = CompilationProfiler::new(10);

        let mut builder = ProfileBuilder::new("workflow-1").with_input_size(100);
        builder
            .stages
            .insert(CompilationStage::Parsing, Duration::from_millis(10));
        builder.set_output_size(200);
        profiler.record(builder.build());

        let stats = profiler.stats();
        assert_eq!(stats.total_compilations, 1);
        assert_eq!(stats.total_input_bytes, 100);
        assert_eq!(stats.total_output_bytes, 200);
    }

    #[test]
    fn test_profiler_max_profiles() {
        let profiler = CompilationProfiler::new(3);

        // Add 5 profiles
        for i in 0..5 {
            let mut builder = ProfileBuilder::new(format!("workflow-{}", i));
            builder
                .stages
                .insert(CompilationStage::Parsing, Duration::from_millis(1));
            profiler.record(builder.build());
        }

        // Should only keep last 3
        let profiles = profiler.recent_profiles();
        assert_eq!(profiles.len(), 3);
        assert_eq!(profiles[0].workflow_id, "workflow-2");
        assert_eq!(profiles[1].workflow_id, "workflow-3");
        assert_eq!(profiles[2].workflow_id, "workflow-4");
    }

    #[test]
    fn test_average_stage_timings() {
        let profiler = CompilationProfiler::new(10);

        // Add two profiles with different timings
        let mut builder1 = ProfileBuilder::new("w1");
        builder1
            .stages
            .insert(CompilationStage::Parsing, Duration::from_millis(10));
        profiler.record(builder1.build());

        let mut builder2 = ProfileBuilder::new("w2");
        builder2
            .stages
            .insert(CompilationStage::Parsing, Duration::from_millis(30));
        profiler.record(builder2.build());

        let averages = profiler.average_stage_timings();
        let avg_parsing = averages.get(&CompilationStage::Parsing).unwrap();

        // Average should be 20ms
        assert!(avg_parsing.as_millis() >= 19 && avg_parsing.as_millis() <= 21);
    }

    #[test]
    fn test_profiler_clear() {
        let profiler = CompilationProfiler::new(10);

        let mut builder = ProfileBuilder::new("test");
        builder
            .stages
            .insert(CompilationStage::Parsing, Duration::from_millis(10));
        profiler.record(builder.build());

        assert_eq!(profiler.stats().total_compilations, 1);

        profiler.clear();

        assert_eq!(profiler.stats().total_compilations, 0);
        assert!(profiler.recent_profiles().is_empty());
    }

    #[test]
    fn test_compilation_stage_names() {
        assert_eq!(CompilationStage::Parsing.name(), "Parsing");
        assert_eq!(CompilationStage::TypeChecking.name(), "Type Checking");
        assert_eq!(CompilationStage::CodeGeneration.name(), "Code Generation");
    }

    #[test]
    fn test_all_stages() {
        let stages = CompilationStage::all();
        assert_eq!(stages.len(), 7);
        assert_eq!(stages[0], CompilationStage::Parsing);
        assert_eq!(stages[6], CompilationStage::Output);
    }

    #[test]
    fn test_throughput_calculation() {
        let mut builder = ProfileBuilder::new("test");
        builder
            .stages
            .insert(CompilationStage::Parsing, Duration::from_secs(1));
        builder.output_size_bytes = 1000;
        let profile = builder.build();

        let throughput = profile.throughput_bytes_per_sec();
        assert!((throughput - 1000.0).abs() < 10.0); // ~1000 bytes/sec
    }
}
