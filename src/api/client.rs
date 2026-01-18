//! Fansly API HTTP client.

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use reqwest::{header, Client, Response};
use tokio::sync::RwLock;

use crate::api::auth::{generate_check_hash, get_client_timestamp, is_device_id_expired};
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
        let session_id = get_session_id(&token).await?;

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
        device_id.clone().ok_or_else(|| Error::Api("No device ID available".into()))
    }

    /// Get the current device ID timestamp.
    pub async fn get_device_id_timestamp(&self) -> Option<i64> {
        *self.device_id_timestamp.read().await
    }

    /// Ensure we have a valid device ID, fetching a new one if needed.
    async fn ensure_device_id(&self) -> Result<()> {
        let timestamp = *self.device_id_timestamp.read().await;

        if is_device_id_expired(timestamp) {
            let new_device_id = self.fetch_device_id().await?;
            let new_timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64;

            *self.device_id.write().await = Some(new_device_id);
            *self.device_id_timestamp.write().await = Some(new_timestamp);
        }

        Ok(())
    }

    /// Fetch a new device ID from the API.
    async fn fetch_device_id(&self) -> Result<String> {
        let url = format!("{}/api/v1/device/id", API_BASE);

        let response = self.client
            .get(&url)
            .header(header::AUTHORIZATION, &self.token)
            .send()
            .await?;

        let api_response: ApiResponse<DeviceIdResponse> = response.json().await?;

        if !api_response.success {
            return Err(Error::Api("Failed to get device ID".into()));
        }

        Ok(api_response.response.device_id)
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

        let response = self.client
            .get(&url)
            .headers(headers)
            .send()
            .await?;

        // Check for rate limiting
        if response.status() == 429 {
            return Err(Error::RateLimited(60));
        }

        // Check for auth errors
        if response.status() == 401 {
            return Err(Error::Authentication("Invalid or expired token".into()));
        }

        Ok(response)
    }

    /// Get client account information (validates token).
    pub async fn get_client_account_info(&self) -> Result<AccountInfo> {
        let response = self.get("/api/v1/account/me").await?;
        let api_response: ApiResponse<AccountInfo> = response.json().await?;

        if !api_response.success {
            return Err(Error::Authentication("Failed to get account info".into()));
        }

        Ok(api_response.response)
    }

    /// Get creator account information by username.
    pub async fn get_creator_account_info(&self, username: &str) -> Result<AccountInfo> {
        let path = format!("/api/v1/account?usernames={}", username);
        let response = self.get(&path).await?;

        // The response is an array of accounts
        let api_response: ApiResponse<Vec<AccountInfo>> = response.json().await?;

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
        let api_response: ApiResponse<TimelineResponse> = response.json().await?;

        if !api_response.success {
            return Err(Error::Api("Failed to get timeline".into()));
        }

        Ok(api_response.response)
    }

    /// Get message groups.
    pub async fn get_groups(&self) -> Result<Vec<MessageGroup>> {
        let response = self.get("/api/v1/group").await?;
        let api_response: ApiResponse<GroupsResponse> = response.json().await?;

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
        let api_response: ApiResponse<MessagesResponse> = response.json().await?;

        if !api_response.success {
            return Err(Error::Api("Failed to get messages".into()));
        }

        Ok(api_response.response)
    }

    /// Get a single post by ID.
    pub async fn get_post(&self, post_id: &str) -> Result<PostResponse> {
        let path = format!("/api/v1/post?ids={}", post_id);

        let response = self.get(&path).await?;
        let api_response: ApiResponse<PostResponse> = response.json().await?;

        if !api_response.success {
            return Err(Error::Api("Failed to get post".into()));
        }

        Ok(api_response.response)
    }

    /// Get media collections (purchased items).
    pub async fn get_collections(&self) -> Result<Vec<MediaOrder>> {
        let response = self.get("/api/v1/account/media/orders/").await?;
        let api_response: ApiResponse<CollectionsResponse> = response.json().await?;

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
        let api_response: ApiResponse<MediaInfoResponse> = response.json().await?;

        if !api_response.success {
            return Err(Error::Api("Failed to get media info".into()));
        }

        Ok(api_response.response.account_media)
    }

    /// Download a file from a URL (with optional streaming).
    pub async fn download_file(&self, url: &str) -> Result<Response> {
        let response = self.client
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
