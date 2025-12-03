//! Integration tests for the orchestrator features.

mod common;

use radium_core::proto::{
    ExecuteAgentRequest, GetRegisteredAgentsRequest, RegisterAgentRequest, StartAgentRequest,
    StopAgentRequest,
};

use common::{create_test_client, start_test_server, start_test_server_on_port};

#[tokio::test]
async fn test_register_and_list_agents() {
    let port = start_test_server().await;
    let mut client = create_test_client(port).await;

    // Register an echo agent
    let register_request = RegisterAgentRequest {
        agent_id: "test-echo".to_string(),
        agent_type: "echo".to_string(),
        description: "Test echo agent".to_string(),
    };

    let response = client.register_agent(register_request).await.unwrap();
    assert!(response.get_ref().success);

    // List registered agents
    let list_request = GetRegisteredAgentsRequest {};
    let list_response = client.get_registered_agents(list_request).await.unwrap();
    let agents = &list_response.get_ref().agents;

    assert!(!agents.is_empty());
    let test_agent = agents.iter().find(|a| a.id == "test-echo");
    assert!(test_agent.is_some());
    assert_eq!(test_agent.unwrap().description, "Test echo agent");
    assert_eq!(test_agent.unwrap().state, "idle");
}

#[tokio::test]
async fn test_register_multiple_agent_types() {
    let port = start_test_server().await;
    let mut client = create_test_client(port).await;

    // Register echo agent
    let echo_request = RegisterAgentRequest {
        agent_id: "echo-1".to_string(),
        agent_type: "echo".to_string(),
        description: "Echo agent".to_string(),
    };
    let response = client.register_agent(echo_request).await.unwrap();
    assert!(response.get_ref().success);

    // Register simple agent
    let simple_request = RegisterAgentRequest {
        agent_id: "simple-1".to_string(),
        agent_type: "simple".to_string(),
        description: "Simple agent".to_string(),
    };
    let response = client.register_agent(simple_request).await.unwrap();
    assert!(response.get_ref().success);

    // Register chat agent
    let chat_request = RegisterAgentRequest {
        agent_id: "chat-1".to_string(),
        agent_type: "chat".to_string(),
        description: "Chat agent".to_string(),
    };
    let response = client.register_agent(chat_request).await.unwrap();
    assert!(response.get_ref().success);

    // Verify all agents are registered
    let list_request = GetRegisteredAgentsRequest {};
    let list_response = client.get_registered_agents(list_request).await.unwrap();
    let agents = &list_response.get_ref().agents;

    assert_eq!(agents.len(), 3);
    assert!(agents.iter().any(|a| a.id == "echo-1"));
    assert!(agents.iter().any(|a| a.id == "simple-1"));
    assert!(agents.iter().any(|a| a.id == "chat-1"));
}

#[tokio::test]
async fn test_execute_agent() {
    let port = start_test_server().await;
    let mut client = create_test_client(port).await;

    // Register an agent
    let register_request = RegisterAgentRequest {
        agent_id: "test-exec".to_string(),
        agent_type: "echo".to_string(),
        description: "Test execution agent".to_string(),
    };
    client.register_agent(register_request).await.unwrap();

    // Execute the agent
    let execute_request = ExecuteAgentRequest {
        agent_id: "test-exec".to_string(),
        input: "Hello, world!".to_string(),
        model_type: None,
        model_id: None,
    };

    let response = client.execute_agent(execute_request).await.unwrap();
    let result = response.get_ref();

    assert!(result.success);
    assert!(!result.output.is_empty());
    assert!(result.output.contains("Echo from test-exec"));
    assert!(result.output.contains("Hello, world!"));
    assert!(result.error.is_none());
}

#[tokio::test]
async fn test_execute_agent_with_custom_model() {
    let port = start_test_server().await;
    let mut client = create_test_client(port).await;

    // Register an agent
    let register_request = RegisterAgentRequest {
        agent_id: "test-model".to_string(),
        agent_type: "simple".to_string(),
        description: "Test model agent".to_string(),
    };
    client.register_agent(register_request).await.unwrap();

    // Execute with custom model
    let execute_request = ExecuteAgentRequest {
        agent_id: "test-model".to_string(),
        input: "Test input".to_string(),
        model_type: Some("mock".to_string()),
        model_id: Some("mock-model".to_string()),
    };

    let response = client.execute_agent(execute_request).await.unwrap();
    let result = response.get_ref();

    assert!(result.success);
    assert!(!result.output.is_empty());
}

#[tokio::test]
async fn test_execute_nonexistent_agent() {
    let port = start_test_server().await;
    let mut client = create_test_client(port).await;

    let execute_request = ExecuteAgentRequest {
        agent_id: "nonexistent".to_string(),
        input: "Test".to_string(),
        model_type: None,
        model_id: None,
    };

    let response = client.execute_agent(execute_request).await.unwrap();
    let result = response.get_ref();

    assert!(!result.success);
    assert!(result.error.is_some());
    assert!(result.error.as_ref().unwrap().contains("not found"));
}

#[tokio::test]
async fn test_start_and_stop_agent() {
    let port = start_test_server().await;
    let mut client = create_test_client(port).await;

    // Register an agent
    let register_request = RegisterAgentRequest {
        agent_id: "test-lifecycle".to_string(),
        agent_type: "echo".to_string(),
        description: "Test lifecycle agent".to_string(),
    };
    client.register_agent(register_request).await.unwrap();

    // Start the agent
    let start_request = StartAgentRequest { agent_id: "test-lifecycle".to_string() };
    let response = client.start_agent(start_request).await.unwrap();
    assert!(response.get_ref().success);

    // Verify agent state is running
    let list_request = GetRegisteredAgentsRequest {};
    let list_response = client.get_registered_agents(list_request).await.unwrap();
    let agents = &list_response.get_ref().agents;
    let agent = agents.iter().find(|a| a.id == "test-lifecycle").unwrap();
    assert_eq!(agent.state, "running");

    // Stop the agent
    let stop_request = StopAgentRequest { agent_id: "test-lifecycle".to_string() };
    let response = client.stop_agent(stop_request).await.unwrap();
    assert!(response.get_ref().success);

    // Verify agent state is stopped
    let list_request = GetRegisteredAgentsRequest {};
    let list_response = client.get_registered_agents(list_request).await.unwrap();
    let agents = &list_response.get_ref().agents;
    let agent = agents.iter().find(|a| a.id == "test-lifecycle").unwrap();
    assert_eq!(agent.state, "stopped");
}

#[tokio::test]
async fn test_register_invalid_agent_type() {
    let port = start_test_server().await;
    let mut client = create_test_client(port).await;

    let register_request = RegisterAgentRequest {
        agent_id: "invalid".to_string(),
        agent_type: "invalid_type".to_string(),
        description: "Invalid agent".to_string(),
    };

    let response = client.register_agent(register_request).await.unwrap();
    let result = response.get_ref();

    assert!(!result.success);
    assert!(result.error.is_some());
    assert!(result.error.as_ref().unwrap().contains("Unknown agent type"));
}

#[tokio::test]
async fn test_multiple_agent_executions() {
    let port = start_test_server().await;
    let mut client = create_test_client(port).await;

    // Register an agent
    let register_request = RegisterAgentRequest {
        agent_id: "multi-exec".to_string(),
        agent_type: "echo".to_string(),
        description: "Multi execution agent".to_string(),
    };
    client.register_agent(register_request).await.unwrap();

    // Execute multiple times
    for i in 0..5 {
        let execute_request = ExecuteAgentRequest {
            agent_id: "multi-exec".to_string(),
            input: format!("Message {}", i),
            model_type: None,
            model_id: None,
        };

        let response = client.execute_agent(execute_request).await.unwrap();
        let result = response.get_ref();

        assert!(result.success);
        assert!(result.output.contains(&format!("Message {}", i)));
    }
}
