//! Integration tests for Workflow CRUD operations.

mod common;

use radium_core::proto::{
    CreateWorkflowRequest, DeleteWorkflowRequest, GetWorkflowRequest, UpdateWorkflowRequest,
    Workflow, WorkflowStep,
};

use common::{create_test_client, start_test_server_on_port};

#[tokio::test]
async fn test_create_and_get_workflow() {
    start_test_server_on_port(50060).await;

    // Connect a client
    let mut client = create_test_client(50060).await;

    // 1. Create a new workflow
    let workflow = Workflow {
        id: "test-workflow-get".to_string(),
        name: "Get Test Workflow".to_string(),
        description: "A workflow for Get integration testing.".to_string(),
        steps: vec![WorkflowStep {
            id: "step-1".to_string(),
            name: "Step 1".to_string(),
            description: "First step".to_string(),
            task_id: "task-1".to_string(),
            config_json: "{}".to_string(),
            order: 0,
        }],
        state: serde_json::to_string(&radium_core::models::WorkflowState::Idle).unwrap(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };

    let create_request =
        tonic::Request::new(CreateWorkflowRequest { workflow: Some(workflow.clone()) });
    let create_response =
        client.create_workflow(create_request).await.expect("create_workflow RPC failed");
    assert_eq!(create_response.into_inner().workflow_id, workflow.id);

    // 2. Get the workflow
    let get_request = tonic::Request::new(GetWorkflowRequest { workflow_id: workflow.id.clone() });
    let get_response = client.get_workflow(get_request).await.expect("get_workflow RPC failed");

    // 3. Assert the response is correct
    let retrieved_workflow = get_response.into_inner().workflow.unwrap();
    assert_eq!(retrieved_workflow.id, workflow.id);
    assert_eq!(retrieved_workflow.name, workflow.name);
    assert_eq!(retrieved_workflow.steps.len(), 1);
}

#[tokio::test]
async fn test_update_and_delete_workflow() {
    start_test_server_on_port(50061).await;

    // Connect a client
    let mut client = create_test_client(50061).await;

    // 1. Create a new workflow
    let mut workflow = Workflow {
        id: "test-workflow-update-delete".to_string(),
        name: "Update/Delete Test Workflow".to_string(),
        description: "A workflow for Update/Delete integration testing.".to_string(),
        steps: vec![],
        state: serde_json::to_string(&radium_core::models::WorkflowState::Idle).unwrap(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };

    let create_request =
        tonic::Request::new(CreateWorkflowRequest { workflow: Some(workflow.clone()) });
    client.create_workflow(create_request).await.expect("create_workflow RPC failed");

    // 2. Update the workflow
    workflow.name = "Updated Workflow Name".to_string();
    workflow.steps.push(WorkflowStep {
        id: "step-1".to_string(),
        name: "Step 1".to_string(),
        description: "First step".to_string(),
        task_id: "task-1".to_string(),
        config_json: "{}".to_string(),
        order: 0,
    });
    let update_request =
        tonic::Request::new(UpdateWorkflowRequest { workflow: Some(workflow.clone()) });
    client.update_workflow(update_request).await.expect("update_workflow RPC failed");

    // 3. Verify the workflow was updated
    let get_request = tonic::Request::new(GetWorkflowRequest { workflow_id: workflow.id.clone() });
    let response = client.get_workflow(get_request).await.expect("get_workflow RPC failed");
    let retrieved_workflow = response.into_inner().workflow.unwrap();
    assert_eq!(retrieved_workflow.name, "Updated Workflow Name");
    assert_eq!(retrieved_workflow.steps.len(), 1);

    // 4. Delete the workflow
    let delete_request =
        tonic::Request::new(DeleteWorkflowRequest { workflow_id: workflow.id.clone() });
    client.delete_workflow(delete_request).await.expect("delete_workflow RPC failed");

    // 5. Verify the workflow was deleted
    let get_request = tonic::Request::new(GetWorkflowRequest { workflow_id: workflow.id.clone() });
    let response = client.get_workflow(get_request).await;
    assert!(response.is_err());
    assert_eq!(response.err().unwrap().code(), tonic::Code::NotFound);
}
