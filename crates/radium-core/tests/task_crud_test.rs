#![cfg(feature = "server")]

//! Integration tests for Task CRUD operations.

mod common;

use radium_core::proto::{
    CreateTaskRequest, DeleteTaskRequest, GetTaskRequest, Task, UpdateTaskRequest,
};

use common::{create_test_client, start_test_server_on_port};

#[tokio::test]
async fn test_create_and_get_task() {
    start_test_server_on_port(50064).await;

    // Connect a client
    let mut client = create_test_client(50064).await;

    // 1. Create a new task
    let task = Task {
        id: "test-task-get".to_string(),
        name: "Get Test Task".to_string(),
        description: "A task for Get integration testing.".to_string(),
        agent_id: "agent-1".to_string(),
        input_json: "{}".to_string(),
        state: serde_json::to_string(&radium_core::models::TaskState::Queued).unwrap(),
        result_json: "".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };

    let create_request = tonic::Request::new(CreateTaskRequest { task: Some(task.clone()) });
    let create_response = client.create_task(create_request).await.expect("create_task RPC failed");
    assert_eq!(create_response.into_inner().task_id, task.id);

    // 2. Get the task
    let get_request = tonic::Request::new(GetTaskRequest { task_id: task.id.clone() });
    let get_response = client.get_task(get_request).await.expect("get_task RPC failed");

    // 3. Assert the response is correct
    let retrieved_task = get_response.into_inner().task.unwrap();
    assert_eq!(retrieved_task.id, task.id);
    assert_eq!(retrieved_task.name, task.name);
}

#[tokio::test]
async fn test_update_and_delete_task() {
    start_test_server_on_port(50065).await;

    // Connect a client
    let mut client = create_test_client(50065).await;

    // 1. Create a new task
    let mut task = Task {
        id: "test-task-update-delete".to_string(),
        name: "Update/Delete Test Task".to_string(),
        description: "A task for Update/Delete integration testing.".to_string(),
        agent_id: "agent-1".to_string(),
        input_json: "{}".to_string(),
        state: serde_json::to_string(&radium_core::models::TaskState::Queued).unwrap(),
        result_json: "".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };

    let create_request = tonic::Request::new(CreateTaskRequest { task: Some(task.clone()) });
    client.create_task(create_request).await.expect("create_task RPC failed");

    // 2. Update the task
    task.name = "Updated Task Name".to_string();
    task.state = serde_json::to_string(&radium_core::models::TaskState::Running).unwrap();
    let update_request = tonic::Request::new(UpdateTaskRequest { task: Some(task.clone()) });
    client.update_task(update_request).await.expect("update_task RPC failed");

    // 3. Verify the task was updated
    let get_request = tonic::Request::new(GetTaskRequest { task_id: task.id.clone() });
    let response = client.get_task(get_request).await.expect("get_task RPC failed");
    let retrieved_task = response.into_inner().task.unwrap();
    assert_eq!(retrieved_task.name, "Updated Task Name");
    assert_eq!(
        retrieved_task.state,
        serde_json::to_string(&radium_core::models::TaskState::Running).unwrap()
    );

    // 4. Delete the task
    let delete_request = tonic::Request::new(DeleteTaskRequest { task_id: task.id.clone() });
    client.delete_task(delete_request).await.expect("delete_task RPC failed");

    // 5. Verify the task was deleted
    let get_request = tonic::Request::new(GetTaskRequest { task_id: task.id.clone() });
    let response = client.get_task(get_request).await;
    assert!(response.is_err());
    assert_eq!(response.err().unwrap().code(), tonic::Code::NotFound);
}
