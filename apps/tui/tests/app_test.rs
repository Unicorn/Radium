mod common;

use radium_tui::app::{App, AppState};
use radium_tui::navigation::View;
use radium_core::proto::{Agent, CreateAgentRequest, Workflow, CreateWorkflowRequest, Task, CreateTaskRequest};
use radium_core::models::WorkflowState;
use crossterm::event::{KeyCode, KeyModifiers};
use serde_json;

use common::{create_test_client, start_test_server};

#[test]
fn test_app_state_new() {
    let server_addr = "http://127.0.0.1:50051".to_string();
    let state = AppState::new(server_addr.clone());
    
    assert_eq!(state.server_addr, server_addr);
    // connection_status is Arc<Mutex<String>>, can't easily check synchronously without async runtime,
    // but we can check it's initialized.
}

#[test]
fn test_app_new() {
    let server_addr = "http://127.0.0.1:50051".to_string();
    let app = App::new(server_addr.clone());
    
    assert!(!app.should_quit);
    assert_eq!(app.app_state.server_addr, server_addr);
    assert!(matches!(app.navigation.current_view(), View::Dashboard));
    assert!(app.dashboard_data.is_none());
    assert!(app.error_message.is_none());
}

#[tokio::test]
async fn test_app_connect_failure() {
    // Test connection to invalid address
    let server_addr = "http://invalid-address:50051".to_string();
    let app = App::new(server_addr);
    
    // Attempt connection
    let result = app.app_state.connect().await;
    
    // It should fail
    assert!(result.is_err());
}

#[tokio::test]
async fn test_app_state_connect_success() {
    let port = start_test_server().await;
    let server_addr = format!("http://127.0.0.1:{}", port);
    let mut state = AppState::new(server_addr.clone());

    let result = state.connect().await;
    assert!(result.is_ok());

    let status = state.connection_status.lock().await;
    assert!(status.contains("Connected"));
    assert!(status.contains(&server_addr));
}

#[tokio::test]
async fn test_app_refresh_dashboard_success() {
    let port = start_test_server().await;
    let server_addr = format!("http://127.0.0.1:{}", port);
    let mut app = App::new(server_addr.clone());

    // Connect the app
    app.app_state.connect().await.unwrap();

    // Create some mock data in the server for dashboard to pick up
    let mut client = create_test_client(port).await;
    let now = chrono::Utc::now().to_rfc3339();

    // Agent
    client.create_agent(tonic::Request::new(CreateAgentRequest {
        agent: Some(Agent {
            id: "test-agent-1".to_string(),
            name: "Test Agent 1".to_string(),
            description: "A test agent".to_string(),
            config_json: r#"{"model_id": "test-model"}"#.to_string(),
            state: "\"idle\"".to_string(),
            created_at: now.clone(),
            updated_at: now.clone(),
        }),
    })).await.unwrap();

    // Workflow
    client.create_workflow(tonic::Request::new(CreateWorkflowRequest {
        workflow: Some(Workflow {
            id: "test-workflow-1".to_string(),
            name: "Test Workflow 1".to_string(),
            description: "A test workflow".to_string(),
            steps: vec![],
            state: serde_json::to_string(&WorkflowState::Idle).unwrap(),
            created_at: now.clone(),
            updated_at: now.clone(),
        }),
    })).await.unwrap();

    // Task
    client.create_task(tonic::Request::new(CreateTaskRequest {
        task: Some(Task {
            id: "test-task-1".to_string(),
            name: "Test Task 1".to_string(),
            description: "A test task".to_string(),
            agent_id: "test-agent-1".to_string(),
            input_json: "{}".to_string(),
            state: "\"queued\"".to_string(),
            result_json: "".to_string(),
            created_at: now.clone(),
            updated_at: now.clone(),
        }),
    })).await.unwrap();

    // Refresh view
    app.refresh_current_view().await;

    // Assert dashboard data is loaded
    assert!(app.dashboard_data.is_some());
    let data = app.dashboard_data.unwrap();
    assert_eq!(data.agent_count, 1);
    assert_eq!(data.workflow_count, 1);
    assert_eq!(data.task_count, 1);
    assert!(app.error_message.is_none());
}

#[tokio::test]
async fn test_app_refresh_dashboard_failure() {
    // Don't start server, so connection fails for data fetch
    let server_addr = "http://127.0.0.1:99999".to_string(); // Invalid port
    let mut app = App::new(server_addr.clone());
    
    // Attempt to connect, this will set the client to None.
    let _ = app.app_state.connect().await; 

    // Set current view to Dashboard
    app.navigation.set_view(View::Dashboard);

    // Refresh view - should fail to fetch data
    app.refresh_current_view().await;

    assert!(app.dashboard_data.is_none());
    assert!(app.error_message.is_some());
    assert!(app.error_message.unwrap().contains("Not connected to server"));
}

#[tokio::test]
async fn test_app_navigation_to_agents() {
    let port = start_test_server().await;
    let server_addr = format!("http://127.0.0.1:{}", port);
    let mut app = App::new(server_addr.clone());

    // Connect the app
    app.app_state.connect().await.unwrap();

    // Simulate key press '2' for Agents view
    app.handle_key(KeyCode::Char('2'), KeyModifiers::NONE).await.unwrap();

    // Assert navigation changed to Agents view
    assert!(matches!(app.navigation.current_view(), View::Agents));
    assert!(app.agent_data.is_some());
    assert!(app.error_message.is_none());
}

#[tokio::test]
async fn test_app_reconnect_success() {
    // Start server
    let port = start_test_server().await;
    let server_addr = format!("http://127.0.0.1:{}", port);
    let mut app = App::new(server_addr.clone());

    // Simulate initial disconnection (client is None, status is Disconnected)
    // (No explicit connect call here, simulate initial state)
    
    // Simulate key press 'c' to reconnect
    app.handle_key(KeyCode::Char('c'), KeyModifiers::NONE).await.unwrap();

    // Assert connection status is updated and dashboard data is loaded
    let status = app.app_state.connection_status.lock().await;
    assert!(status.contains("Connected"));
    assert!(app.dashboard_data.is_some());
    assert!(app.error_message.is_none());
}
