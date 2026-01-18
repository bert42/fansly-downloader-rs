//! Fansly API HTTP client.

use std::sync::Arc;

use reqwest::{header, Client, Response};
use tokio::sync::RwLock;

use crate::api::auth::{generate_check_hash, get_client_timestamp};
use crate::api::types::*;
use crate::api::websocket::get_session_id;
use crate::error::{Error, Result};

/// Fansly API base URL.
const API_BASE: &str = "https://apiv3.fansly.com";

/// Maximum media IDs per batch request.
pub const BATCH_SIZE: usize = 150;

/// Fansly API client with authentication and session management.
pub struct FanslyApi {
    client: Client,
    token: String,
    user_agent: String,
    check_key: String,
    session_id: String,
    device_id: Arc<RwLock<Option<String>>>,
    device_id_timestamp: Arc<RwLock<Option<i64>>>,
    client_timestamp: Arc<RwLock<i64>>,
}

impl FanslyApi {
    /// Create a new API client and establish WebSocket session.
    pub async fn new(
        token: String,
        user_agent: String,
        check_key: String,
        device_id: Option<String>,
        device_id_timestamp: Option<i64>,
    ) -> Result<Self> {
        // Build HTTP client
        let client = Client::builder()
            .user_agent(&user_agent)
            .build()
            .map_err(|e| Error::Api(format!("Failed to create HTTP client: {}", e)))?;

        // Get WebSocket session ID
        let session_id = get_session_id(&token, &user_agent).await?;

        let api = Self {
            client,
            token,
            user_agent,
            check_key,
            session_id,
            device_id: Arc::new(RwLock::new(device_id)),
            device_id_timestamp: Arc::new(RwLock::new(device_id_timestamp)),
            client_timestamp: Arc::new(RwLock::new(get_client_timestamp())),
        };

        // Ensure we have a valid device ID
        api.ensure_device_id().await?;

        Ok(api)
    }

    /// Get the current device ID, refreshing if expired.
    pub async fn get_device_id(&self) -> Result<String> {
        self.ensure_device_id().await?;
        let device_id = self.device_id.read().await;
        device_id
            .clone()
            .ok_or_else(|| Error::Api("No device ID available".into()))
    }

    /// Get the current device ID timestamp.
    pub async fn get_device_id_timestamp(&self) -> Option<i64> {
        *self.device_id_timestamp.read().await
    }

    /// Ensure we have a valid device ID.
    async fn ensure_device_id(&self) -> Result<()> {
        let device_id = self.device_id.read().await;
        if device_id.is_none() {
            return Err(Error::MissingConfig(
                "device_id (get this from the 'fansly-d' cookie in your browser)".to_string(),
            ));
        }
        Ok(())
    }

    /// Build common headers for API requests.
    async fn build_headers(&self, url_path: &str) -> Result<header::HeaderMap> {
        let mut headers = header::HeaderMap::new();

        let device_id = self.get_device_id().await?;

        // Update client timestamp if needed
        let mut ts = self.client_timestamp.write().await;
        let new_ts = get_client_timestamp();
        if new_ts > *ts {
            *ts = new_ts;
        }
        let client_ts = *ts;
        drop(ts);

        // Generate check hash
        let check_hash = generate_check_hash(&self.check_key, url_path, &device_id);

        headers.insert(header::AUTHORIZATION, self.token.parse().unwrap());
        headers.insert("fansly-client-id", device_id.parse().unwrap());
        headers.insert("fansly-client-ts", client_ts.to_string().parse().unwrap());
        headers.insert("fansly-client-check", check_hash.parse().unwrap());
        headers.insert("fansly-session-id", self.session_id.parse().unwrap());

        Ok(headers)
    }

    /// Make an authenticated GET request.
    async fn get(&self, path: &str) -> Result<Response> {
        let url = format!("{}{}", API_BASE, path);
        let headers = self.build_headers(path).await?;

        tracing::debug!("GET {}", url);
        tracing::debug!("Headers: {:?}", headers);

        let response = self
            .client
            .get(&url)
            .query(&[("ngsw-bypass", "true")])
            .headers(headers)
            .send()
            .await?;

        let status = response.status();
        tracing::debug!("Response status: {}", status);

        // Check for rate limiting
        if status == 429 {
            return Err(Error::RateLimited(60));
        }

        // Check for auth errors
        if status == 401 || status == 403 {
            let body = response.text().await.unwrap_or_default();
            tracing::error!("Auth error response: {}", body);
            return Err(Error::Authentication(format!(
                "HTTP {}: {}",
                status,
                if body.is_empty() {
                    "Authentication failed"
                } else {
                    &body
                }
            )));
        }

        Ok(response)
    }

    /// Get client account information (validates token).
    pub async fn get_client_account_info(&self) -> Result<AccountInfo> {
        let response = self.get("/api/v1/account/me").await?;
        let text = response.text().await?;
        tracing::debug!("Account info response: {}", text);

        let api_response: ApiResponse<AccountMeResponse> =
            serde_json::from_str(&text).map_err(|e| {
                Error::Api(format!(
                    "Failed to parse account info: {} - Response: {}",
                    e, text
                ))
            })?;

        if !api_response.success {
            return Err(Error::Authentication("Failed to get account info".into()));
        }

        Ok(api_response.response.account)
    }

    /// Get creator account information by username.
    pub async fn get_creator_account_info(&self, username: &str) -> Result<AccountInfo> {
        let path = format!("/api/v1/account?usernames={}", username);
        let response = self.get(&path).await?;
        let text = response.text().await?;
        tracing::debug!("Creator account response: {}", text);

        // The response is an array of accounts
        let api_response: ApiResponse<Vec<AccountInfo>> =
            serde_json::from_str(&text).map_err(|e| {
                Error::Api(format!(
                    "Failed to parse creator account: {} - Response: {}",
                    e, text
                ))
            })?;

        if !api_response.success || api_response.response.is_empty() {
            return Err(Error::AccountNotFound(username.to_string()));
        }

        Ok(api_response.response.into_iter().next().unwrap())
    }

    /// Get timeline posts for a creator.
    pub async fn get_timeline(&self, creator_id: &str, cursor: &str) -> Result<TimelineResponse> {
        let path = format!(
            "/api/v1/timelinenew/{}?before={}&after=0&wallId=&contentSearch=",
            creator_id, cursor
        );

        let response = self.get(&path).await?;
        let text = response.text().await?;
        tracing::debug!("Timeline response: {}", text);

        let api_response: ApiResponse<TimelineResponse> =
            serde_json::from_str(&text).map_err(|e| {
                Error::Api(format!(
                    "Failed to parse timeline: {} - Response: {}",
                    e, text
                ))
            })?;

        if !api_response.success {
            return Err(Error::Api("Failed to get timeline".into()));
        }

        Ok(api_response.response)
    }

    /// Get message groups.
    pub async fn get_groups(&self) -> Result<Vec<MessageGroup>> {
        let response = self.get("/api/v1/group").await?;
        let status = response.status();
        let text = response.text().await?;
        tracing::debug!("Groups response: {}", text);

        // Handle 400 error with "missing groupId" - means no groups exist
        if status == 400 {
            if text.contains("missing groupId") {
                tracing::debug!("No message groups found");
                return Ok(Vec::new());
            }
            return Err(Error::Api(format!(
                "Failed to get groups: HTTP {} - {}",
                status, text
            )));
        }

        // Parse successful response
        let api_response: ApiResponse<GroupsResponse> =
            serde_json::from_str(&text).map_err(|e| {
                Error::Api(format!(
                    "Failed to parse groups: {} - Response: {}",
                    e, text
                ))
            })?;

        if !api_response.success {
            return Err(Error::Api("Failed to get message groups".into()));
        }

        Ok(api_response.response.groups)
    }

    /// Get messages from a group.
    pub async fn get_messages(&self, group_id: &str, cursor: &str) -> Result<MessagesResponse> {
        let path = format!(
            "/api/v1/message?groupId={}&limit=25&before={}",
            group_id, cursor
        );

        let response = self.get(&path).await?;
        let text = response.text().await?;
        tracing::debug!("Messages response: {}", text);

        let api_response: ApiResponse<MessagesResponse> =
            serde_json::from_str(&text).map_err(|e| {
                Error::Api(format!(
                    "Failed to parse messages: {} - Response: {}",
                    e, text
                ))
            })?;

        if !api_response.success {
            return Err(Error::Api("Failed to get messages".into()));
        }

        Ok(api_response.response)
    }

    /// Get a single post by ID.
    pub async fn get_post(&self, post_id: &str) -> Result<PostResponse> {
        let path = format!("/api/v1/post?ids={}", post_id);

        let response = self.get(&path).await?;
        let text = response.text().await?;
        tracing::debug!("Post response: {}", text);

        let api_response: ApiResponse<PostResponse> = serde_json::from_str(&text)
            .map_err(|e| Error::Api(format!("Failed to parse post: {} - Response: {}", e, text)))?;

        if !api_response.success {
            return Err(Error::Api("Failed to get post".into()));
        }

        Ok(api_response.response)
    }

    /// Get media collections (purchased items).
    pub async fn get_collections(&self) -> Result<Vec<MediaOrder>> {
        let response = self.get("/api/v1/account/media/orders/").await?;
        let text = response.text().await?;
        tracing::debug!("Collections response: {}", text);

        let api_response: ApiResponse<CollectionsResponse> =
            serde_json::from_str(&text).map_err(|e| {
                Error::Api(format!(
                    "Failed to parse collections: {} - Response: {}",
                    e, text
                ))
            })?;

        if !api_response.success {
            return Err(Error::Api("Failed to get collections".into()));
        }

        Ok(api_response.response.account_media_orders)
    }

    /// Get media info by IDs (batch request).
    pub async fn get_media_info(&self, media_ids: &[String]) -> Result<Vec<AccountMedia>> {
        if media_ids.is_empty() {
            return Ok(Vec::new());
        }

        let ids_str = media_ids.join(",");
        let path = format!("/api/v1/account/media?ids={}", ids_str);

        let response = self.get(&path).await?;
        let text = response.text().await?;
        tracing::debug!("Media info response length: {} bytes", text.len());

        // Response is directly an array: {"success":true,"response":[...]}
        let api_response: ApiResponse<Vec<AccountMedia>> =
            serde_json::from_str(&text).map_err(|e| {
                Error::Api(format!(
                    "Failed to parse media info: {} - Response: {}",
                    e,
                    &text[..text.len().min(500)]
                ))
            })?;

        if !api_response.success {
            return Err(Error::Api("Failed to get media info".into()));
        }

        Ok(api_response.response)
    }

    /// Download a file from a URL (with optional streaming).
    pub async fn download_file(&self, url: &str) -> Result<Response> {
        let response = self
            .client
            .get(url)
            .header(header::USER_AGENT, &self.user_agent)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::Download(format!(
                "Failed to download file: HTTP {}",
                response.status()
            )));
        }

        Ok(response)
    }
}
