//! Integration tests for Agent CRUD operations.

mod common;

use radium_core::proto::{
    Agent, CreateAgentRequest, DeleteAgentRequest, GetAgentRequest, UpdateAgentRequest,
};

use common::{create_test_client, start_test_server_on_port};

#[tokio::test]
async fn test_create_and_get_agent() {
    start_test_server_on_port(50056).await;

    // Connect a client
    let mut client = create_test_client(50056).await;

    // 1. Create a new agent
    let agent = Agent {
        id: "test-agent-get".to_string(),
        name: "Get Test Agent".to_string(),
        description: "An agent for Get integration testing.".to_string(),
        config_json: r#"{"model_id": "test-model"}"#.to_string(),
        state: serde_json::to_string(&radium_core::models::AgentState::Idle).unwrap(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };

    let create_request = tonic::Request::new(CreateAgentRequest { agent: Some(agent.clone()) });
    let create_response =
        client.create_agent(create_request).await.expect("create_agent RPC failed");
    assert_eq!(create_response.into_inner().agent_id, agent.id);

    // 2. Get the agent
    let get_request = tonic::Request::new(GetAgentRequest { agent_id: agent.id.clone() });
    let get_response = client.get_agent(get_request).await.expect("get_agent RPC failed");

    // 3. Assert the response is correct
    let retrieved_agent = get_response.into_inner().agent.unwrap();
    assert_eq!(retrieved_agent.id, agent.id);
    assert_eq!(retrieved_agent.name, agent.name);
}

#[tokio::test]
async fn test_update_and_delete_agent() {
    start_test_server_on_port(50057).await;

    // Connect a client
    let mut client = create_test_client(50057).await;

    // 1. Create a new agent
    let mut agent = Agent {
        id: "test-agent-update-delete".to_string(),
        name: "Update/Delete Test Agent".to_string(),
        description: "An agent for Update/Delete integration testing.".to_string(),
        config_json: r#"{"model_id": "test-model"}"#.to_string(),
        state: serde_json::to_string(&radium_core::models::AgentState::Idle).unwrap(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };

    let create_request = tonic::Request::new(CreateAgentRequest { agent: Some(agent.clone()) });
    client.create_agent(create_request).await.expect("create_agent RPC failed");

    // 2. Update the agent
    agent.name = "Updated Agent Name".to_string();
    agent.description = "Updated description.".to_string();
    let update_request = tonic::Request::new(UpdateAgentRequest { agent: Some(agent.clone()) });
    client.update_agent(update_request).await.expect("update_agent RPC failed");

    // 3. Verify the agent was updated
    let get_request = tonic::Request::new(GetAgentRequest { agent_id: agent.id.clone() });
    let response = client.get_agent(get_request).await.expect("get_agent RPC failed");
    let retrieved_agent = response.into_inner().agent.unwrap();
    assert_eq!(retrieved_agent.name, "Updated Agent Name");
    assert_eq!(retrieved_agent.description, "Updated description.");

    // 4. Delete the agent
    let delete_request = tonic::Request::new(DeleteAgentRequest { agent_id: agent.id.clone() });
    client.delete_agent(delete_request).await.expect("delete_agent RPC failed");

    // 5. Verify the agent was deleted
    let get_request = tonic::Request::new(GetAgentRequest { agent_id: agent.id.clone() });
    let response = client.get_agent(get_request).await;
    assert!(response.is_err());
    assert_eq!(response.err().unwrap().code(), tonic::Code::NotFound);
}
