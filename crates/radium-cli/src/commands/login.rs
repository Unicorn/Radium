use crate::config::{Config, Profile};

/// Create or update a profile in the config file.
///
/// # Errors
///
/// Returns an error if the config file cannot be read or written.
pub fn run(
    profile: &str,
    url: &str,
    key: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut config = Config::load().unwrap_or_default();

    config.set_profile(
        profile.to_string(),
        Profile {
            api_url: url.to_string(),
            api_key: key.to_string(),
        },
    );

    config.save()?;

    let result = serde_json::json!({
        "status": "ok",
        "message": format!("Profile '{profile}' saved successfully."),
        "profile": profile,
        "api_url": url,
    });

    Ok(serde_json::to_string_pretty(&result)?)
}
