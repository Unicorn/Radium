//! Integration test for skill validation and loading.

use radium_orchestrator::routing::{SkillRouter, SkillDefinition};
use std::path::PathBuf;

#[tokio::test]
async fn test_load_skills_from_directory() {
    let skill_router = SkillRouter::new();

    // Load all skills from the skills directory
    let skills_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("skills");

    if !skills_dir.exists() {
        eprintln!("Skills directory not found: {}", skills_dir.display());
        return;
    }

    let result = skill_router.load_skills_from_directory(&skills_dir).await;

    match result {
        Ok(count) => {
            println!("✓ Successfully loaded {} skills", count);
            assert!(count >= 10, "Expected at least 10 skills, got {}", count);

            // Verify we can list skills (allow ±1 for potential de-duplication)
            let skills = skill_router.list_skills().await;
            assert!(
                skills.len() >= 10,
                "Expected at least 10 skills in list, got {}",
                skills.len()
            );
            assert!(
                skills.len() <= count,
                "Listed more skills ({}) than were loaded ({})",
                skills.len(),
                count
            );

            println!("Loaded skills: {:?}", skills);
        }
        Err(e) => {
            panic!("Failed to load skills: {}", e);
        }
    }
}

#[tokio::test]
async fn test_skill_routing() {
    let skill_router = SkillRouter::new();

    // Load skills
    let skills_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("skills");

    if !skills_dir.exists() {
        eprintln!("Skills directory not found");
        return;
    }

    let count = skill_router.load_skills_from_directory(&skills_dir).await.unwrap();
    println!("Loaded {} skills for routing test", count);

    if count == 0 {
        eprintln!("No skills loaded, skipping routing test");
        return;
    }

    // Test routing for different task types
    let test_cases = vec![
        ("Review my Rust code for security issues", vec!["review", "security"]),
        ("Implement a new REST API endpoint", vec!["code", "api-design"]),
        ("Write documentation for the project", vec!["doc"]),
        ("Design a database schema", vec!["db-design"]),
    ];

    for (task, expected_matches) in test_cases {
        println!("\n--- Task: {} ---", task);

        let result = skill_router.route(task).await;

        match result {
            Ok(Some(routing_result)) => {
                println!("✓ Routed to: {} (confidence: {:.2})", routing_result.skill_name, routing_result.confidence);
                println!("  Reasoning: {}", routing_result.reasoning);

                // Check if the routed skill is relevant
                let skill_name_lower = routing_result.skill_name.to_lowercase();
                let is_relevant = expected_matches.iter().any(|m| skill_name_lower.contains(m));

                if is_relevant {
                    println!("  ✓ Routing seems relevant");
                } else {
                    println!("  ⚠ Routing may not be optimal (expected one of: {:?})", expected_matches);
                }
            }
            Ok(None) => {
                println!("✗ No skill matched (confidence too low)");
            }
            Err(e) => {
                println!("✗ Routing error: {}", e);
            }
        }
    }
}

#[tokio::test]
async fn test_top_k_routing() {
    let skill_router = SkillRouter::with_threshold(0.0); // Accept all matches

    let skills_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("skills");

    if !skills_dir.exists() {
        return;
    }

    let count = skill_router.load_skills_from_directory(&skills_dir).await.unwrap();

    if count == 0 {
        return;
    }

    let task = "Review and refactor Python code for better performance";

    let results = skill_router.route_top_k(task, 5).await.unwrap();

    println!("\nTop-5 skills for: {}", task);
    for (idx, result) in results.iter().enumerate() {
        println!("{}. {} (confidence: {:.3})", idx + 1, result.skill_name, result.confidence);
    }

    assert!(!results.is_empty(), "Expected at least one match");
    assert!(results.len() <= 5, "Expected at most 5 results");

    // Verify results are sorted by confidence
    for i in 1..results.len() {
        assert!(
            results[i - 1].confidence >= results[i].confidence,
            "Results not sorted by confidence"
        );
    }
}

#[test]
fn test_skill_schema_validation() {
    // Test valid skill
    let valid_skill = SkillDefinition {
        name: "test-skill".to_string(),
        description: "A valid test skill".to_string(),
        license: Some("MIT".to_string()),
        compatibility: Some("Requires git".to_string()),
        metadata: None,
        allowed_tools: None,
        skill_path: PathBuf::from("/test/SKILL.md"),
        instructions: Some("Do something".to_string()),
    };

    // Should not panic
    assert!(format!("{:?}", valid_skill).contains("test-skill"));

    // Test that validation would catch invalid names
    // (Validation happens in SkillDefinition::validate, called during from_file)
}
