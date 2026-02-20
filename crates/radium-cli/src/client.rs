use crate::config::Profile;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::de::DeserializeOwned;
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Authentication failed: {0}")]
    AuthError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("API error ({status}): {message}")]
    ApiError { status: u16, message: String },

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("Failed to parse response: {0}")]
    ParseError(String),
}

impl ApiError {
    /// Return the appropriate exit code for this error type.
    pub const fn exit_code(&self) -> i32 {
        match self {
            Self::ValidationError(_) => 1,
            Self::ApiError { .. } | Self::NetworkError(_) | Self::ParseError(_) => 2,
            Self::AuthError(_) => 3,
        }
    }
}

pub struct ApiClient {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
}

impl ApiClient {
    pub fn new(profile: &Profile) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: profile.api_url.trim_end_matches('/').to_string(),
            api_key: profile.api_key.clone(),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base_url)
    }

    fn auth_header(&self) -> String {
        format!("Bearer {}", self.api_key)
    }

    async fn handle_response<T: DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> Result<T, ApiError> {
        let status = response.status();

        if status == reqwest::StatusCode::UNAUTHORIZED
            || status == reqwest::StatusCode::FORBIDDEN
        {
            let body = response.text().await.unwrap_or_default();
            return Err(ApiError::AuthError(body));
        }

        if status == reqwest::StatusCode::UNPROCESSABLE_ENTITY
            || status == reqwest::StatusCode::BAD_REQUEST
        {
            let body = response.text().await.unwrap_or_default();
            return Err(ApiError::ValidationError(body));
        }

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(ApiError::ApiError {
                status: status.as_u16(),
                message: body,
            });
        }

        let body = response.text().await?;
        serde_json::from_str(&body)
            .map_err(|e| ApiError::ParseError(format!("{e}: {body}")))
    }

    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, ApiError> {
        let response = self
            .client
            .get(self.url(path))
            .header(AUTHORIZATION, self.auth_header())
            .send()
            .await?;
        self.handle_response(response).await
    }

    pub async fn post<T: DeserializeOwned>(
        &self,
        path: &str,
        body: &impl Serialize,
        content_type: &str,
    ) -> Result<T, ApiError> {
        let response = self
            .client
            .post(self.url(path))
            .header(AUTHORIZATION, self.auth_header())
            .header(CONTENT_TYPE, content_type)
            .json(body)
            .send()
            .await?;
        self.handle_response(response).await
    }

    pub async fn post_raw<T: DeserializeOwned>(
        &self,
        path: &str,
        body: String,
        content_type: &str,
    ) -> Result<T, ApiError> {
        let response = self
            .client
            .post(self.url(path))
            .header(AUTHORIZATION, self.auth_header())
            .header(CONTENT_TYPE, content_type)
            .body(body)
            .send()
            .await?;
        self.handle_response(response).await
    }

    pub async fn put<T: DeserializeOwned>(
        &self,
        path: &str,
        body: &impl Serialize,
        content_type: &str,
    ) -> Result<T, ApiError> {
        let response = self
            .client
            .put(self.url(path))
            .header(AUTHORIZATION, self.auth_header())
            .header(CONTENT_TYPE, content_type)
            .json(body)
            .send()
            .await?;
        self.handle_response(response).await
    }

    #[allow(dead_code)]
    pub async fn put_raw<T: DeserializeOwned>(
        &self,
        path: &str,
        body: String,
        content_type: &str,
    ) -> Result<T, ApiError> {
        let response = self
            .client
            .put(self.url(path))
            .header(AUTHORIZATION, self.auth_header())
            .header(CONTENT_TYPE, content_type)
            .body(body)
            .send()
            .await?;
        self.handle_response(response).await
    }

    pub async fn delete_request(&self, path: &str) -> Result<(), ApiError> {
        let response = self
            .client
            .delete(self.url(path))
            .header(AUTHORIZATION, self.auth_header())
            .send()
            .await?;

        let status = response.status();

        if status == reqwest::StatusCode::UNAUTHORIZED
            || status == reqwest::StatusCode::FORBIDDEN
        {
            let body = response.text().await.unwrap_or_default();
            return Err(ApiError::AuthError(body));
        }

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(ApiError::ApiError {
                status: status.as_u16(),
                message: body,
            });
        }

        Ok(())
    }
}
