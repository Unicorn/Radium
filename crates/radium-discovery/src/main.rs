use std::net::SocketAddr;

mod api;
mod config;
mod embeddings;
mod graph;
mod state;

use config::DiscoveryConfig;
use state::AppState;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "radium_discovery=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = DiscoveryConfig::from_env().expect(
        "Discovery service requires NEO4J_URI, NEO4J_USER, NEO4J_PASSWORD environment variables",
    );

    tracing::info!("Connecting to Neo4j at {}", config.neo4j_uri);
    let graph = neo4rs::Graph::new(&config.neo4j_uri, &config.neo4j_user, &config.neo4j_password)
        .await
        .expect("Failed to connect to Neo4j");

    let embedding_provider = embeddings::create_provider()
        .expect("No embedding provider configured. Set ANTHROPIC_API_KEY or OPENAI_API_KEY.");

    tracing::info!("Using embedding provider: {}", embedding_provider.provider_name());

    graph::schema::initialize(&graph)
        .await
        .expect("Failed to initialize Neo4j schema");

    graph::schema::initialize_vector_indexes(&graph, embedding_provider.dimension())
        .await
        .expect("Failed to initialize vector indexes");

    let state = AppState {
        graph,
        embeddings: std::sync::Arc::from(embedding_provider),
    };

    let app = api::router(state);
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));

    tracing::info!("Discovery service listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.expect("Failed to bind TCP listener");
    axum::serve(listener, app).await.expect("Server exited with error");
}
