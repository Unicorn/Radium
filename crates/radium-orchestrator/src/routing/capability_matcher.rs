//! Capability-based matching for skill routing.
//!
//! This module implements semantic matching between task descriptions
//! and skill descriptions using multiple strategies:
//! 1. Keyword/phrase extraction and matching
//! 2. TF-IDF-based similarity scoring
//! 3. Category-based boosting

use std::collections::HashMap;
use super::skill_router::SkillDefinition;

/// Capability matcher for semantic skill routing.
pub struct CapabilityMatcher {
    /// Keyword weights for different domain indicators.
    keyword_weights: HashMap<String, f32>,
}

impl CapabilityMatcher {
    /// Create a new capability matcher with default keyword weights.
    pub fn new() -> Self {
        let mut keyword_weights = HashMap::new();

        // Development keywords
        keyword_weights.insert("code".to_string(), 0.8);
        keyword_weights.insert("review".to_string(), 0.8);
        keyword_weights.insert("refactor".to_string(), 0.8);
        keyword_weights.insert("debug".to_string(), 0.8);
        keyword_weights.insert("test".to_string(), 0.7);
        keyword_weights.insert("api".to_string(), 0.7);
        keyword_weights.insert("database".to_string(), 0.7);

        // Language-specific keywords
        keyword_weights.insert("python".to_string(), 0.9);
        keyword_weights.insert("rust".to_string(), 0.9);
        keyword_weights.insert("javascript".to_string(), 0.9);
        keyword_weights.insert("typescript".to_string(), 0.9);

        // Document keywords
        keyword_weights.insert("pdf".to_string(), 0.95);
        keyword_weights.insert("xlsx".to_string(), 0.95);
        keyword_weights.insert("docx".to_string(), 0.95);
        keyword_weights.insert("pptx".to_string(), 0.95);

        // Creative keywords
        keyword_weights.insert("design".to_string(), 0.75);
        keyword_weights.insert("art".to_string(), 0.75);
        keyword_weights.insert("image".to_string(), 0.7);

        // Business keywords
        keyword_weights.insert("brand".to_string(), 0.7);
        keyword_weights.insert("communication".to_string(), 0.6);
        keyword_weights.insert("meeting".to_string(), 0.6);

        // Productivity keywords
        keyword_weights.insert("documentation".to_string(), 0.7);
        keyword_weights.insert("organize".to_string(), 0.6);
        keyword_weights.insert("workflow".to_string(), 0.6);

        Self { keyword_weights }
    }

    /// Find the best matching skill for a task description.
    ///
    /// Returns (skill, confidence, reasoning) tuple, or None if no skills available.
    pub async fn find_best_match<'a>(
        &self,
        task_description: &str,
        skills: &[&'a SkillDefinition],
    ) -> Option<(&'a SkillDefinition, f32, String)> {
        let mut rankings = self.rank_skills(task_description, skills).await;

        rankings.into_iter().next()
    }

    /// Rank all skills by match quality for a task description.
    ///
    /// Returns Vec<(skill, confidence, reasoning)> sorted by confidence (highest first).
    pub async fn rank_skills<'a>(
        &self,
        task_description: &str,
        skills: &[&'a SkillDefinition],
    ) -> Vec<(&'a SkillDefinition, f32, String)> {
        let mut results: Vec<(&SkillDefinition, f32, String)> = skills
            .iter()
            .map(|skill| {
                let (score, reasoning) = self.calculate_match_score(task_description, skill);
                (*skill, score, reasoning)
            })
            .collect();

        // Sort by score descending
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        results
    }

    /// Calculate match score between task and skill.
    ///
    /// Returns (score: 0.0-1.0, reasoning: String).
    fn calculate_match_score(&self, task: &str, skill: &SkillDefinition) -> (f32, String) {
        let task_lower = task.to_lowercase();
        let desc_lower = skill.description.to_lowercase();

        // Strategy 1: Keyword-based matching
        let keyword_score = self.keyword_match_score(&task_lower, &desc_lower);

        // Strategy 2: Phrase overlap (Jaccard similarity)
        let phrase_score = self.phrase_overlap_score(&task_lower, &desc_lower);

        // Strategy 3: Name match bonus
        let name_score = if task_lower.contains(&skill.name) {
            0.3
        } else {
            0.0
        };

        // Weighted combination
        let base_score = (keyword_score * 0.5) + (phrase_score * 0.4) + (name_score * 0.1);

        // Cap at 1.0
        let final_score = base_score.min(1.0);

        // Generate reasoning
        let reasoning = format!(
            "Matched '{}' (keyword: {:.2}, phrase: {:.2}, name: {:.2})",
            skill.name, keyword_score, phrase_score, name_score
        );

        (final_score, reasoning)
    }

    /// Calculate keyword-based match score.
    fn keyword_match_score(&self, task: &str, skill_desc: &str) -> f32 {
        let task_words: Vec<&str> = task.split_whitespace().collect();
        let desc_words: Vec<&str> = skill_desc.split_whitespace().collect();

        let mut total_weight = 0.0;
        let mut matched_weight = 0.0;

        for (keyword, weight) in &self.keyword_weights {
            total_weight += weight;

            let task_has_keyword = task_words.iter().any(|w| w.contains(keyword.as_str()));
            let desc_has_keyword = desc_words.iter().any(|w| w.contains(keyword.as_str()));

            if task_has_keyword && desc_has_keyword {
                matched_weight += weight;
            }
        }

        if total_weight > 0.0 {
            matched_weight / total_weight
        } else {
            0.0
        }
    }

    /// Calculate phrase overlap score using Jaccard similarity.
    fn phrase_overlap_score(&self, task: &str, skill_desc: &str) -> f32 {
        let task_words: std::collections::HashSet<&str> = task.split_whitespace().collect();
        let desc_words: std::collections::HashSet<&str> = skill_desc.split_whitespace().collect();

        if task_words.is_empty() || desc_words.is_empty() {
            return 0.0;
        }

        let intersection = task_words.intersection(&desc_words).count();
        let union = task_words.union(&desc_words).count();

        if union > 0 {
            intersection as f32 / union as f32
        } else {
            0.0
        }
    }

    /// Add a custom keyword with weight.
    pub fn add_keyword(&mut self, keyword: String, weight: f32) {
        self.keyword_weights.insert(keyword, weight);
    }

    /// Remove a keyword from the matcher.
    pub fn remove_keyword(&mut self, keyword: &str) {
        self.keyword_weights.remove(keyword);
    }
}

impl Default for CapabilityMatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_skill(name: &str, description: &str) -> SkillDefinition {
        SkillDefinition {
            name: name.to_string(),
            description: description.to_string(),
            license: None,
            compatibility: None,
            metadata: None,
            allowed_tools: None,
            skill_path: PathBuf::from("/test/SKILL.md"),
            instructions: None,
        }
    }

    #[test]
    fn test_keyword_matching() {
        let matcher = CapabilityMatcher::new();

        let task = "Review Python code for security issues";
        let desc1 = "Code review for Python projects with security analysis";
        let desc2 = "Create PDF documents from data";

        let score1 = matcher.keyword_match_score(&task.to_lowercase(), &desc1.to_lowercase());
        let score2 = matcher.keyword_match_score(&task.to_lowercase(), &desc2.to_lowercase());

        assert!(score1 > score2, "Python code review should score higher than PDF creation");
    }

    #[test]
    fn test_phrase_overlap() {
        let matcher = CapabilityMatcher::new();

        let task = "create a new API endpoint";
        let desc1 = "API design and endpoint creation";
        let desc2 = "Image processing and enhancement";

        let score1 = matcher.phrase_overlap_score(&task.to_lowercase(), &desc1.to_lowercase());
        let score2 = matcher.phrase_overlap_score(&task.to_lowercase(), &desc2.to_lowercase());

        assert!(score1 > score2, "API task should overlap more with API skill");
    }

    #[tokio::test]
    async fn test_ranking() {
        let matcher = CapabilityMatcher::new();

        let skills = vec![
            create_test_skill("code-review", "Automated code review for pull requests"),
            create_test_skill("pdf", "PDF creation and manipulation"),
            create_test_skill("python-dev", "Python development with modern patterns"),
        ];

        let skill_refs: Vec<&SkillDefinition> = skills.iter().collect();

        let task = "Review my Python code for best practices";
        let rankings = matcher.rank_skills(task, &skill_refs).await;

        // Python-dev or code-review should be top ranked
        let top_skill = rankings[0].0;
        assert!(
            top_skill.name == "python-dev" || top_skill.name == "code-review",
            "Top skill should be python-dev or code-review, got: {}",
            top_skill.name
        );

        // PDF should be last
        let last_skill = rankings.last().unwrap().0;
        assert_eq!(last_skill.name, "pdf");
    }

    #[tokio::test]
    async fn test_exact_name_match() {
        let matcher = CapabilityMatcher::new();

        let skill = create_test_skill("typescript", "TypeScript development");
        let task1 = "Help with typescript error";
        let task2 = "Python debugging";

        let (score1, _) = matcher.calculate_match_score(task1, &skill);
        let (score2, _) = matcher.calculate_match_score(task2, &skill);

        assert!(score1 > score2, "Task mentioning skill name should score higher");
    }

    #[test]
    fn test_custom_keywords() {
        let mut matcher = CapabilityMatcher::new();

        matcher.add_keyword("kubernetes".to_string(), 0.9);

        let task = "Deploy to kubernetes cluster";
        let desc = "Kubernetes deployment automation";

        let score = matcher.keyword_match_score(&task.to_lowercase(), &desc.to_lowercase());
        assert!(score > 0.0, "Custom keyword should be matched");
    }
}
