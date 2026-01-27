//! API response type definitions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Generic API response wrapper.
#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub response: T,
}

/// Account info wrapper for /account/me endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct AccountMeResponse {
    pub account: AccountInfo,
}

/// Account information response.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfo {
    pub id: String,
    pub username: String,
    pub display_name: Option<String>,
    pub following: Option<bool>,
    pub subscribed: Option<bool>,
    pub timeline_stats: Option<TimelineStats>,
}

/// Timeline statistics.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineStats {
    pub image_count: Option<u64>,
    pub video_count: Option<u64>,
}

/// Timeline response containing posts and media.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineResponse {
    #[serde(default)]
    pub posts: Vec<Post>,
    #[serde(default)]
    pub account_media: Vec<AccountMedia>,
    #[serde(default)]
    pub account_media_bundles: Vec<MediaBundle>,
}

/// A post from the timeline.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Post {
    pub id: String,
    pub account_id: String,
    pub created_at: i64,
    #[serde(default)]
    pub attachments: Vec<Attachment>,
}

/// Account media item.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountMedia {
    pub id: String,
    pub account_id: String,
    pub preview_id: Option<String>,
    #[serde(default)]
    pub access: bool,
    pub media: Option<MediaDetails>,
    pub preview: Option<MediaDetails>,
}

/// Detailed media information.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaDetails {
    pub id: String,
    pub created_at: i64,
    pub mimetype: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    #[serde(default)]
    pub locations: Vec<MediaLocation>,
    #[serde(default)]
    pub variants: Vec<MediaVariant>,
}

/// Media file location.
#[derive(Debug, Clone, Deserialize)]
pub struct MediaLocation {
    pub location: String,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Media variant (different resolutions).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaVariant {
    pub id: String,
    pub mimetype: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    #[serde(default)]
    pub locations: Vec<MediaLocation>,
    pub updated_at: Option<i64>,
}

/// Media bundle containing multiple media items.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaBundle {
    pub id: String,
    pub account_id: String,
    #[serde(default)]
    pub account_media_ids: Vec<String>,
    pub preview_id: Option<String>,
    pub created_at: i64,
}

/// Post attachment.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub content_id: String,
    pub content_type: i32,
}

/// Messages response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessagesResponse {
    #[serde(default)]
    pub messages: Vec<Message>,
    #[serde(default)]
    pub account_media: Vec<AccountMedia>,
    #[serde(default)]
    pub account_media_bundles: Vec<MediaBundle>,
}

/// A direct message.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: String,
    pub sender_id: String,
    pub created_at: i64,
    #[serde(default)]
    pub attachments: Vec<Attachment>,
}

/// Message group information.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageGroup {
    pub id: String,
    #[serde(default)]
    pub users: Vec<GroupUser>,
}

/// User in a message group.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupUser {
    pub user_id: String,
    pub username: Option<String>,
}

/// Groups response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupsResponse {
    #[serde(default)]
    pub groups: Vec<MessageGroup>,
}

/// Post response for single post download.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostResponse {
    #[serde(default)]
    pub posts: Vec<Post>,
    #[serde(default)]
    pub account_media: Vec<AccountMedia>,
    #[serde(default)]
    pub account_media_bundles: Vec<MediaBundle>,
}

/// Collections/orders response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionsResponse {
    #[serde(default)]
    pub account_media_orders: Vec<MediaOrder>,
}

/// Media order (purchased item).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaOrder {
    pub account_id: String,
    pub account_media_id: String,
    #[serde(rename = "type")]
    pub order_type: i32,
    pub created_at: i64,
    pub bundle_id: Option<String>,
}

/// Device ID response.
#[derive(Debug, Deserialize)]
pub struct DeviceIdResponse {
    #[serde(rename = "deviceId")]
    pub device_id: String,
}

/// WebSocket authentication message.
#[derive(Debug, Serialize)]
pub struct WsAuthMessage {
    pub t: i32,
    pub d: String,
}

/// WebSocket response.
#[derive(Debug, Deserialize)]
pub struct WsResponse {
    pub t: i32,
    pub d: String,
}

/// WebSocket session data.
#[derive(Debug, Deserialize)]
pub struct WsSessionData {
    pub session: WsSession,
}

/// WebSocket session.
#[derive(Debug, Deserialize)]
pub struct WsSession {
    pub id: String,
}

/// Media info batch response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaInfoResponse {
    #[serde(default)]
    pub account_media: Vec<AccountMedia>,
}
