//! Extracted media and its presentation and source types.

use super::Image;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaPresentation {
    // #[serde(rename = "__typename")]
    // pub type_name: String,
    pub id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoSource {
    // #[serde(rename = "__typename")]
    // pub type_name: String,
    pub format: String,
    pub height: i64,
    pub mime_type: String,
    pub url: String,
    pub width: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSource {
    // #[serde(rename = "__typename")]
    // pub type_name: String,
    pub filesize: i64,
    pub format: String,
    pub mime_type: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "__typename")]
pub enum Media {
    MediaImage {
        alt: Option<String>,
        id: Option<String>,
        media_content_type: String,
        presentation: Option<MediaPresentation>,
        preview_image: Option<Image>,
        image: Option<Image>,
    },
    ExternalVideo {
        alt: Option<String>,
        id: Option<String>,
        media_content_type: String,
        presentation: Option<MediaPresentation>,
        preview_image: Option<Image>,
        embed_url: String,
        host: String,
        origin_url: String,
    },
    Video {
        alt: Option<String>,
        id: Option<String>,
        media_content_type: String,
        presentation: Option<MediaPresentation>,
        preview_image: Option<Image>,
        sources: Vec<VideoSource>,
    },
    Model3d {
        alt: Option<String>,
        id: Option<String>,
        media_content_type: String,
        presentation: Option<MediaPresentation>,
        preview_image: Option<Image>,
        sources: Vec<ModelSource>,
    },
}
