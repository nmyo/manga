use serde::Serialize;

pub(crate) const DEFAULT_READER_CACHE_LIMIT_BYTES: u64 = 512 * 1024 * 1024;
pub(crate) const MIN_READER_CACHE_LIMIT_BYTES: u64 = 128 * 1024 * 1024;
pub(crate) const MAX_READER_CACHE_LIMIT_BYTES: u64 = 2048 * 1024 * 1024;

#[derive(Debug, Clone)]
pub(crate) struct ReaderManifest {
    pub(crate) endpoint: String,
    pub(crate) read_id: String,
    pub(crate) read_id_number: u32,
    pub(crate) pages: Vec<ReaderPage>,
}

#[derive(Debug, Clone)]
pub(crate) struct ReaderPage {
    pub(crate) index: u32,
    pub(crate) page_name: String,
    pub(crate) source_url: String,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum ReaderPageMaterializeOrigin {
    Visible,
    Prefetch,
}

impl ReaderPageMaterializeOrigin {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Visible => "visible",
            Self::Prefetch => "prefetch",
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ComicReadManifestResult {
    pub endpoint: String,
    #[serde(rename = "readId")]
    pub read_id: String,
    #[serde(rename = "pageCount")]
    pub page_count: u32,
    #[serde(rename = "cacheLimitBytes")]
    pub cache_limit_bytes: u64,
}

#[derive(Debug, Serialize)]
pub struct ComicReadPageResult {
    #[serde(rename = "readId")]
    pub read_id: String,
    pub index: u32,
    pub path: String,
    pub width: u32,
    pub height: u32,
    #[serde(rename = "aspectRatio")]
    pub aspect_ratio: f32,
    #[serde(rename = "isCached")]
    pub is_cached: bool,
}

#[derive(Debug, Serialize)]
pub struct ReaderCacheStatsResult {
    #[serde(rename = "cacheDir")]
    pub cache_dir: String,
    #[serde(rename = "totalBytes")]
    pub total_bytes: u64,
    #[serde(rename = "fileCount")]
    pub file_count: u32,
    #[serde(rename = "cacheLimitBytes")]
    pub cache_limit_bytes: u64,
    #[serde(rename = "cacheTrimBytes")]
    pub cache_trim_bytes: u64,
}

impl ReaderManifest {
    pub(crate) fn to_result(&self) -> ComicReadManifestResult {
        ComicReadManifestResult {
            endpoint: self.endpoint.clone(),
            read_id: self.read_id.clone(),
            page_count: self.pages.len() as u32,
            cache_limit_bytes: DEFAULT_READER_CACHE_LIMIT_BYTES,
        }
    }

    pub(crate) fn page_count(&self) -> u32 {
        self.pages.len() as u32
    }
}
