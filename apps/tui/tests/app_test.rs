use radium_tui::app::{App, AppState};
use radium_tui::navigation::View;

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
