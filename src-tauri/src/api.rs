use aes::Aes256;
use base64::prelude::{Engine as _, BASE64_STANDARD};
use ecb::cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyInit};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

type Aes256EcbDec = ecb::Decryptor<Aes256>;
type ApiResult<T> = Result<T, ApiError>;

const API_VERSION: &str = "1.8.2";
const API_SECRET: &str = "185Hcomic3PAPP7R";
const DEFAULT_API_ENDPOINT: &str = API_ENDPOINTS[0];
const API_ENDPOINTS: [&str; 5] = [
    "https://www.cdnhth.club",
    "https://www.cdnmhwscc.vip",
    "https://www.jmapiproxyxxx.vip",
    "https://www.cdnxxx-proxy.xyz",
    "https://www.jmeadpoolcdn.life",
];

#[derive(Debug)]
pub enum ApiErrorKind {
    Api,
    Client,
    Decode,
    Decrypt,
    Empty,
    Http,
    MissingData,
    Network,
    Payload,
    UnsupportedEndpoint,
}

#[derive(Debug)]
pub struct ApiError {
    kind: ApiErrorKind,
    message: String,
}

impl ApiError {
    fn new(kind: ApiErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:?}: {}", self.kind, self.message)
    }
}

impl std::error::Error for ApiError {}

#[derive(Debug, Deserialize)]
struct ApiResponse<T> {
    code: i32,
    data: Option<T>,
    #[serde(rename = "errorMsg")]
    error_msg: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SearchPayload {
    search_query: String,
    #[serde(deserialize_with = "deserialize_u32_from_string_or_number")]
    total: u32,
    redirect_aid: Option<String>,
    content: Vec<SearchContentItem>,
}

#[derive(Debug, Deserialize)]
struct SearchContentItem {
    id: String,
    author: String,
    description: Option<String>,
    name: String,
    image: String,
    category: Option<SearchCategory>,
    category_sub: Option<SearchCategory>,
    update_at: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct SearchCategory {
    title: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RemoteSettingPayload {
    img_host: String,
}

#[derive(Debug, Serialize)]
pub struct RemoteSettingResult {
    pub endpoint: String,
    #[serde(rename = "imgHost")]
    pub img_host: String,
}

#[derive(Debug, Serialize)]
pub struct SearchAlbumsResult {
    pub query: String,
    pub page: u32,
    pub total: u32,
    pub endpoint: Option<String>,
    #[serde(rename = "redirectAid")]
    pub redirect_aid: Option<String>,
    pub items: Vec<SearchAlbum>,
}

#[derive(Debug, Serialize)]
pub struct SearchAlbum {
    pub id: String,
    pub title: String,
    pub author: String,
    pub description: String,
    pub image: String,
    pub tags: Vec<String>,
    pub href: String,
    pub updated_at: Option<i64>,
    #[serde(rename = "isRedirect")]
    pub is_redirect: bool,
}

struct ApiAuth {
    ts: u64,
    token: String,
    tokenparam: String,
}

impl ApiAuth {
    fn current() -> Self {
        let ts = current_timestamp();

        Self {
            ts,
            token: md5_hex(&format!("{ts}{API_SECRET}")),
            tokenparam: format!("{ts},{API_VERSION}"),
        }
    }
}

pub async fn get_remote_setting(endpoint: Option<String>) -> ApiResult<RemoteSettingResult> {
    let endpoint = resolve_api_endpoint(endpoint)?;
    let client = build_http_client()?;
    let auth = ApiAuth::current();
    let setting = request_remote_setting(&client, endpoint, &auth).await?;

    Ok(RemoteSettingResult {
        endpoint: endpoint.to_string(),
        img_host: setting.img_host,
    })
}

pub async fn search_comics(
    query: String,
    page: Option<u32>,
    endpoint: Option<String>,
) -> ApiResult<SearchAlbumsResult> {
    let page = page.unwrap_or(1);
    let query = query.trim().to_string();
    let endpoint = resolve_api_endpoint(endpoint)?;

    if query.is_empty() {
        return Ok(SearchAlbumsResult {
            query,
            page,
            total: 0,
            endpoint: None,
            redirect_aid: None,
            items: Vec::new(),
        });
    }

    let client = build_http_client()?;
    let auth = ApiAuth::current();
    let img_host = match request_remote_setting(&client, endpoint, &auth).await {
        Ok(setting) => Some(setting.img_host),
        Err(error) => {
            eprintln!("Failed to load remote setting for search covers: {error}");
            None
        }
    };

    request_search(&client, endpoint, &query, page, &auth, img_host.as_deref()).await
}

fn build_http_client() -> ApiResult<reqwest::Client> {
    reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(5))
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .map_err(|error| ApiError::new(ApiErrorKind::Client, error.to_string()))
}

async fn request_remote_setting(
    client: &reqwest::Client,
    endpoint: &str,
    auth: &ApiAuth,
) -> ApiResult<RemoteSettingPayload> {
    request_api_data::<RemoteSettingPayload>(client, endpoint, "setting", &[], auth).await
}

async fn request_search(
    client: &reqwest::Client,
    endpoint: &str,
    query: &str,
    page: u32,
    auth: &ApiAuth,
    img_host: Option<&str>,
) -> ApiResult<SearchAlbumsResult> {
    let mut payload: SearchPayload = request_api_data(
        client,
        endpoint,
        "search",
        &[
            ("page", page.to_string()),
            ("o", "mr".to_string()),
            ("search_query", query.to_string()),
        ],
        auth,
    )
    .await?;

    let items = payload
        .content
        .into_iter()
        .map(|item| {
            let mut tags = Vec::new();
            let image = cover_image_url(img_host, &item.id).unwrap_or(item.image);

            if let Some(title) = item.category.and_then(|category| category.title) {
                if !title.is_empty() {
                    tags.push(title);
                }
            }

            if let Some(title) = item.category_sub.and_then(|category| category.title) {
                if !title.is_empty() && !tags.contains(&title) {
                    tags.push(title);
                }
            }

            SearchAlbum {
                href: format!("{endpoint}/album/{}", item.id),
                id: item.id,
                title: item.name,
                author: item.author,
                description: item.description.unwrap_or_default(),
                image,
                tags,
                updated_at: item.update_at,
                is_redirect: false,
            }
        })
        .collect::<Vec<_>>();

    let redirect_aid = payload.redirect_aid.take();
    let items = if items.is_empty() {
        redirect_aid
            .clone()
            .map(|id| {
                vec![SearchAlbum {
                    href: String::new(),
                    id: id.clone(),
                    title: format!("JM{id}"),
                    author: String::new(),
                    description: String::new(),
                    image: String::new(),
                    tags: Vec::new(),
                    updated_at: None,
                    is_redirect: true,
                }]
            })
            .unwrap_or(items)
    } else {
        items
    };

    Ok(SearchAlbumsResult {
        query: payload.search_query,
        page,
        total: payload.total,
        endpoint: Some(endpoint.to_string()),
        redirect_aid,
        items,
    })
}

async fn request_api_data<T>(
    client: &reqwest::Client,
    endpoint: &str,
    path: &str,
    query: &[(&str, String)],
    auth: &ApiAuth,
) -> ApiResult<T>
where
    T: DeserializeOwned,
{
    let request_name = format!("{endpoint}/{path}");
    let url = format!("{endpoint}/{path}");
    let query = query
        .iter()
        .map(|(key, value)| (*key, value.as_str()))
        .collect::<Vec<_>>();

    let response = client
        .get(url)
        .header("accept", "application/json")
        .header("token", &auth.token)
        .header("tokenparam", &auth.tokenparam)
        .query(&query)
        .send()
        .await
        .map_err(|error| {
            ApiError::new(ApiErrorKind::Network, format!("{request_name}: {error}"))
        })?;

    if !response.status().is_success() {
        return Err(ApiError::new(
            ApiErrorKind::Http,
            format!("{request_name}: API returned HTTP {}", response.status()),
        ));
    }

    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
        .to_string();
    let body = response.text().await.map_err(|error| {
        ApiError::new(ApiErrorKind::Network, format!("{request_name}: {error}"))
    })?;
    let body = body.trim();

    if body.is_empty() {
        return Err(ApiError::new(
            ApiErrorKind::Empty,
            format!("{request_name}: API returned an empty response"),
        ));
    }

    let envelope: ApiResponse<serde_json::Value> = serde_json::from_str(body).map_err(|error| {
        ApiError::new(
            ApiErrorKind::Decode,
            format!(
                "{request_name}: Invalid API response ({content_type}): {error}. Body starts with: {}",
                response_preview(body)
            ),
        )
    })?;

    if envelope.code != 200 {
        return Err(ApiError::new(
            ApiErrorKind::Api,
            envelope
                .error_msg
                .map(|message| format!("{request_name}: {message}"))
                .unwrap_or_else(|| format!("{request_name}: API returned code {}", envelope.code)),
        ));
    }

    let data = envelope.data.ok_or_else(|| {
        ApiError::new(
            ApiErrorKind::MissingData,
            format!("{request_name}: API response did not include data"),
        )
    })?;

    match data {
        serde_json::Value::String(encrypted) => {
            let decrypted = decrypt_data(&encrypted, auth.ts).map_err(|error| {
                ApiError::new(ApiErrorKind::Decrypt, format!("{request_name}: {error}"))
            })?;
            serde_json::from_str(&decrypted).map_err(|error| {
                ApiError::new(
                    ApiErrorKind::Payload,
                    format!(
                        "{request_name}: Invalid payload: {error}. Payload starts with: {}",
                        response_preview(&decrypted)
                    ),
                )
            })
        }
        value => serde_json::from_value(value).map_err(|error| {
            ApiError::new(
                ApiErrorKind::Payload,
                format!("{request_name}: Invalid payload: {error}"),
            )
        }),
    }
}

fn decrypt_data(data: &str, ts: u64) -> Result<String, String> {
    let encrypted = BASE64_STANDARD
        .decode(data)
        .map_err(|error| format!("Invalid encrypted data: {error}"))?;
    let key = md5_hex(&format!("{ts}{API_SECRET}"));
    let decrypted = Aes256EcbDec::new_from_slice(key.as_bytes())
        .map_err(|error| format!("Invalid AES key: {error}"))?
        .decrypt_padded_vec_mut::<Pkcs7>(&encrypted)
        .map_err(|error| format!("Failed to decrypt response: {error}"))?;

    String::from_utf8(decrypted).map_err(|error| format!("Invalid decrypted text: {error}"))
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn md5_hex(input: &str) -> String {
    format!("{:x}", md5::compute(input))
}

fn deserialize_u32_from_string_or_number<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;

    match value {
        serde_json::Value::Number(number) => number
            .as_u64()
            .and_then(|value| u32::try_from(value).ok())
            .ok_or_else(|| serde::de::Error::custom("expected a valid u32 number")),
        serde_json::Value::String(value) => value
            .parse::<u32>()
            .map_err(|error| serde::de::Error::custom(format!("expected a u32 string: {error}"))),
        _ => Err(serde::de::Error::custom("expected a u32 number or string")),
    }
}

fn resolve_api_endpoint(endpoint: Option<String>) -> ApiResult<&'static str> {
    let Some(endpoint) = endpoint else {
        return Ok(DEFAULT_API_ENDPOINT);
    };
    let endpoint = endpoint.trim().trim_end_matches('/');

    if endpoint.is_empty() {
        return Ok(DEFAULT_API_ENDPOINT);
    }

    API_ENDPOINTS
        .iter()
        .copied()
        .find(|candidate| *candidate == endpoint)
        .ok_or_else(|| {
            ApiError::new(
                ApiErrorKind::UnsupportedEndpoint,
                format!("Unsupported API endpoint: {endpoint}"),
            )
        })
}

fn cover_image_url(img_host: Option<&str>, comic_id: &str) -> Option<String> {
    let img_host = img_host?.trim().trim_end_matches('/');

    if img_host.is_empty() {
        return None;
    }

    Some(format!("{img_host}/media/albums/{comic_id}_3x4.jpg"))
}

fn response_preview(value: &str) -> String {
    value
        .chars()
        .take(180)
        .collect::<String>()
        .replace('\n', "\\n")
}
