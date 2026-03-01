//! Reusable deploy pipeline for compiling and deploying individual workflow
//! services.
//!
//! The core function [`deploy_single_service`] encapsulates the full
//! validate -> codegen -> store -> update-status pipeline so that it can be
//! called from both the single-service deploy endpoint and the bundled
//! project-deploy endpoint (Task 2).

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::state::AppState;
use crate::codegen;
use crate::validation;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Status ID for a deployed workflow.
pub const DEPLOYED_STATUS_ID: &str = "00000000-0000-0000-0000-000000000003";

// ---------------------------------------------------------------------------
// Result types
// ---------------------------------------------------------------------------

/// Categorises the step at which the deploy pipeline failed so that callers
/// can map failures to the appropriate HTTP status without string matching.
#[derive(Debug)]
pub enum DeployFailureKind {
    /// The workflow was not found (or the user does not own it).
    NotFound,
    /// The stored definition JSON could not be parsed.
    ParseError,
    /// Validation produced one or more errors.
    ValidationFailed(Vec<String>),
    /// Code generation failed.
    CodegenError,
    /// Writing compiled code or reading from the database failed.
    StorageError,
    /// Updating the workflow status row failed.
    StatusUpdateError,
}

/// Outcome of deploying a single service/workflow.
#[derive(Debug)]
pub enum SingleServiceResult {
    /// The service was compiled and deployed successfully.
    Success {
        service_id: String,
        compiled_at: String,
    },
    /// The service failed at some step in the pipeline.
    Failure {
        service_id: String,
        kind: DeployFailureKind,
        error: String,
    },
}

// ---------------------------------------------------------------------------
// Report types (returned by bundled deploy in Task 2)
// ---------------------------------------------------------------------------

/// Summary report for a (possibly bundled) deploy operation.
#[derive(Debug, Serialize)]
pub struct DeployReport {
    pub project_id: String,
    pub deployed: Vec<DeployedService>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed: Option<FailedService>,
    pub skipped: Vec<SkippedService>,
}

/// A service that was successfully deployed.
#[derive(Debug, Serialize)]
pub struct DeployedService {
    pub service_id: String,
    pub compiled_at: String,
}

/// A service that failed to deploy.
#[derive(Debug, Serialize)]
pub struct FailedService {
    pub service_id: String,
    pub error: String,
}

/// A service that was skipped (e.g. already deployed, unchanged).
#[derive(Debug, Serialize)]
pub struct SkippedService {
    pub service_id: String,
    pub reason: String,
}

// ---------------------------------------------------------------------------
// Supabase row types (private to this module)
// ---------------------------------------------------------------------------

/// Row shape for loading a workflow from the `workflows` table.
#[derive(Debug, Deserialize)]
struct WorkflowRow {
    #[allow(dead_code)]
    id: String,
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    status_id: String,
    definition: serde_json::Value,
    #[allow(dead_code)]
    #[serde(default)]
    deployed_at: Option<String>,
}

/// Row to insert into `workflow_compiled_code`.
#[derive(Debug, Serialize)]
struct InsertCompiledCodeRow {
    id: String,
    workflow_id: String,
    code: serde_json::Value,
    compiled_at: String,
}

/// Row to update on the `workflows` table when deploying.
#[derive(Debug, Serialize)]
struct DeployUpdateRow {
    status_id: String,
    deployed_at: String,
}

// ---------------------------------------------------------------------------
// Core pipeline
// ---------------------------------------------------------------------------

/// Deploy a single workflow service through the full pipeline:
///
/// 1. Load workflow from `workflows` table
/// 2. Parse definition into [`WorkflowDefinition`]
/// 3. Validate via [`validation::validate`]
/// 4. Generate code via [`codegen::generate`]
/// 5. Store compiled code in `workflow_compiled_code`
/// 6. Update workflow status to deployed
/// 7. Fire telemetry (fire-and-forget) if discovery is configured
///
/// Returns [`SingleServiceResult::Success`] or [`SingleServiceResult::Failure`]
/// so that callers (including bundled deploy) never short-circuit on error.
pub async fn deploy_single_service(
    state: &AppState,
    service_id: &str,
    user_id: &str,
) -> SingleServiceResult {
    let fail = |kind: DeployFailureKind, error: String| SingleServiceResult::Failure {
        service_id: service_id.to_string(),
        kind,
        error,
    };

    // 1. Load workflow from Supabase (scoped to user).
    let user_filter = format!("eq.{user_id}");
    let workflow: WorkflowRow = match state
        .supabase
        .select_one(
            "workflows",
            &[
                ("id", &format!("eq.{service_id}")),
                ("created_by", &user_filter),
                ("select", "id,name,status_id,definition,deployed_at"),
            ],
        )
        .await
    {
        Ok(row) => row,
        Err(e) => {
            return fail(
                DeployFailureKind::NotFound,
                format!("Failed to load workflow: {e}"),
            )
        }
    };

    // 2. Parse the stored definition JSONB into a WorkflowDefinition.
    let definition: crate::schema::WorkflowDefinition =
        match serde_json::from_value(workflow.definition) {
            Ok(def) => def,
            Err(e) => {
                return fail(
                    DeployFailureKind::ParseError,
                    format!("Failed to parse stored workflow definition: {e}"),
                )
            }
        };

    // 3. Validate.
    let validation_result = validation::validate(&definition);
    if !validation_result.is_valid() {
        let details: Vec<String> = validation_result
            .errors
            .iter()
            .map(std::string::ToString::to_string)
            .collect();
        return fail(
            DeployFailureKind::ValidationFailed(details),
            "Workflow validation failed".to_string(),
        );
    }

    // 4. Compile via codegen.
    let generated = match codegen::generate(&definition) {
        Ok(g) => g,
        Err(e) => {
            return fail(
                DeployFailureKind::CodegenError,
                format!("Code generation failed: {e}"),
            )
        }
    };

    // 5. Serialize the generated code as a JSON blob for storage.
    let code_json = match serde_json::to_value(&generated) {
        Ok(v) => v,
        Err(e) => {
            return fail(
                DeployFailureKind::CodegenError,
                format!("Failed to serialize generated code: {e}"),
            )
        }
    };

    let now = Utc::now().to_rfc3339();

    // 6. Insert compiled code into `workflow_compiled_code`.
    let compiled_row = InsertCompiledCodeRow {
        id: Uuid::new_v4().to_string(),
        workflow_id: service_id.to_string(),
        code: code_json,
        compiled_at: now.clone(),
    };

    let insert_result: Result<serde_json::Value, _> = state
        .supabase
        .insert("workflow_compiled_code", &compiled_row)
        .await;

    if let Err(e) = insert_result {
        return fail(
            DeployFailureKind::StorageError,
            format!("Failed to store compiled code: {e}"),
        );
    }

    // 7. Update workflow status to deployed.
    let update_body = DeployUpdateRow {
        status_id: DEPLOYED_STATUS_ID.to_string(),
        deployed_at: now.clone(),
    };

    let update_result: Result<Vec<serde_json::Value>, _> = state
        .supabase
        .update(
            "workflows",
            &[
                ("id", &format!("eq.{service_id}")),
                ("created_by", &user_filter),
            ],
            &update_body,
        )
        .await;

    if let Err(e) = update_result {
        return fail(
            DeployFailureKind::StatusUpdateError,
            format!("Failed to update workflow status: {e}"),
        );
    }

    // Fire-and-forget: record deploy telemetry in discovery service.
    if let Some(ref discovery) = state.discovery {
        let discovery = discovery.clone();
        let wf_id = service_id.to_string();
        let deploy_user_id = user_id.to_string();
        tokio::spawn(async move {
            discovery
                .telemetry(&wf_id, "deploy", &deploy_user_id, &[])
                .await;
        });
    }

    SingleServiceResult::Success {
        service_id: service_id.to_string(),
        compiled_at: now,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deploy_report_serialization() {
        let report = DeployReport {
            project_id: "proj-1".to_string(),
            deployed: vec![DeployedService {
                service_id: "svc-1".to_string(),
                compiled_at: "2026-03-01T12:00:00Z".to_string(),
            }],
            failed: Some(FailedService {
                service_id: "svc-2".to_string(),
                error: "Validation failed".to_string(),
            }),
            skipped: vec![SkippedService {
                service_id: "svc-3".to_string(),
                reason: "Already deployed".to_string(),
            }],
        };

        let json = serde_json::to_value(&report).unwrap();
        assert_eq!(json["project_id"], "proj-1");
        assert_eq!(json["deployed"][0]["service_id"], "svc-1");
        assert_eq!(json["deployed"][0]["compiled_at"], "2026-03-01T12:00:00Z");
        assert_eq!(json["failed"]["service_id"], "svc-2");
        assert_eq!(json["failed"]["error"], "Validation failed");
        assert_eq!(json["skipped"][0]["service_id"], "svc-3");
        assert_eq!(json["skipped"][0]["reason"], "Already deployed");
    }

    #[test]
    fn test_deploy_report_no_failures() {
        let report = DeployReport {
            project_id: "proj-2".to_string(),
            deployed: vec![DeployedService {
                service_id: "svc-a".to_string(),
                compiled_at: "2026-03-01T12:00:00Z".to_string(),
            }],
            failed: None,
            skipped: vec![],
        };

        let json = serde_json::to_value(&report).unwrap();
        assert_eq!(json["project_id"], "proj-2");
        assert_eq!(json["deployed"].as_array().unwrap().len(), 1);
        // `failed` should be absent from JSON thanks to skip_serializing_if
        assert!(json.get("failed").is_none());
        assert_eq!(json["skipped"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn test_single_service_result_success() {
        let result = SingleServiceResult::Success {
            service_id: "svc-ok".to_string(),
            compiled_at: "2026-03-01T12:00:00Z".to_string(),
        };

        match result {
            SingleServiceResult::Success {
                service_id,
                compiled_at,
            } => {
                assert_eq!(service_id, "svc-ok");
                assert_eq!(compiled_at, "2026-03-01T12:00:00Z");
            }
            SingleServiceResult::Failure { .. } => panic!("Expected Success"),
        }
    }

    #[test]
    fn test_single_service_result_failure() {
        let result = SingleServiceResult::Failure {
            service_id: "svc-bad".to_string(),
            kind: DeployFailureKind::CodegenError,
            error: "Code generation failed".to_string(),
        };

        match result {
            SingleServiceResult::Failure {
                service_id,
                kind,
                error,
            } => {
                assert_eq!(service_id, "svc-bad");
                assert_eq!(error, "Code generation failed");
                assert!(matches!(kind, DeployFailureKind::CodegenError));
            }
            SingleServiceResult::Success { .. } => panic!("Expected Failure"),
        }
    }

    #[test]
    fn test_deploy_failure_kind_variants() {
        // Verify all variants can be constructed and matched.
        let not_found = DeployFailureKind::NotFound;
        assert!(matches!(not_found, DeployFailureKind::NotFound));

        let parse = DeployFailureKind::ParseError;
        assert!(matches!(parse, DeployFailureKind::ParseError));

        let validation = DeployFailureKind::ValidationFailed(vec!["err1".to_string()]);
        match validation {
            DeployFailureKind::ValidationFailed(details) => {
                assert_eq!(details, vec!["err1"]);
            }
            _ => panic!("Expected ValidationFailed"),
        }

        let codegen = DeployFailureKind::CodegenError;
        assert!(matches!(codegen, DeployFailureKind::CodegenError));

        let storage = DeployFailureKind::StorageError;
        assert!(matches!(storage, DeployFailureKind::StorageError));

        let status = DeployFailureKind::StatusUpdateError;
        assert!(matches!(status, DeployFailureKind::StatusUpdateError));
    }
}
