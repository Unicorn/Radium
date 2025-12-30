//! Integration tests for process monitoring, error detection, and routing.
//!
//! This test verifies the end-to-end flow:
//! 1. Error log is classified by severity
//! 2. High-severity errors are routed to ErrorRouter
//! 3. Fix proposals are generated and queued for approval
//! 4. Proposals can be approved and applied

use radium_core::monitoring::{ErrorClassifier, ErrorSeverity};
use radium_orchestrator::error_router::{ErrorRouter, ErrorContext, ProposalStatus};

fn create_error_context(process_id: &str, error_line: &str, agent: &str) -> ErrorContext {
    ErrorContext {
        process_id: process_id.to_string(),
        error_lines: vec![error_line.to_string()],
        severity: "high".to_string(),
        error_type: "compilation_error".to_string(),
        confidence: 0.85,
        suggested_agent: Some(agent.to_string()),
        metadata: serde_json::json!({
            "command": "test",
            "working_dir": "/test"
        }),
    }
}

#[tokio::test]
async fn test_error_detection_and_routing_flow() {
    // Step 1: Create error classifier
    let mut classifier = ErrorClassifier::new();

    // Step 2: Simulate error log lines from a process
    let rust_compile_error = "error[E0308]: mismatched types";
    let panic_error = "thread 'main' panicked at 'index out of bounds'";
    let warning = "warning: unused variable `foo`";

    // Step 3: Classify errors (verify classifier works)
    let result1 = classifier.classify(rust_compile_error, None);
    let result2 = classifier.classify(panic_error, None);
    let result3 = classifier.classify(warning, None);

    // Verify errors have different severities
    assert!(result1.severity != result2.severity || result2.severity != result3.severity);

    // Step 4: Route high-severity error to ErrorRouter
    let router = ErrorRouter::new();
    let error_context = create_error_context(
        "test-process-123",
        rust_compile_error,
        "rust-expert-agent",
    );

    let proposal_id = router.route_error(error_context).await.unwrap();

    // Step 5: Verify proposal was created
    let proposals = router.get_all_proposals().await;
    assert_eq!(proposals.len(), 1);
    assert_eq!(proposals[0].id, proposal_id);
    assert_eq!(proposals[0].agent_id, "rust-expert-agent");

    // Step 6: Verify proposal status
    let proposal = &proposals[0];
    assert_eq!(proposal.status, ProposalStatus::Pending);

    // Step 7: Approve the proposal
    router.approve_proposal(&proposal_id).await.unwrap();

    let approved_proposals = router.get_all_proposals().await;
    assert_eq!(approved_proposals[0].status, ProposalStatus::Approved);

    // Step 8: Verify proposal is no longer in pending state
    let next_pending = router.get_next_pending_proposal().await;
    assert!(next_pending.is_none());
}

#[tokio::test]
async fn test_error_frequency_escalation() {
    let mut classifier = ErrorClassifier::new();

    // First occurrence of an error should be Medium severity
    let error_line = "Error: Connection timeout";
    let result1 = classifier.classify(error_line, None);
    assert_eq!(result1.severity, ErrorSeverity::Medium);

    // After multiple occurrences, severity should escalate
    for _ in 0..5 {
        let _ = classifier.classify(error_line, None);
    }

    let result_after_repetition = classifier.classify(error_line, None);
    assert!(result_after_repetition.severity >= ErrorSeverity::High);
}

#[tokio::test]
async fn test_proposal_rejection() {
    let router = ErrorRouter::new();

    let error_context = create_error_context(
        "test-process-456",
        "TypeError: Cannot read property 'foo'",
        "typescript-agent",
    );

    let proposal_id = router.route_error(error_context).await.unwrap();

    // Reject the proposal
    router
        .reject_proposal(&proposal_id, Some("Fix looks incorrect".to_string()))
        .await
        .unwrap();

    let rejected_proposals = router.get_all_proposals().await;
    assert_eq!(rejected_proposals[0].status, ProposalStatus::Rejected);
}

#[tokio::test]
async fn test_multiple_concurrent_errors() {
    let router = ErrorRouter::new();

    // Simulate multiple errors happening concurrently
    let errors = vec![
        ("error[E0277]: trait bound not satisfied", "rust-expert-agent"),
        ("SyntaxError: Unexpected token", "typescript-agent"),
        ("fatal: unable to access", "build-agent"),
    ];

    let mut proposal_ids = Vec::new();

    for (i, (error_line, expected_agent)) in errors.iter().enumerate() {
        let error_context = create_error_context(
            &format!("process-{}", i),
            error_line,
            expected_agent,
        );

        let proposal_id = router.route_error(error_context).await.unwrap();
        proposal_ids.push(proposal_id);
    }

    // Verify all proposals were created
    let all_proposals = router.get_all_proposals().await;
    assert_eq!(all_proposals.len(), 3);

    // Verify agents were correctly assigned
    for (idx, (_, expected_agent)) in errors.iter().enumerate() {
        assert_eq!(all_proposals[idx].agent_id, *expected_agent);
    }
}

#[tokio::test]
async fn test_get_next_pending_proposal() {
    let router = ErrorRouter::new();

    // Create multiple proposals
    for i in 0..3 {
        let error_context = create_error_context(
            &format!("process-{}", i),
            &format!("Error {}", i),
            "code-agent",
        );

        router.route_error(error_context).await.unwrap();
    }

    // Get next pending proposal (FIFO order)
    let next = router.get_next_pending_proposal().await;
    assert!(next.is_some());

    let proposal = next.unwrap();
    assert_eq!(proposal.error_context.process_id, "process-0");

    // After approving, next should be process-1
    router.approve_proposal(&proposal.id).await.unwrap();

    let next2 = router.get_next_pending_proposal().await;
    assert!(next2.is_some());
    assert_eq!(next2.unwrap().error_context.process_id, "process-1");
}

#[tokio::test]
async fn test_approval_and_application_flow() {
    let router = ErrorRouter::new();

    let error_context = create_error_context(
        "test-process-789",
        "error: package lock file is out of date",
        "build-agent",
    );

    let proposal_id = router.route_error(error_context).await.unwrap();

    // Verify initial status
    let proposals = router.get_all_proposals().await;
    assert_eq!(proposals[0].status, ProposalStatus::Pending);

    // Approve
    router.approve_proposal(&proposal_id).await.unwrap();

    let proposals = router.get_all_proposals().await;
    assert_eq!(proposals[0].status, ProposalStatus::Approved);

    // Apply
    router.apply_approved_fix(&proposal_id).await.unwrap();

    let proposals = router.get_all_proposals().await;
    assert_eq!(proposals[0].status, ProposalStatus::Applied);
}

#[tokio::test]
async fn test_fix_verification() {
    use std::time::Duration;

    let router = ErrorRouter::new();

    let error_context = create_error_context(
        "test-process-verify",
        "npm ERR! missing script: start",
        "build-agent",
    );

    let proposal_id = router.route_error(error_context).await.unwrap();

    // Approve and apply
    router.approve_proposal(&proposal_id).await.unwrap();
    router.apply_approved_fix(&proposal_id).await.unwrap();

    // Verify
    let verification_result = router
        .verify_fix(&proposal_id, Duration::from_secs(5))
        .await
        .unwrap();

    assert!(verification_result.success);

    let proposals = router.get_all_proposals().await;
    assert_eq!(proposals[0].status, ProposalStatus::Verified);
}
