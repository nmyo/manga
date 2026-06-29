use crate::plugin_codec::decode_setting_payload;
use aes::Aes256;
use base64::prelude::{Engine as _, BASE64_STANDARD};
use ecb::cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyInit};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::{Mutex, OnceLock};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

type Aes256EcbDec = ecb::Decryptor<Aes256>;
pub(crate) type ApiResult<T> = Result<T, ApiError>;

const API_VERSION: &str = "2.0.20";
const API_SECRET: &str = "185Hcomic3PAPP7R";
const DEFAULT_API_ENDPOINT: &str = FALLBACK_API_ENDPOINTS[0];
const FALLBACK_API_ENDPOINTS: [&str; 2] = ["https://www.cdnhth.club", "https://www.cdnhjk.net"];
const HOST_CONFIG_AES_SEED: &str = "diosfjckwpqpdfjkvnqQjsik";
const HOST_CONFIG_URLS: [&str; 2] = [
    "https://rup4a04-c02.tos-cn-hongkong.bytepluses.com/newsvr-2025.txt",
    "https://rup4a04-c01.tos-ap-southeast-1.bytepluses.com/newsvr-2025.txt",
];
const UNSUPPORTED_HOME_SECTION_TITLES: [&str; 4] = ["禁漫小说", "禁漫书库", "禁漫書庫", "禁漫小說"];
const HOME_SECTION_LIST_PAGE_SIZE: usize = 20;
const SEARCH_PAGE_SIZE: usize = 80;
const JM_PLUGIN_ID: &str = "bf99008d-010b-4f17-ac7c-61a9b57dc3d9";
static IMG_HOST_CACHE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
static SHARED_HTTP_CLIENT: OnceLock<Mutex<Option<reqwest::Client>>> = OnceLock::new();
static NETWORK_PROXY_CONFIG: OnceLock<Mutex<NetworkProxyConfig>> = OnceLock::new();
static JWT_TOKEN: OnceLock<Mutex<Option<String>>> = OnceLock::new();

#[derive(Debug)]
pub enum ApiErrorKind {
    Api,
    Cache,
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
    pub(crate) fn new(kind: ApiErrorKind, message: impl Into<String>) -> Self {
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
    #[serde(deserialize_with = "deserialize_u32_from_string_or_number")]
    total: u32,
    #[serde(default)]
    redirect_aid: Option<String>,
    #[serde(default)]
    content: Vec<SearchComicPayload>,
}

#[derive(Debug, Default, Deserialize)]
struct SearchComicPayload {
    #[serde(default, deserialize_with = "deserialize_string_from_any")]
    id: String,
    #[serde(default, deserialize_with = "deserialize_string_from_any")]
    author: String,
    #[serde(default, deserialize_with = "deserialize_optional_string_from_any")]
    description: Option<String>,
    #[serde(default, deserialize_with = "deserialize_string_from_any")]
    name: String,
    #[serde(default, deserialize_with = "deserialize_string_from_any")]
    image: String,
    #[serde(default)]
    category: Option<SearchCategory>,
    #[serde(default)]
    category_sub: Option<SearchCategory>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_i64_from_string_or_number"
    )]
    update_at: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_u32_from_any")]
    likes: u32,
    #[serde(
        default,
        alias = "totalViews",
        deserialize_with = "deserialize_u32_from_any"
    )]
    total_views: u32,
    #[serde(default, deserialize_with = "deserialize_string_vec_from_any")]
    tags: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_string_vec_from_any")]
    works: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_string_vec_from_any")]
    actors: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_bool_from_any")]
    liked: bool,
    #[serde(default, deserialize_with = "deserialize_bool_from_any")]
    is_favorite: bool,
}

#[derive(Debug, Default, Deserialize)]
struct SearchCategory {
    #[serde(default, deserialize_with = "deserialize_string_from_any")]
    id: String,
    title: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ComicListItemPayload {
    #[serde(default, deserialize_with = "deserialize_string_from_any")]
    id: String,
    #[serde(default, deserialize_with = "deserialize_string_from_any")]
    author: String,
    #[serde(default, deserialize_with = "deserialize_optional_string_from_any")]
    description: Option<String>,
    #[serde(default, deserialize_with = "deserialize_string_from_any")]
    name: String,
    #[serde(default, deserialize_with = "deserialize_string_from_any")]
    image: String,
    #[serde(default)]
    category: Option<SearchCategory>,
    #[serde(default)]
    category_sub: Option<SearchCategory>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_i64_from_string_or_number"
    )]
    update_at: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct HomeFeedSectionPayload {
    #[serde(deserialize_with = "deserialize_string_from_string_or_number")]
    id: String,
    title: String,
    slug: String,
    #[serde(rename = "type")]
    section_type: String,
    #[serde(deserialize_with = "deserialize_string_from_string_or_number")]
    filter_val: String,
    content: Vec<ComicListItemPayload>,
}

#[derive(Debug, Deserialize)]
struct WeekPayload {
    categories: Vec<WeekCategoryPayload>,
    #[serde(rename = "type")]
    types: Vec<WeekTypePayload>,
}

#[derive(Debug, Deserialize)]
struct WeekCategoryPayload {
    id: String,
    time: String,
    title: String,
}

#[derive(Debug, Deserialize)]
struct WeekTypePayload {
    id: String,
    title: String,
}

#[derive(Debug, Deserialize)]
struct WeekComicsPayload {
    #[serde(deserialize_with = "deserialize_u32_from_string_or_number")]
    total: u32,
    list: Vec<ComicListItemPayload>,
}

#[derive(Debug, Deserialize)]
struct PromoteListPayload {
    #[serde(default, deserialize_with = "deserialize_u32_from_any")]
    total: u32,
    #[serde(default)]
    list: Vec<ComicListItemPayload>,
}

#[derive(Debug, Deserialize)]
struct WeeklyUpdatePayload {
    #[serde(default)]
    list: Vec<ComicListItemPayload>,
}

#[derive(Debug, Deserialize)]
struct CategoryFilterPayload {
    #[serde(default, deserialize_with = "deserialize_u32_from_any")]
    total: u32,
    #[serde(default)]
    content: Vec<ComicListItemPayload>,
}

#[derive(Debug, Deserialize)]
struct FavoriteListPayload {
    #[serde(default, deserialize_with = "deserialize_u32_from_any")]
    total: u32,
    #[serde(default)]
    list: Vec<FavoriteComicPayload>,
    #[serde(default)]
    folder_list: Vec<FavoriteFolderPayload>,
}

#[derive(Debug, Deserialize)]
struct FavoriteComicPayload {
    #[serde(
        default,
        alias = "AID",
        alias = "aid",
        deserialize_with = "deserialize_string_from_any"
    )]
    id: String,
    #[serde(default, deserialize_with = "deserialize_string_from_any")]
    name: String,
    #[serde(default, deserialize_with = "deserialize_string_from_any")]
    author: String,
    #[serde(default, deserialize_with = "deserialize_optional_string_from_any")]
    description: Option<String>,
    #[serde(default, deserialize_with = "deserialize_string_from_any")]
    image: String,
    #[serde(default)]
    category: Option<SearchCategory>,
    #[serde(default)]
    category_sub: Option<SearchCategory>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_i64_from_string_or_number"
    )]
    update_at: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct FavoriteFolderPayload {
    #[serde(
        default,
        rename = "FID",
        alias = "id",
        alias = "folder_id",
        deserialize_with = "deserialize_string_from_any"
    )]
    id: String,
    #[serde(default, deserialize_with = "deserialize_string_from_any")]
    name: String,
}

#[derive(Debug, Deserialize)]
struct ComicDetailPayload {
    #[serde(deserialize_with = "deserialize_string_from_string_or_number")]
    id: String,
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default, deserialize_with = "deserialize_string_vec_from_any")]
    author: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_u32_from_any")]
    total_views: u32,
    #[serde(default, deserialize_with = "deserialize_u32_from_any")]
    likes: u32,
    #[serde(default, deserialize_with = "deserialize_u32_from_any")]
    comment_total: u32,
    #[serde(default, deserialize_with = "deserialize_string_vec_from_any")]
    tags: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_string_vec_from_any")]
    actors: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_string_vec_from_any")]
    works: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_bool_from_any")]
    is_favorite: bool,
    #[serde(default, deserialize_with = "deserialize_bool_from_any")]
    liked: bool,
    #[serde(default)]
    related_list: Vec<ComicDetailRelatedPayload>,
    #[serde(default)]
    series: Vec<ComicDetailSeriesPayload>,
    #[serde(default, deserialize_with = "deserialize_string_from_any")]
    series_id: String,
    #[serde(default, deserialize_with = "deserialize_u32_from_any")]
    price: u32,
    #[serde(default, deserialize_with = "deserialize_bool_from_any")]
    purchased: bool,
}

#[derive(Debug, Deserialize)]
struct ComicDetailRelatedPayload {
    #[serde(deserialize_with = "deserialize_string_from_string_or_number")]
    id: String,
    name: String,
    #[serde(default)]
    author: String,
    #[serde(default)]
    image: String,
}

#[derive(Debug, Deserialize)]
struct ComicDetailSeriesPayload {
    #[serde(deserialize_with = "deserialize_string_from_string_or_number")]
    id: String,
    name: String,
    #[serde(default, deserialize_with = "deserialize_string_from_any")]
    sort: String,
}

#[derive(Debug, Deserialize)]
struct CommentListPayload {
    #[serde(default)]
    list: Vec<CommentPayload>,
    #[serde(default, deserialize_with = "deserialize_u32_from_any")]
    total: u32,
}

#[derive(Debug, Deserialize)]
struct CommentPayload {
    #[serde(
        default,
        rename = "AID",
        deserialize_with = "deserialize_optional_string_from_any"
    )]
    aid: Option<String>,
    #[serde(
        default,
        rename = "CID",
        deserialize_with = "deserialize_string_from_any"
    )]
    cid: String,
    #[serde(
        default,
        rename = "UID",
        deserialize_with = "deserialize_string_from_any"
    )]
    uid: String,
    #[serde(default)]
    username: String,
    #[serde(default)]
    nickname: String,
    #[serde(default, deserialize_with = "deserialize_u32_from_any")]
    likes: u32,
    #[serde(default)]
    update_at: String,
    #[serde(default)]
    addtime: String,
    #[serde(
        default,
        rename = "parent_CID",
        deserialize_with = "deserialize_string_from_any"
    )]
    parent_cid: String,
    #[serde(default)]
    content: String,
    #[serde(default)]
    photo: String,
    #[serde(default, deserialize_with = "deserialize_bool_from_any")]
    spoiler: bool,
    #[serde(default)]
    replys: Option<Vec<CommentPayload>>,
}

#[derive(Debug, Deserialize)]
struct RemoteSettingPayload {
    #[serde(default, deserialize_with = "deserialize_string_from_any")]
    img_host: String,
}

#[derive(Clone, Debug, PartialEq)]
struct NetworkProxyConfig {
    mode: NetworkProxyMode,
    host: String,
    port: u16,
}

#[derive(Clone, Debug, PartialEq)]
enum NetworkProxyMode {
    Off,
    Http,
    Socks5,
}

impl Default for NetworkProxyConfig {
    fn default() -> Self {
        Self {
            mode: NetworkProxyMode::Off,
            host: "127.0.0.1".to_string(),
            port: 7890,
        }
    }
}

#[derive(Debug, Deserialize)]
struct HostConfigPayload {
    #[serde(default, rename = "Server")]
    server: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct LoginPayload {
    #[serde(default, deserialize_with = "deserialize_optional_string_from_any")]
    jwttoken: Option<String>,
    #[serde(default, deserialize_with = "deserialize_u32_from_any")]
    uid: u32,
    username: String,
    #[serde(default)]
    email: String,
    #[serde(default)]
    photo: String,
    #[serde(default, deserialize_with = "deserialize_u32_from_any")]
    coin: u32,
    #[serde(default, deserialize_with = "deserialize_u32_from_any")]
    album_favorites: u32,
    #[serde(default)]
    level_name: String,
    #[serde(default, deserialize_with = "deserialize_u32_from_any")]
    level: u32,
    #[serde(
        default,
        rename = "nextLevelExp",
        deserialize_with = "deserialize_u32_from_any"
    )]
    next_level_exp: u32,
    #[serde(default, deserialize_with = "deserialize_u32_from_any")]
    exp: u32,
    #[serde(
        default,
        rename = "expPercent",
        deserialize_with = "deserialize_f32_from_any"
    )]
    exp_percent: f32,
    #[serde(default, deserialize_with = "deserialize_u32_from_any")]
    album_favorites_max: u32,
}

#[derive(Debug, Deserialize)]
struct SignInDataPayload {
    daily_id: u32,
    #[serde(default, deserialize_with = "deserialize_u32_from_any")]
    three_days_coin: u32,
    #[serde(default, deserialize_with = "deserialize_u32_from_any")]
    three_days_exp: u32,
    #[serde(default, deserialize_with = "deserialize_u32_from_any")]
    seven_days_coin: u32,
    #[serde(default, deserialize_with = "deserialize_u32_from_any")]
    seven_days_exp: u32,
    #[serde(default)]
    event_name: String,
    #[serde(default)]
    background_pc: String,
    #[serde(default)]
    background_phone: String,
    #[serde(default, rename = "currentProgress")]
    current_progress: String,
    #[serde(default)]
    record: Vec<Vec<SignInRecordPayload>>,
}

#[derive(Debug, Deserialize)]
struct SignInRecordPayload {
    #[serde(default)]
    date: String,
    #[serde(default, deserialize_with = "deserialize_bool_from_any")]
    signed: bool,
    #[serde(default, deserialize_with = "deserialize_bool_from_any")]
    bonus: bool,
}

#[derive(Debug, Deserialize)]
struct SignInPayload {
    msg: String,
}

#[derive(Debug, Serialize)]
pub struct RemoteSettingResult {
    pub endpoint: String,
    #[serde(rename = "imgHost")]
    pub img_host: String,
}

#[derive(Debug, Serialize)]
pub struct ApiEndpointProbe {
    pub endpoint: String,
    pub available: bool,
    #[serde(rename = "latencyMs")]
    pub latency_ms: Option<u64>,
    #[serde(rename = "imgHost")]
    pub img_host: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LoginResult {
    pub endpoint: String,
    pub user: UserProfile,
}

#[derive(Debug, Serialize)]
pub struct UserProfile {
    pub id: u32,
    pub username: String,
    pub email: String,
    pub avatar: String,
    #[serde(rename = "avatarUrl")]
    pub avatar_url: String,
    pub level: u32,
    #[serde(rename = "levelName")]
    pub level_name: String,
    #[serde(rename = "currentLevelExp")]
    pub current_level_exp: u32,
    #[serde(rename = "nextLevelExp")]
    pub next_level_exp: u32,
    #[serde(rename = "expPercent")]
    pub exp_percent: f32,
    #[serde(rename = "currentCollectCount")]
    pub current_collect_count: u32,
    #[serde(rename = "maxCollectCount")]
    pub max_collect_count: u32,
    #[serde(rename = "jCoin")]
    pub j_coin: u32,
}

#[derive(Debug, Serialize)]
pub struct SignInDataResult {
    pub endpoint: String,
    #[serde(rename = "dailyId")]
    pub daily_id: u32,
    #[serde(rename = "threeDaysCoin")]
    pub three_days_coin: u32,
    #[serde(rename = "threeDaysExp")]
    pub three_days_exp: u32,
    #[serde(rename = "sevenDaysCoin")]
    pub seven_days_coin: u32,
    #[serde(rename = "sevenDaysExp")]
    pub seven_days_exp: u32,
    #[serde(rename = "eventName")]
    pub event_name: String,
    #[serde(rename = "currentProgress")]
    pub current_progress: String,
    #[serde(rename = "backgroundPc")]
    pub background_pc: String,
    #[serde(rename = "backgroundPhone")]
    pub background_phone: String,
    pub records: Vec<SignInRecord>,
}

#[derive(Debug, Serialize)]
pub struct SignInRecord {
    pub day: u32,
    pub date: String,
    pub signed: bool,
    pub bonus: bool,
}

#[derive(Debug, Serialize)]
pub struct SignInResult {
    pub endpoint: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct SearchResultContract {
    pub source: String,
    pub r#extern: HashMap<String, serde_json::Value>,
    pub scheme: SearchResultScheme,
    pub data: SearchResultData,
    pub paging: SearchPagingInfo,
    pub items: Vec<PluginComicListItem>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SearchResultScheme {
    pub version: String,
    #[serde(rename = "type")]
    pub scheme_type: String,
    pub source: String,
    pub list: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct SearchResultData {
    pub paging: SearchPagingInfo,
    pub items: Vec<PluginComicListItem>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SearchPagingInfo {
    pub page: u32,
    pub pages: u32,
    pub total: u32,
    #[serde(rename = "hasReachedMax")]
    pub has_reached_max: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct PluginComicListItem {
    pub source: String,
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub finished: bool,
    #[serde(rename = "likesCount")]
    pub likes_count: u32,
    #[serde(rename = "viewsCount")]
    pub views_count: u32,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    pub cover: PluginImageItem,
    pub metadata: Vec<PluginMetadataListItem>,
    pub raw: HashMap<String, serde_json::Value>,
    pub r#extern: HashMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PluginImageItem {
    pub id: String,
    pub url: String,
    pub name: String,
    pub path: String,
    pub r#extern: HashMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PluginMetadataListItem {
    #[serde(rename = "type")]
    pub metadata_type: String,
    pub name: String,
    pub value: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct HomeFeedResult {
    pub endpoint: String,
    pub sections: Vec<HomeFeedSection>,
}

#[derive(Debug, Serialize)]
pub struct HomeFeedSection {
    pub id: String,
    pub title: String,
    pub slug: String,
    #[serde(rename = "type")]
    pub section_type: String,
    #[serde(rename = "filterValue")]
    pub filter_value: String,
    pub items: Vec<FeedComic>,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum HomeSectionListMode {
    Promote,
    Weekly,
    Latest,
    Ranking,
}

#[derive(Debug, Serialize)]
pub struct HomeSectionListResult {
    pub endpoint: String,
    pub mode: HomeSectionListMode,
    pub page: u32,
    #[serde(rename = "pageSize")]
    pub page_size: u32,
    pub total: u32,
    #[serde(rename = "hasMore")]
    pub has_more: bool,
    pub title: String,
    pub items: Vec<FeedComic>,
}

#[derive(Debug, Serialize)]
pub struct WeekFiltersResult {
    pub endpoint: String,
    pub categories: Vec<WeekCategory>,
    pub types: Vec<WeekType>,
    #[serde(rename = "defaultCategoryId")]
    pub default_category_id: Option<String>,
    #[serde(rename = "defaultTypeId")]
    pub default_type_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct WeekItemsResult {
    pub endpoint: String,
    pub page: u32,
    pub total: u32,
    pub items: Vec<FeedComic>,
}

#[derive(Debug, Serialize)]
pub struct ComicDetailResult {
    pub endpoint: String,
    pub comic: ComicDetail,
}

#[derive(Debug, Serialize)]
pub struct FavoriteToggleResult {
    pub endpoint: String,
    pub favorited: bool,
}

#[derive(Debug, Serialize)]
pub struct FavoriteListResult {
    pub endpoint: String,
    pub page: u32,
    pub total: u32,
    #[serde(rename = "hasMore")]
    pub has_more: bool,
    pub folders: Vec<FavoriteFolder>,
    pub items: Vec<FeedComic>,
}

#[derive(Debug, Serialize)]
pub struct FavoriteFolder {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct ComicDetail {
    pub id: String,
    pub title: String,
    pub author: Vec<String>,
    pub description: String,
    #[serde(rename = "totalViews")]
    pub total_views: u32,
    pub likes: u32,
    #[serde(rename = "commentTotal")]
    pub comment_total: u32,
    pub tags: Vec<String>,
    pub actors: Vec<String>,
    pub works: Vec<String>,
    #[serde(rename = "isFavorite")]
    pub is_favorite: bool,
    pub liked: bool,
    #[serde(rename = "relatedList")]
    pub related_list: Vec<RelatedComic>,
    pub series: Vec<ComicChapter>,
    #[serde(rename = "seriesId")]
    pub series_id: String,
    pub price: u32,
    pub purchased: bool,
    pub image: String,
}

#[derive(Debug, Serialize)]
pub struct RelatedComic {
    pub id: String,
    pub title: String,
    pub author: String,
    pub image: String,
}

#[derive(Debug, Serialize)]
pub struct ComicChapter {
    pub id: String,
    pub title: String,
    pub sort: String,
}

#[derive(Debug, Serialize)]
pub struct ComicCommentsResult {
    pub endpoint: String,
    pub page: u32,
    pub total: u32,
    pub comments: Vec<ComicComment>,
}

#[derive(Debug, Serialize)]
pub struct ComicComment {
    pub id: String,
    #[serde(rename = "comicId")]
    pub comic_id: Option<String>,
    #[serde(rename = "userId")]
    pub user_id: String,
    pub username: String,
    pub nickname: String,
    pub content: String,
    #[serde(rename = "likeCount")]
    pub like_count: u32,
    pub time: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    pub avatar: String,
    #[serde(rename = "parentId")]
    pub parent_id: String,
    pub spoiler: bool,
    pub replies: Vec<ComicComment>,
}

#[derive(Debug, Serialize)]
pub struct WeekCategory {
    pub id: String,
    pub time: String,
    pub title: String,
    pub label: String,
}

#[derive(Debug, Serialize)]
pub struct WeekType {
    pub id: String,
    pub title: String,
}

#[derive(Debug, Serialize)]
pub struct FeedComic {
    pub id: String,
    pub title: String,
    pub author: String,
    pub description: String,
    pub image: String,
    pub tags: Vec<String>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<i64>,
}

pub(crate) struct ApiAuth {
    pub(crate) ts: String,
    pub(crate) token: String,
    pub(crate) tokenparam: String,
}

impl ApiAuth {
    pub(crate) fn current() -> Self {
        let ts = current_millis_timestamp();

        Self {
            token: md5_hex(&format!("{ts}{API_VERSION}")),
            tokenparam: format!("{ts},{API_VERSION}"),
            ts,
        }
    }
}

pub(crate) struct SettingAuth {
    ts: String,
    token: String,
    tokenparam: String,
}

impl SettingAuth {
    fn current() -> Self {
        let ts = current_seconds_timestamp();

        Self {
            token: md5_hex(&format!("{ts}{API_SECRET}")),
            tokenparam: format!("{ts},{API_VERSION}"),
            ts,
        }
    }
}

pub async fn get_remote_setting(endpoint: Option<String>) -> ApiResult<RemoteSettingResult> {
    let endpoint = resolve_api_endpoint(endpoint)?;
    let client = build_http_client()?;
    let auth = SettingAuth::current();
    let img_host = request_remote_img_host(&client, &endpoint, &auth).await?;

    Ok(RemoteSettingResult { endpoint, img_host })
}

pub async fn discover_api_endpoints() -> ApiResult<Vec<ApiEndpointProbe>> {
    let client = create_http_client()?;
    let mut candidates = discover_api_endpoint_candidates(&client).await?;

    if candidates.is_empty() {
        candidates.push(DEFAULT_API_ENDPOINT.to_string());
    }

    let auth = SettingAuth::current();
    let mut probes = Vec::with_capacity(candidates.len());

    for endpoint in candidates {
        let started_at = Instant::now();
        let result = request_remote_setting(&client, &endpoint, &auth).await;
        let latency_ms = started_at.elapsed().as_millis() as u64;

        probes.push(match result {
            Ok(setting) => ApiEndpointProbe {
                endpoint,
                available: true,
                latency_ms: Some(latency_ms),
                img_host: Some(setting.img_host),
                error: None,
            },
            Err(error) => ApiEndpointProbe {
                endpoint,
                available: false,
                latency_ms: None,
                img_host: None,
                error: Some(error.to_string()),
            },
        });
    }

    probes.sort_by(|left, right| match (left.available, right.available) {
        (true, true) => left
            .latency_ms
            .unwrap_or(u64::MAX)
            .cmp(&right.latency_ms.unwrap_or(u64::MAX)),
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        (false, false) => left.endpoint.cmp(&right.endpoint),
    });

    Ok(probes)
}

pub async fn search_comics(
    keyword: String,
    page: Option<u32>,
    extern_payload: Option<HashMap<String, serde_json::Value>>,
    endpoint: Option<String>,
) -> ApiResult<SearchResultContract> {
    let page = page.unwrap_or(1);
    let keyword = keyword.trim().to_string();
    let extern_payload = normalize_search_extern(extern_payload);
    let order = search_order_from_extern(&extern_payload);
    let endpoint = resolve_api_endpoint(endpoint)?;

    if keyword.is_empty() {
        return Ok(build_search_result(page, 0, Vec::new(), extern_payload));
    }

    let client = build_http_client()?;
    let setting_auth = SettingAuth::current();
    let api_auth = ApiAuth::current();
    let img_host = match request_remote_img_host(&client, &endpoint, &setting_auth).await {
        Ok(img_host) => Some(img_host),
        Err(error) => {
            eprintln!("Failed to load remote setting for search covers: {error}");
            None
        }
    };

    if page == 1 {
        if let Some(comic_id) = direct_search_comic_id(&keyword) {
            match request_comic_detail(&client, &endpoint, &comic_id, &api_auth).await {
                Ok(payload) => {
                    let item = search_payload_from_detail(payload);

                    return Ok(build_search_result(
                        page,
                        1,
                        vec![map_search_comic_item(item, img_host.as_deref())],
                        extern_payload,
                    ));
                }
                Err(error) => {
                    eprintln!("Failed direct search detail fallback for {comic_id}: {error}");
                }
            }
        }
    }

    request_search(
        &client,
        &endpoint,
        &keyword,
        page,
        &order,
        &api_auth,
        img_host.as_deref(),
        extern_payload,
    )
    .await
}

pub async fn get_home_feed(endpoint: Option<String>) -> ApiResult<HomeFeedResult> {
    let endpoint = resolve_api_endpoint(endpoint)?;
    let client = build_http_client()?;
    let setting_auth = SettingAuth::current();
    let api_auth = ApiAuth::current();
    let img_host = match request_remote_img_host(&client, &endpoint, &setting_auth).await {
        Ok(img_host) => Some(img_host),
        Err(error) => {
            eprintln!("Failed to load remote setting for home covers: {error}");
            None
        }
    };
    let sections = request_home_feed(&client, &endpoint, &api_auth, img_host.as_deref()).await?;

    Ok(HomeFeedResult { endpoint, sections })
}

#[allow(clippy::too_many_arguments)]
pub async fn get_home_section_list(
    mode: String,
    page: Option<u32>,
    section_id: Option<String>,
    section_title: Option<String>,
    _slug: Option<String>,
    _section_type: Option<String>,
    filter_value: Option<String>,
    category: Option<String>,
    week: Option<String>,
    order: Option<String>,
    endpoint: Option<String>,
) -> ApiResult<HomeSectionListResult> {
    let mode = parse_home_section_list_mode(&mode)?;
    let page = page.unwrap_or(1).max(1);
    let endpoint = resolve_api_endpoint(endpoint)?;
    let section_id = section_id.unwrap_or_default().trim().to_string();
    let section_title = section_title.unwrap_or_default().trim().to_string();
    let filter_value = filter_value.unwrap_or_default().trim().to_string();
    let category = category.unwrap_or_default().trim().to_string();
    let week = week.unwrap_or_default().trim().to_string();
    let order = order.unwrap_or_default().trim().to_string();
    let title = if section_title.is_empty() {
        default_home_section_list_title(mode)
    } else {
        section_title.clone()
    };

    let client = build_http_client()?;
    let setting_auth = SettingAuth::current();
    let api_auth = ApiAuth::current();
    let img_host_future = request_remote_img_host(&client, &endpoint, &setting_auth);
    let payload_future = request_home_section_list(
        &client,
        &endpoint,
        mode,
        page,
        &section_id,
        &filter_value,
        &category,
        &week,
        &order,
        &api_auth,
    );
    let (img_host_result, payload_result) = tokio::join!(img_host_future, payload_future);
    let img_host = match img_host_result {
        Ok(img_host) => Some(img_host),
        Err(error) => {
            eprintln!("Failed to load remote setting for home section list covers: {error}");
            None
        }
    };
    let payload = payload_result?;

    Ok(HomeSectionListResult {
        endpoint,
        mode,
        page,
        page_size: HOME_SECTION_LIST_PAGE_SIZE as u32,
        total: payload.total,
        has_more: payload.has_more,
        title,
        items: payload
            .items
            .into_iter()
            .map(|item| map_feed_comic(item, img_host.as_deref()))
            .collect(),
    })
}

pub async fn get_week_filters(endpoint: Option<String>) -> ApiResult<WeekFiltersResult> {
    let endpoint = resolve_api_endpoint(endpoint)?;
    let client = build_http_client()?;
    let auth = ApiAuth::current();
    let week = request_week_data(&client, &endpoint, &auth).await?;
    let categories = map_week_categories(week.categories);
    let types = map_week_types(week.types);

    Ok(WeekFiltersResult {
        endpoint,
        default_category_id: categories.first().map(|item| item.id.clone()),
        default_type_id: types.first().map(|item| item.id.clone()),
        categories,
        types,
    })
}

pub async fn get_week_items(
    page: Option<u32>,
    category_id: String,
    type_id: String,
    endpoint: Option<String>,
) -> ApiResult<WeekItemsResult> {
    let page = page.unwrap_or(1);
    let endpoint = resolve_api_endpoint(endpoint)?;
    let category_id = category_id.trim();
    let type_id = type_id.trim();

    if category_id.is_empty() || type_id.is_empty() {
        return Err(ApiError::new(
            ApiErrorKind::MissingData,
            "Week items need both category_id and type_id",
        ));
    }

    let client = build_http_client()?;
    let setting_auth = SettingAuth::current();
    let api_auth = ApiAuth::current();
    let img_host_future = request_remote_img_host(&client, &endpoint, &setting_auth);
    let payload_future =
        request_week_comics(&client, &endpoint, page, category_id, type_id, &api_auth);
    let (img_host_result, payload_result) = tokio::join!(img_host_future, payload_future);
    let img_host = match img_host_result {
        Ok(img_host) => Some(img_host),
        Err(error) => {
            eprintln!("Failed to load remote setting for weekly covers: {error}");
            None
        }
    };
    let payload = payload_result?;

    Ok(WeekItemsResult {
        endpoint,
        page,
        total: payload.total,
        items: payload
            .list
            .into_iter()
            .map(|item| map_feed_comic(item, img_host.as_deref()))
            .collect(),
    })
}

pub async fn get_comic_detail(
    comic_id: String,
    endpoint: Option<String>,
) -> ApiResult<ComicDetailResult> {
    let comic_id = comic_id.trim().to_string();

    if comic_id.is_empty() {
        return Err(ApiError::new(
            ApiErrorKind::MissingData,
            "Comic detail needs a comic_id",
        ));
    }

    let endpoint = resolve_api_endpoint(endpoint)?;
    let client = build_http_client()?;
    let setting_auth = SettingAuth::current();
    let api_auth = ApiAuth::current();
    let img_host_future = request_remote_img_host(&client, &endpoint, &setting_auth);
    let payload_future = request_comic_detail(&client, &endpoint, &comic_id, &api_auth);
    let (img_host_result, payload_result) = tokio::join!(img_host_future, payload_future);
    let img_host = match img_host_result {
        Ok(img_host) => Some(img_host),
        Err(error) => {
            eprintln!("Failed to load remote setting for detail images: {error}");
            None
        }
    };

    Ok(ComicDetailResult {
        endpoint,
        comic: map_comic_detail(payload_result?, img_host.as_deref()),
    })
}

pub async fn toggle_comic_favorite(
    comic_id: String,
    current_favorite: bool,
    endpoint: Option<String>,
) -> ApiResult<FavoriteToggleResult> {
    let comic_id = comic_id.trim().to_string();

    if comic_id.is_empty() {
        return Err(ApiError::new(
            ApiErrorKind::MissingData,
            "Favorite toggle needs a comic_id",
        ));
    }

    let endpoint = resolve_api_endpoint(endpoint)?;
    let client = build_http_client()?;
    let auth = ApiAuth::current();
    let request_name = format!("{endpoint}/favorite");
    let response = client
        .post(&request_name)
        .with_jm_headers(&request_name, &auth, true)?
        .form(&[("aid", comic_id.as_str())])
        .send()
        .await
        .map_err(|error| {
            ApiError::new(ApiErrorKind::Network, format!("{request_name}: {error}"))
        })?;

    let _: serde_json::Value = decode_api_response(response, &request_name, &auth).await?;

    Ok(FavoriteToggleResult {
        endpoint,
        favorited: !current_favorite,
    })
}

pub async fn get_favorite_comics(
    page: Option<u32>,
    folder_id: Option<String>,
    order: Option<String>,
    endpoint: Option<String>,
) -> ApiResult<FavoriteListResult> {
    let page = page.unwrap_or(1).max(1);
    let folder_id = folder_id.unwrap_or_default();
    let folder_id = folder_id.trim().to_string();
    let order = order.unwrap_or_else(|| "mr".to_string()).trim().to_string();
    let order = if order.is_empty() {
        "mr".to_string()
    } else {
        order
    };
    let endpoint = resolve_api_endpoint(endpoint)?;
    let client = build_http_client()?;
    let setting_auth = SettingAuth::current();
    let api_auth = ApiAuth::current();
    let img_host_future = request_remote_img_host(&client, &endpoint, &setting_auth);
    let payload_future =
        request_favorite_comics(&client, &endpoint, page, &folder_id, &order, &api_auth);
    let (img_host_result, payload_result) = tokio::join!(img_host_future, payload_future);
    let img_host = match img_host_result {
        Ok(img_host) => Some(img_host),
        Err(error) => {
            eprintln!("Failed to load remote setting for favorite covers: {error}");
            None
        }
    };
    let payload = payload_result?;
    let total = payload.total;
    let folders = payload
        .folder_list
        .into_iter()
        .filter(|folder| !folder.id.trim().is_empty())
        .map(|folder| FavoriteFolder {
            id: folder.id,
            name: folder.name,
        })
        .collect();
    let items = payload
        .list
        .into_iter()
        .filter(|item| !item.id.trim().is_empty())
        .map(|item| map_favorite_comic(item, img_host.as_deref()))
        .collect::<Vec<_>>();
    let has_more = if total > 0 {
        page.saturating_mul(20) < total
    } else {
        items.len() >= 20
    };

    Ok(FavoriteListResult {
        endpoint,
        page,
        total,
        has_more,
        folders,
        items,
    })
}

pub async fn get_comic_comments(
    comic_id: String,
    page: Option<u32>,
    endpoint: Option<String>,
) -> ApiResult<ComicCommentsResult> {
    let comic_id = comic_id.trim().to_string();
    let page = page.unwrap_or(1);

    if comic_id.is_empty() {
        return Err(ApiError::new(
            ApiErrorKind::MissingData,
            "Comic comments need a comic_id",
        ));
    }

    let endpoint = resolve_api_endpoint(endpoint)?;
    let client = build_http_client()?;
    let setting_auth = SettingAuth::current();
    let api_auth = ApiAuth::current();
    let img_host_future = request_remote_img_host(&client, &endpoint, &setting_auth);
    let payload_future = request_comic_comments(&client, &endpoint, &comic_id, page, &api_auth);
    let (img_host_result, payload_result) = tokio::join!(img_host_future, payload_future);
    let img_host = match img_host_result {
        Ok(img_host) => Some(img_host),
        Err(error) => {
            eprintln!("Failed to load remote setting for comment avatars: {error}");
            None
        }
    };
    let payload = payload_result?;

    Ok(ComicCommentsResult {
        endpoint,
        page,
        total: payload.total,
        comments: payload
            .list
            .into_iter()
            .map(|comment| map_comment(comment, img_host.as_deref()))
            .collect(),
    })
}

pub async fn login(
    username: String,
    password: String,
    endpoint: Option<String>,
) -> ApiResult<LoginResult> {
    let username = username.trim().to_string();

    if username.is_empty() || password.trim().is_empty() {
        return Err(ApiError::new(
            ApiErrorKind::MissingData,
            "Login needs both username and password",
        ));
    }

    let endpoint = resolve_api_endpoint(endpoint)?;
    clear_session();
    let client = build_http_client()?;
    let setting_auth = SettingAuth::current();
    let login_auth = ApiAuth::current();
    let img_host_future = request_remote_img_host(&client, &endpoint, &setting_auth);
    let payload_future = request_login(&client, &endpoint, &username, &password, &login_auth);
    let (img_host_result, payload_result) = tokio::join!(img_host_future, payload_future);
    let payload = payload_result?;
    set_jwt_token(payload.jwttoken.as_deref())?;
    let img_host = match img_host_result {
        Ok(img_host) => Some(img_host),
        Err(error) => {
            eprintln!("Failed to load remote setting for user avatar: {error}");
            None
        }
    };

    Ok(LoginResult {
        endpoint,
        user: map_login_user(payload, img_host.as_deref()),
    })
}

pub async fn get_sign_in_data(
    user_id: u32,
    endpoint: Option<String>,
) -> ApiResult<SignInDataResult> {
    if user_id == 0 {
        return Err(ApiError::new(
            ApiErrorKind::MissingData,
            "Sign-in data needs a user_id",
        ));
    }

    let endpoint = resolve_api_endpoint(endpoint)?;
    let client = build_http_client()?;
    let auth = ApiAuth::current();
    let payload = request_sign_in_data(&client, &endpoint, user_id, &auth).await?;

    Ok(SignInDataResult {
        endpoint,
        daily_id: payload.daily_id,
        three_days_coin: payload.three_days_coin,
        three_days_exp: payload.three_days_exp,
        seven_days_coin: payload.seven_days_coin,
        seven_days_exp: payload.seven_days_exp,
        event_name: payload.event_name,
        current_progress: payload.current_progress,
        background_pc: payload.background_pc,
        background_phone: payload.background_phone,
        records: map_sign_in_records(payload.record),
    })
}

pub async fn sign_in(
    user_id: u32,
    daily_id: u32,
    endpoint: Option<String>,
) -> ApiResult<SignInResult> {
    if user_id == 0 || daily_id == 0 {
        return Err(ApiError::new(
            ApiErrorKind::MissingData,
            "Sign-in needs both user_id and daily_id",
        ));
    }

    let endpoint = resolve_api_endpoint(endpoint)?;
    let client = build_http_client()?;
    let auth = ApiAuth::current();
    let payload = request_sign_in(&client, &endpoint, user_id, daily_id, &auth).await?;

    Ok(SignInResult {
        endpoint,
        message: payload.msg,
    })
}

pub fn clear_session() {
    if let Some(jwt_token) = JWT_TOKEN.get() {
        if let Ok(mut jwt_token) = jwt_token.lock() {
            *jwt_token = None;
        }
    }
}

pub fn configure_network_proxy(
    mode: String,
    host: Option<String>,
    port: Option<u16>,
) -> ApiResult<()> {
    let next_config = normalize_network_proxy_config(mode, host, port)?;
    let proxy_config =
        NETWORK_PROXY_CONFIG.get_or_init(|| Mutex::new(NetworkProxyConfig::default()));
    let mut proxy_config = proxy_config
        .lock()
        .map_err(|error| ApiError::new(ApiErrorKind::Client, error.to_string()))?;

    if *proxy_config == next_config {
        return Ok(());
    }

    *proxy_config = next_config;
    reset_http_client()?;
    clear_session();

    Ok(())
}

pub(crate) fn build_http_client() -> ApiResult<reqwest::Client> {
    let client = SHARED_HTTP_CLIENT.get_or_init(|| Mutex::new(None));
    let mut client = client
        .lock()
        .map_err(|error| ApiError::new(ApiErrorKind::Client, error.to_string()))?;

    if let Some(client) = client.as_ref() {
        return Ok(client.clone());
    }

    let next_client = create_http_client()?;
    *client = Some(next_client.clone());

    Ok(next_client)
}

fn create_http_client() -> ApiResult<reqwest::Client> {
    let mut builder = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(5))
        .timeout(std::time::Duration::from_secs(8));

    if let Some(proxy_url) = current_proxy_url()? {
        let proxy = reqwest::Proxy::all(&proxy_url).map_err(|error| {
            ApiError::new(
                ApiErrorKind::Client,
                format!("Invalid proxy {proxy_url}: {error}"),
            )
        })?;
        builder = builder.proxy(proxy);
    }

    builder
        .build()
        .map_err(|error| ApiError::new(ApiErrorKind::Client, error.to_string()))
}

fn reset_http_client() -> ApiResult<()> {
    let client = SHARED_HTTP_CLIENT.get_or_init(|| Mutex::new(None));
    let mut client = client
        .lock()
        .map_err(|error| ApiError::new(ApiErrorKind::Client, error.to_string()))?;
    *client = None;

    Ok(())
}

fn set_jwt_token(token: Option<&str>) -> ApiResult<()> {
    let token = token
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(str::to_string);
    let jwt_token = JWT_TOKEN.get_or_init(|| Mutex::new(None));
    let mut jwt_token = jwt_token
        .lock()
        .map_err(|error| ApiError::new(ApiErrorKind::Client, error.to_string()))?;

    *jwt_token = token;

    Ok(())
}

pub(crate) fn current_jwt_token() -> ApiResult<Option<String>> {
    let jwt_token = JWT_TOKEN.get_or_init(|| Mutex::new(None));
    jwt_token
        .lock()
        .map(|token| token.clone())
        .map_err(|error| ApiError::new(ApiErrorKind::Client, error.to_string()))
}

trait JmRequestBuilderExt {
    fn with_jm_headers(
        self,
        url: &str,
        auth: &ApiAuth,
        use_jwt: bool,
    ) -> ApiResult<reqwest::RequestBuilder>;
}

impl JmRequestBuilderExt for reqwest::RequestBuilder {
    fn with_jm_headers(
        self,
        url: &str,
        auth: &ApiAuth,
        use_jwt: bool,
    ) -> ApiResult<reqwest::RequestBuilder> {
        let builder = self
            .header("accept", "application/json")
            .header("token", &auth.token)
            .header("tokenparam", &auth.tokenparam)
            .header("user-agent", android_user_agent());
        let builder = if let Some(host) = request_url_host(url) {
            builder.header("Host", host)
        } else {
            builder
        };
        let builder = if use_jwt {
            if let Some(jwt) = current_jwt_token()? {
                builder.header("Authorization", format!("Bearer {jwt}"))
            } else {
                builder
            }
        } else {
            builder
        };

        Ok(builder)
    }
}

fn normalize_network_proxy_config(
    mode: String,
    host: Option<String>,
    port: Option<u16>,
) -> ApiResult<NetworkProxyConfig> {
    let default_config = NetworkProxyConfig::default();
    let mode = match mode.trim().to_ascii_lowercase().as_str() {
        "" | "off" | "none" | "disabled" => NetworkProxyMode::Off,
        "http" | "https" => NetworkProxyMode::Http,
        "socks" | "socks5" => NetworkProxyMode::Socks5,
        value => {
            return Err(ApiError::new(
                ApiErrorKind::UnsupportedEndpoint,
                format!("Unsupported proxy mode: {value}"),
            ));
        }
    };

    if mode == NetworkProxyMode::Off {
        return Ok(default_config);
    }

    let host = host
        .unwrap_or(default_config.host)
        .trim()
        .trim_end_matches('/')
        .to_string();
    let port = port.unwrap_or(default_config.port);

    if host.is_empty() {
        return Err(ApiError::new(
            ApiErrorKind::MissingData,
            "Proxy host is required",
        ));
    }

    if port == 0 {
        return Err(ApiError::new(
            ApiErrorKind::MissingData,
            "Proxy port must be greater than 0",
        ));
    }

    Ok(NetworkProxyConfig { mode, host, port })
}

pub(crate) fn current_proxy_url() -> ApiResult<Option<String>> {
    let proxy_config =
        NETWORK_PROXY_CONFIG.get_or_init(|| Mutex::new(NetworkProxyConfig::default()));
    let proxy_config = proxy_config
        .lock()
        .map_err(|error| ApiError::new(ApiErrorKind::Client, error.to_string()))?
        .clone();

    let scheme = match proxy_config.mode {
        NetworkProxyMode::Off => return Ok(None),
        NetworkProxyMode::Http => "http",
        NetworkProxyMode::Socks5 => "socks5h",
    };
    let host = if proxy_config.host.contains(':')
        && !proxy_config.host.starts_with('[')
        && !proxy_config.host.ends_with(']')
    {
        format!("[{}]", proxy_config.host)
    } else {
        proxy_config.host
    };

    Ok(Some(format!("{scheme}://{host}:{}", proxy_config.port)))
}

async fn request_remote_setting(
    client: &reqwest::Client,
    endpoint: &str,
    auth: &SettingAuth,
) -> ApiResult<RemoteSettingPayload> {
    let request_name = format!("{endpoint}/setting");
    let response = client
        .get(&request_name)
        .header("Tokenparam", &auth.tokenparam)
        .header("Token", &auth.token)
        .query(&[("app_img_shunt", "1"), ("t", auth.ts.as_str())])
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

    let body = response.text().await.map_err(|error| {
        ApiError::new(ApiErrorKind::Network, format!("{request_name}: {error}"))
    })?;

    decode_setting_payload::<RemoteSettingPayload>(body.trim(), &auth.ts).map_err(|error| {
        ApiError::new(
            ApiErrorKind::Payload,
            format!(
                "{request_name}: Invalid setting payload: {error}. Body starts with: {}",
                response_preview(&body)
            ),
        )
    })
}

async fn request_remote_img_host(
    client: &reqwest::Client,
    endpoint: &str,
    auth: &SettingAuth,
) -> ApiResult<String> {
    if let Some(img_host) = cached_img_host(endpoint) {
        return Ok(img_host);
    }

    let setting = request_remote_setting(client, endpoint, auth).await?;
    cache_img_host(endpoint, &setting.img_host);

    Ok(setting.img_host)
}

pub(crate) async fn resolve_cached_img_host(
    client: &reqwest::Client,
    endpoint: &str,
) -> ApiResult<String> {
    let auth = SettingAuth::current();

    request_remote_img_host(client, endpoint, &auth).await
}

async fn discover_api_endpoint_candidates(client: &reqwest::Client) -> ApiResult<Vec<String>> {
    let mut candidates = FALLBACK_API_ENDPOINTS
        .iter()
        .filter_map(|endpoint| normalize_api_endpoint(endpoint).ok())
        .collect::<Vec<_>>();

    match fetch_host_config(client).await {
        Ok(hosts) => {
            candidates.extend(
                hosts
                    .into_iter()
                    .filter_map(|host| normalize_api_endpoint(&host).ok()),
            );
        }
        Err(error) => {
            eprintln!("Failed to load JM host config, fallback endpoints only: {error}");
        }
    }

    let mut unique = Vec::new();
    for endpoint in candidates {
        if !unique.contains(&endpoint) {
            unique.push(endpoint);
        }
    }

    Ok(unique)
}

async fn fetch_host_config(client: &reqwest::Client) -> ApiResult<Vec<String>> {
    let mut last_error = None;

    for url in HOST_CONFIG_URLS {
        match fetch_host_config_from_url(client, url).await {
            Ok(hosts) => return Ok(hosts),
            Err(error) => last_error = Some(error),
        }
    }

    Err(last_error.unwrap_or_else(|| {
        ApiError::new(ApiErrorKind::Network, "JM host config urls are unavailable")
    }))
}

async fn fetch_host_config_from_url(client: &reqwest::Client, url: &str) -> ApiResult<Vec<String>> {
    let response = client
        .get(url)
        .header("accept", "text/plain,*/*")
        .send()
        .await
        .map_err(|error| ApiError::new(ApiErrorKind::Network, format!("{url}: {error}")))?;

    if !response.status().is_success() {
        return Err(ApiError::new(
            ApiErrorKind::Http,
            format!("{url}: host config returned HTTP {}", response.status()),
        ));
    }

    let body = response
        .text()
        .await
        .map_err(|error| ApiError::new(ApiErrorKind::Network, format!("{url}: {error}")))?;
    let normalized = body
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '+' | '/' | '='))
        .collect::<String>();
    let key = md5_hex(HOST_CONFIG_AES_SEED);
    let decrypted = decrypt_base64_with_key(&normalized, &key).map_err(|error| {
        ApiError::new(
            ApiErrorKind::Decrypt,
            format!("{url}: failed to decrypt host config: {error}"),
        )
    })?;
    let payload = serde_json::from_str::<HostConfigPayload>(&decrypted).map_err(|error| {
        ApiError::new(
            ApiErrorKind::Payload,
            format!("{url}: invalid host config payload: {error}"),
        )
    })?;

    Ok(payload.server)
}

fn cached_img_host(endpoint: &str) -> Option<String> {
    IMG_HOST_CACHE
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .ok()
        .and_then(|cache| cache.get(endpoint).cloned())
}

fn cache_img_host(endpoint: &str, img_host: &str) {
    if let Ok(mut cache) = IMG_HOST_CACHE
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
    {
        cache.insert(endpoint.to_string(), img_host.to_string());
    }
}

async fn request_search(
    client: &reqwest::Client,
    endpoint: &str,
    keyword: &str,
    page: u32,
    order: &str,
    auth: &ApiAuth,
    img_host: Option<&str>,
    extern_payload: HashMap<String, serde_json::Value>,
) -> ApiResult<SearchResultContract> {
    let mut payload: SearchPayload = request_api_data(
        client,
        endpoint,
        "search",
        &[
            ("page", page.to_string()),
            ("o", order.to_string()),
            ("search_query", keyword.to_string()),
        ],
        auth,
    )
    .await?;

    let items = payload
        .content
        .into_iter()
        .map(|item| map_search_comic_item(item, img_host))
        .collect::<Vec<_>>();

    let redirect_aid = payload.redirect_aid.take();
    let items = if items.is_empty() {
        redirect_aid
            .clone()
            .map(|id| {
                let item = SearchComicPayload {
                    id: id.clone(),
                    name: format!("JM{id}"),
                    ..SearchComicPayload::default()
                };

                vec![map_search_comic_item(item, img_host)]
            })
            .unwrap_or(items)
    } else {
        items
    };

    Ok(build_search_result(
        page,
        payload.total,
        items,
        extern_payload,
    ))
}

async fn request_home_feed(
    client: &reqwest::Client,
    endpoint: &str,
    auth: &ApiAuth,
    img_host: Option<&str>,
) -> ApiResult<Vec<HomeFeedSection>> {
    let payload: Vec<HomeFeedSectionPayload> =
        request_api_data(client, endpoint, "promote", &[], auth).await?;

    Ok(payload
        .into_iter()
        .filter(|section| !is_unsupported_home_section(&section.title))
        .map(|section| HomeFeedSection {
            id: section.id,
            title: section.title,
            slug: section.slug,
            section_type: section.section_type,
            filter_value: section.filter_val,
            items: section
                .content
                .into_iter()
                .map(|item| map_feed_comic(item, img_host))
                .collect(),
        })
        .collect())
}

struct HomeSectionListPayload {
    total: u32,
    has_more: bool,
    items: Vec<ComicListItemPayload>,
}

#[allow(clippy::too_many_arguments)]
async fn request_home_section_list(
    client: &reqwest::Client,
    endpoint: &str,
    mode: HomeSectionListMode,
    page: u32,
    section_id: &str,
    filter_value: &str,
    category: &str,
    week: &str,
    order: &str,
    auth: &ApiAuth,
) -> ApiResult<HomeSectionListPayload> {
    match mode {
        HomeSectionListMode::Promote => {
            request_promote_list(client, endpoint, page, section_id, filter_value, auth).await
        }
        HomeSectionListMode::Weekly => {
            request_weekly_update_list(client, endpoint, page, week, category, auth).await
        }
        HomeSectionListMode::Latest => request_latest_list(client, endpoint, page, auth).await,
        HomeSectionListMode::Ranking => {
            request_category_filter_list(client, endpoint, page, category, order, auth).await
        }
    }
}

async fn request_promote_list(
    client: &reqwest::Client,
    endpoint: &str,
    page: u32,
    section_id: &str,
    filter_value: &str,
    auth: &ApiAuth,
) -> ApiResult<HomeSectionListPayload> {
    const SOURCE_PAGE_SIZE: usize = 27;

    let id = parse_u32_or_default(section_id)
        .or_else(|| parse_u32_or_default(filter_value))
        .unwrap_or_default();
    let start = local_list_start(page);
    let mut source_page = (start / SOURCE_PAGE_SIZE) as u32;
    let offset = start % SOURCE_PAGE_SIZE;
    let mut total = 0;
    let mut source_has_more = true;
    let mut buffer = Vec::new();

    while buffer.len() < offset + HOME_SECTION_LIST_PAGE_SIZE && source_has_more {
        let payload = request_promote_source_page(client, endpoint, id, source_page, auth).await?;
        total = payload.total;
        let count = payload.list.len();
        let loaded_count = source_page as usize * SOURCE_PAGE_SIZE + count;
        source_has_more = count >= SOURCE_PAGE_SIZE
            && (payload.total == 0 || loaded_count < payload.total as usize);
        buffer.extend(payload.list);
        source_page = source_page.saturating_add(1);
    }

    let available = buffer.len().saturating_sub(offset);
    let has_more = if total > 0 {
        (page as usize * HOME_SECTION_LIST_PAGE_SIZE) < total as usize
    } else {
        available > HOME_SECTION_LIST_PAGE_SIZE || source_has_more
    };
    let items = buffer
        .into_iter()
        .skip(offset)
        .take(HOME_SECTION_LIST_PAGE_SIZE)
        .collect();

    Ok(HomeSectionListPayload {
        total,
        has_more,
        items,
    })
}

async fn request_promote_source_page(
    client: &reqwest::Client,
    endpoint: &str,
    id: u32,
    page: u32,
    auth: &ApiAuth,
) -> ApiResult<PromoteListPayload> {
    request_api_data(
        client,
        endpoint,
        "promote_list",
        &[("id", id.to_string()), ("page", page.to_string())],
        auth,
    )
    .await
}

async fn request_weekly_update_list(
    client: &reqwest::Client,
    endpoint: &str,
    page: u32,
    week: &str,
    category: &str,
    auth: &ApiAuth,
) -> ApiResult<HomeSectionListPayload> {
    const SOURCE_PAGE_SIZE: usize = 40;

    let start = local_list_start(page);
    let request_page = (start / SOURCE_PAGE_SIZE) as u32 + 1;
    let offset = start % SOURCE_PAGE_SIZE;
    let date = parse_u32_or_default(week).unwrap_or_else(current_china_weekday);
    let category = if category.is_empty() { "all" } else { category };
    let value: serde_json::Value = request_api_data(
        client,
        endpoint,
        "serialization",
        &[
            ("date", date.to_string()),
            ("type", category.to_string()),
            ("page", request_page.to_string()),
        ],
        auth,
    )
    .await?;

    if value
        .get("error")
        .and_then(|error| error.as_str())
        .map(|error| error == "没有资料")
        .unwrap_or(false)
    {
        return Ok(HomeSectionListPayload {
            total: 0,
            has_more: false,
            items: Vec::new(),
        });
    }

    let payload: WeeklyUpdatePayload = serde_json::from_value(value).map_err(|error| {
        ApiError::new(
            ApiErrorKind::Payload,
            format!("{endpoint}/serialization: Invalid payload: {error}"),
        )
    })?;
    let source_count = payload.list.len();
    let has_more =
        source_count > offset + HOME_SECTION_LIST_PAGE_SIZE || source_count >= SOURCE_PAGE_SIZE;
    let items = payload
        .list
        .into_iter()
        .skip(offset)
        .take(HOME_SECTION_LIST_PAGE_SIZE)
        .collect();

    Ok(HomeSectionListPayload {
        total: 0,
        has_more,
        items,
    })
}

async fn request_latest_list(
    client: &reqwest::Client,
    endpoint: &str,
    page: u32,
    auth: &ApiAuth,
) -> ApiResult<HomeSectionListPayload> {
    const SOURCE_PAGE_SIZE: usize = 80;

    let start = local_list_start(page);
    let request_page = (start / SOURCE_PAGE_SIZE) as u32;
    let offset = start % SOURCE_PAGE_SIZE;
    let items: Vec<ComicListItemPayload> = request_api_data(
        client,
        endpoint,
        "latest",
        &[("page", request_page.to_string())],
        auth,
    )
    .await?;
    let source_count = items.len();
    let has_more =
        source_count > offset + HOME_SECTION_LIST_PAGE_SIZE || source_count >= SOURCE_PAGE_SIZE;
    let items = items
        .into_iter()
        .skip(offset)
        .take(HOME_SECTION_LIST_PAGE_SIZE)
        .collect();

    Ok(HomeSectionListPayload {
        total: 0,
        has_more,
        items,
    })
}

async fn request_category_filter_list(
    client: &reqwest::Client,
    endpoint: &str,
    page: u32,
    category: &str,
    order: &str,
    auth: &ApiAuth,
) -> ApiResult<HomeSectionListPayload> {
    const SOURCE_PAGE_SIZE: usize = 80;

    let start = local_list_start(page);
    let request_page = (start / SOURCE_PAGE_SIZE) as u32;
    let offset = start % SOURCE_PAGE_SIZE;
    let category = if category.is_empty() {
        "latest"
    } else {
        category
    };
    let order = if order.is_empty() { "new" } else { order };
    let payload: CategoryFilterPayload = request_api_data(
        client,
        endpoint,
        "categories/filter",
        &[
            ("page", request_page.to_string()),
            ("c", category.to_string()),
            ("o", order.to_string()),
        ],
        auth,
    )
    .await?;
    let source_count = payload.content.len();
    let has_more = if payload.total > 0 {
        (page as usize * HOME_SECTION_LIST_PAGE_SIZE) < payload.total as usize
    } else {
        source_count > offset + HOME_SECTION_LIST_PAGE_SIZE || source_count >= SOURCE_PAGE_SIZE
    };
    let items = payload
        .content
        .into_iter()
        .skip(offset)
        .take(HOME_SECTION_LIST_PAGE_SIZE)
        .collect();

    Ok(HomeSectionListPayload {
        total: payload.total,
        has_more,
        items,
    })
}

fn is_unsupported_home_section(title: &str) -> bool {
    let title = title.trim();
    UNSUPPORTED_HOME_SECTION_TITLES.contains(&title)
}

fn parse_home_section_list_mode(value: &str) -> ApiResult<HomeSectionListMode> {
    match value.trim().to_ascii_lowercase().as_str() {
        "promote" | "promotelist" | "recommend" => Ok(HomeSectionListMode::Promote),
        "weekly" | "week" => Ok(HomeSectionListMode::Weekly),
        "latest" => Ok(HomeSectionListMode::Latest),
        "ranking" | "category" | "categories" | "timeranking" => Ok(HomeSectionListMode::Ranking),
        value => Err(ApiError::new(
            ApiErrorKind::MissingData,
            format!("Unsupported home section list mode: {value}"),
        )),
    }
}

fn default_home_section_list_title(mode: HomeSectionListMode) -> String {
    match mode {
        HomeSectionListMode::Promote => "推荐".to_string(),
        HomeSectionListMode::Weekly => "每周连载更新".to_string(),
        HomeSectionListMode::Latest => "最新".to_string(),
        HomeSectionListMode::Ranking => "分类更新".to_string(),
    }
}

fn local_list_start(page: u32) -> usize {
    page.saturating_sub(1) as usize * HOME_SECTION_LIST_PAGE_SIZE
}

fn current_china_weekday() -> u32 {
    const SECONDS_PER_DAY: u64 = 86_400;
    const CHINA_OFFSET_SECONDS: u64 = 8 * 60 * 60;

    let seconds = current_timestamp().saturating_add(CHINA_OFFSET_SECONDS);
    // 1970-01-01 is Thursday. Breeze uses Sunday=7, Monday=1.
    match ((seconds / SECONDS_PER_DAY) + 4) % 7 {
        0 => 7,
        value => value as u32,
    }
}

fn parse_u32_or_default(value: &str) -> Option<u32> {
    value.trim().parse::<u32>().ok()
}

async fn request_week_data(
    client: &reqwest::Client,
    endpoint: &str,
    auth: &ApiAuth,
) -> ApiResult<WeekPayload> {
    request_api_data(client, endpoint, "week", &[], auth).await
}

async fn request_week_comics(
    client: &reqwest::Client,
    endpoint: &str,
    page: u32,
    category_id: &str,
    type_id: &str,
    auth: &ApiAuth,
) -> ApiResult<WeekComicsPayload> {
    request_api_data(
        client,
        endpoint,
        "week/filter",
        &[
            ("page", page.to_string()),
            ("id", category_id.to_string()),
            ("type", type_id.to_string()),
        ],
        auth,
    )
    .await
}

async fn request_comic_detail(
    client: &reqwest::Client,
    endpoint: &str,
    comic_id: &str,
    auth: &ApiAuth,
) -> ApiResult<ComicDetailPayload> {
    let request_name = format!("{endpoint}/album");
    let value: serde_json::Value = request_api_data(
        client,
        endpoint,
        "album",
        &[("id", comic_id.to_string())],
        auth,
    )
    .await?;

    if value
        .as_object()
        .map(|object| object.is_empty() || !object.contains_key("name"))
        .unwrap_or(false)
    {
        return Err(ApiError::new(
            ApiErrorKind::Payload,
            format!("{request_name}: 当前条目可能是小说或书库内容，暂不支持漫画详情阅读"),
        ));
    }

    serde_json::from_value(value).map_err(|error| {
        ApiError::new(
            ApiErrorKind::Payload,
            format!("{request_name}: Invalid payload: {error}"),
        )
    })
}

async fn request_comic_comments(
    client: &reqwest::Client,
    endpoint: &str,
    comic_id: &str,
    page: u32,
    auth: &ApiAuth,
) -> ApiResult<CommentListPayload> {
    request_api_data(
        client,
        endpoint,
        "forum",
        &[
            ("page", page.to_string()),
            ("aid", comic_id.to_string()),
            ("mode", "manhua".to_string()),
        ],
        auth,
    )
    .await
}

async fn request_favorite_comics(
    client: &reqwest::Client,
    endpoint: &str,
    page: u32,
    folder_id: &str,
    order: &str,
    auth: &ApiAuth,
) -> ApiResult<FavoriteListPayload> {
    request_api_data(
        client,
        endpoint,
        "favorite",
        &[
            ("page", page.to_string()),
            ("folder_id", folder_id.to_string()),
            ("o", order.to_string()),
        ],
        auth,
    )
    .await
}

async fn request_login(
    client: &reqwest::Client,
    endpoint: &str,
    username: &str,
    password: &str,
    auth: &ApiAuth,
) -> ApiResult<LoginPayload> {
    request_api_form_data_with_jwt(
        client,
        endpoint,
        "login",
        vec![
            ("username".to_string(), username.to_string()),
            ("password".to_string(), password.to_string()),
        ],
        auth,
        false,
    )
    .await
}

async fn request_sign_in_data(
    client: &reqwest::Client,
    endpoint: &str,
    user_id: u32,
    auth: &ApiAuth,
) -> ApiResult<SignInDataPayload> {
    request_api_data(
        client,
        endpoint,
        "daily",
        &[("user_id", user_id.to_string())],
        auth,
    )
    .await
}

async fn request_sign_in(
    client: &reqwest::Client,
    endpoint: &str,
    user_id: u32,
    daily_id: u32,
    auth: &ApiAuth,
) -> ApiResult<SignInPayload> {
    request_api_form_data(
        client,
        endpoint,
        "daily_chk",
        vec![
            ("user_id".to_string(), user_id.to_string()),
            ("daily_id".to_string(), daily_id.to_string()),
        ],
        auth,
    )
    .await
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
        .with_jm_headers(&request_name, auth, true)?
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
            let decrypted = decrypt_data(&encrypted, &auth.ts).map_err(|error| {
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

async fn request_api_form_data<T>(
    client: &reqwest::Client,
    endpoint: &str,
    path: &str,
    fields: Vec<(String, String)>,
    auth: &ApiAuth,
) -> ApiResult<T>
where
    T: DeserializeOwned,
{
    request_api_form_data_with_jwt(client, endpoint, path, fields, auth, true).await
}

async fn request_api_form_data_with_jwt<T>(
    client: &reqwest::Client,
    endpoint: &str,
    path: &str,
    fields: Vec<(String, String)>,
    auth: &ApiAuth,
    use_jwt: bool,
) -> ApiResult<T>
where
    T: DeserializeOwned,
{
    let request_name = format!("{endpoint}/{path}");
    let url = format!("{endpoint}/{path}");

    let response = client
        .post(url)
        .with_jm_headers(&request_name, auth, use_jwt)?
        .form(&fields)
        .send()
        .await
        .map_err(|error| {
            ApiError::new(ApiErrorKind::Network, format!("{request_name}: {error}"))
        })?;

    decode_api_response(response, &request_name, auth).await
}

async fn decode_api_response<T>(
    response: reqwest::Response,
    request_name: &str,
    auth: &ApiAuth,
) -> ApiResult<T>
where
    T: DeserializeOwned,
{
    let status = response.status();

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

    if !status.is_success() {
        return Err(ApiError::new(
            ApiErrorKind::Api,
            envelope
                .error_msg
                .map(|message| format!("{request_name}: {message}"))
                .unwrap_or_else(|| format!("{request_name}: API returned HTTP {status}")),
        ));
    }

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
            let decrypted = decrypt_data(&encrypted, &auth.ts).map_err(|error| {
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

fn map_feed_comic(item: ComicListItemPayload, img_host: Option<&str>) -> FeedComic {
    let mut tags = Vec::new();

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

    FeedComic {
        image: cover_image_url(img_host, &item.id).unwrap_or(item.image),
        id: item.id,
        title: item.name,
        author: item.author,
        description: item.description.unwrap_or_default(),
        tags,
        updated_at: item.update_at,
    }
}

fn map_favorite_comic(item: FavoriteComicPayload, img_host: Option<&str>) -> FeedComic {
    let mut tags = Vec::new();

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

    FeedComic {
        image: cover_image_url(img_host, &item.id).unwrap_or(item.image),
        id: item.id,
        title: item.name,
        author: item.author,
        description: item.description.unwrap_or_default(),
        tags,
        updated_at: item.update_at,
    }
}

fn build_search_result(
    page: u32,
    total: u32,
    items: Vec<PluginComicListItem>,
    extern_payload: HashMap<String, serde_json::Value>,
) -> SearchResultContract {
    let has_reached_max = items.is_empty()
        || items.len() < SEARCH_PAGE_SIZE
        || (total > 0
            && ((page.saturating_sub(1) as usize) * SEARCH_PAGE_SIZE + items.len())
                >= total as usize);
    let paging = SearchPagingInfo {
        page,
        pages: page,
        total,
        has_reached_max,
    };
    let scheme = SearchResultScheme {
        version: "1.0.0".to_string(),
        scheme_type: "searchResult".to_string(),
        source: JM_PLUGIN_ID.to_string(),
        list: "comicGrid".to_string(),
    };
    let data = SearchResultData {
        paging: paging.clone(),
        items: items.clone(),
    };

    SearchResultContract {
        source: JM_PLUGIN_ID.to_string(),
        r#extern: extern_payload,
        scheme,
        data,
        paging,
        items,
    }
}

fn normalize_search_extern(
    extern_payload: Option<HashMap<String, serde_json::Value>>,
) -> HashMap<String, serde_json::Value> {
    let mut extern_payload = extern_payload.unwrap_or_default();
    let sort_by = extern_payload
        .get("sortBy")
        .and_then(json_value_to_u32)
        .unwrap_or(1);

    extern_payload.insert(
        "sortBy".to_string(),
        serde_json::Value::Number(serde_json::Number::from(sort_by)),
    );

    extern_payload
}

fn search_order_from_extern(extern_payload: &HashMap<String, serde_json::Value>) -> String {
    let order = extern_payload
        .get("sort")
        .and_then(json_value_to_string)
        .unwrap_or_default();

    if !order.trim().is_empty() {
        return order.trim().to_string();
    }

    match extern_payload.get("sortBy").and_then(json_value_to_u32) {
        Some(2) => "mv".to_string(),
        Some(3) => "mp".to_string(),
        Some(4) => "tf".to_string(),
        _ => "mr".to_string(),
    }
}

fn json_value_to_u32(value: &serde_json::Value) -> Option<u32> {
    match value {
        serde_json::Value::Number(number) => {
            number.as_u64().and_then(|value| u32::try_from(value).ok())
        }
        serde_json::Value::String(value) => value.trim().parse::<u32>().ok(),
        _ => None,
    }
}

fn json_value_to_string(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(value) => Some(value.to_string()),
        serde_json::Value::Number(value) => Some(value.to_string()),
        serde_json::Value::Bool(value) => Some(value.to_string()),
        _ => None,
    }
}

fn direct_search_comic_id(keyword: &str) -> Option<String> {
    let keyword = keyword.trim();

    if keyword.is_empty() {
        return None;
    }

    let lowercase = keyword.to_ascii_lowercase();
    let comic_id = if lowercase.starts_with("jm") {
        keyword.get(2..).unwrap_or_default().trim()
    } else {
        keyword
    };

    if comic_id
        .parse::<u32>()
        .ok()
        .filter(|value| *value >= 100)
        .is_some()
    {
        Some(comic_id.to_string())
    } else {
        None
    }
}

fn build_jm_cover_url(img_host: Option<&str>, comic_id: &str, image: &str) -> String {
    let image = image.trim();

    if image.starts_with("http://") || image.starts_with("https://") {
        return image.to_string();
    }

    let Some(img_host) = img_host else {
        return image.to_string();
    };
    let img_host = img_host.trim().trim_end_matches('/');

    if img_host.is_empty() {
        return image.to_string();
    }

    if image.starts_with('/') {
        return format!("{img_host}{image}");
    }

    if image.starts_with("media/") {
        return format!("{img_host}/{image}");
    }

    if comic_id.trim().is_empty() {
        image.to_string()
    } else {
        format!("{img_host}/media/albums/{comic_id}_3x4.jpg")
    }
}

fn search_payload_from_detail(payload: ComicDetailPayload) -> SearchComicPayload {
    SearchComicPayload {
        id: payload.id,
        author: payload.author.join("/"),
        description: Some(payload.description),
        name: payload.name,
        image: String::new(),
        category: None,
        category_sub: None,
        update_at: None,
        likes: payload.likes,
        total_views: payload.total_views,
        tags: payload.tags,
        works: payload.works,
        actors: payload.actors,
        liked: payload.liked,
        is_favorite: payload.is_favorite,
    }
}

fn non_empty_string(value: Option<&str>) -> Option<String> {
    let value = value?.trim();

    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn map_search_comic_item(item: SearchComicPayload, img_host: Option<&str>) -> PluginComicListItem {
    let id = item.id;
    let cover_url = build_jm_cover_url(img_host, &id, &item.image);
    let updated_at = item
        .update_at
        .map(|value| value.to_string())
        .unwrap_or_default();
    let author = item.author;
    let description = item.description.unwrap_or_default();
    let category = item.category;
    let category_sub = item.category_sub;
    let category_title = category
        .as_ref()
        .and_then(|category| non_empty_string(category.title.as_deref()));
    let category_sub_title = category_sub
        .as_ref()
        .and_then(|category| non_empty_string(category.title.as_deref()));
    let category_id = category.map(|category| category.id);
    let category_sub_id = category_sub.map(|category| category.id);
    let name = item.name;
    let tags = item.tags;
    let works = item.works;
    let actors = item.actors;
    let image = item.image;

    PluginComicListItem {
        source: JM_PLUGIN_ID.to_string(),
        id: id.clone(),
        title: name.clone(),
        subtitle: String::new(),
        finished: false,
        likes_count: item.likes,
        views_count: item.total_views,
        updated_at: updated_at.clone(),
        cover: PluginImageItem {
            id: id.clone(),
            url: cover_url,
            name: String::new(),
            path: format!("{id}.jpg"),
            r#extern: hashmap_from_pairs([(
                "path",
                serde_json::Value::String(format!("{id}.jpg")),
            )]),
        },
        metadata: search_metadata(
            author.as_str(),
            category_title.as_deref(),
            category_sub_title.as_deref(),
            &tags,
            &works,
            &actors,
        ),
        raw: search_raw(
            &id,
            &name,
            &author,
            &description,
            &updated_at,
            &image,
            category_id,
            category_title,
            category_sub_id,
            category_sub_title,
            item.liked,
            item.is_favorite,
            item.likes,
            item.total_views,
            &tags,
            &works,
            &actors,
        ),
        r#extern: HashMap::new(),
    }
}

fn search_metadata(
    author: &str,
    category_title: Option<&str>,
    category_sub_title: Option<&str>,
    tags: &[String],
    works: &[String],
    actors: &[String],
) -> Vec<PluginMetadataListItem> {
    let mut metadata = Vec::new();

    push_metadata(&mut metadata, "author", "作者", [author.to_string()]);
    push_metadata(
        &mut metadata,
        "categories",
        "分类",
        [
            category_title.unwrap_or_default().to_string(),
            category_sub_title.unwrap_or_default().to_string(),
        ],
    );
    push_metadata(&mut metadata, "tags", "标签", tags.iter().cloned());
    push_metadata(&mut metadata, "works", "作品", works.iter().cloned());
    push_metadata(&mut metadata, "actors", "角色", actors.iter().cloned());

    metadata
}

fn push_metadata<I>(
    metadata: &mut Vec<PluginMetadataListItem>,
    metadata_type: &str,
    name: &str,
    values: I,
) where
    I: IntoIterator<Item = String>,
{
    let value = values
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();

    if value.is_empty() {
        return;
    }

    metadata.push(PluginMetadataListItem {
        metadata_type: metadata_type.to_string(),
        name: name.to_string(),
        value,
    });
}

#[allow(clippy::too_many_arguments)]
fn search_raw(
    id: &str,
    name: &str,
    author: &str,
    description: &str,
    updated_at: &str,
    image: &str,
    category_id: Option<String>,
    category_title: Option<String>,
    category_sub_id: Option<String>,
    category_sub_title: Option<String>,
    liked: bool,
    is_favorite: bool,
    likes: u32,
    total_views: u32,
    tags: &[String],
    works: &[String],
    actors: &[String],
) -> HashMap<String, serde_json::Value> {
    hashmap_from_pairs([
        ("id", serde_json::Value::String(id.to_string())),
        ("author", serde_json::Value::String(author.to_string())),
        (
            "description",
            serde_json::Value::String(description.to_string()),
        ),
        ("name", serde_json::Value::String(name.to_string())),
        ("image", serde_json::Value::String(image.to_string())),
        (
            "category",
            serde_json::json!({
                "id": category_id.unwrap_or_default(),
                "title": category_title.unwrap_or_default()
            }),
        ),
        (
            "category_sub",
            serde_json::json!({
                "id": category_sub_id,
                "title": category_sub_title
            }),
        ),
        ("liked", serde_json::Value::Bool(liked)),
        ("is_favorite", serde_json::Value::Bool(is_favorite)),
        (
            "update_at",
            serde_json::Value::Number(serde_json::Number::from(
                updated_at.parse::<i64>().unwrap_or_default(),
            )),
        ),
        (
            "likes",
            serde_json::Value::Number(serde_json::Number::from(likes)),
        ),
        (
            "totalViews",
            serde_json::Value::Number(serde_json::Number::from(total_views)),
        ),
        ("tags", serde_json::json!(tags)),
        ("works", serde_json::json!(works)),
        ("actors", serde_json::json!(actors)),
    ])
}

fn hashmap_from_pairs<const N: usize>(
    pairs: [(&str, serde_json::Value); N],
) -> HashMap<String, serde_json::Value> {
    pairs
        .into_iter()
        .map(|(key, value)| (key.to_string(), value))
        .collect()
}

fn map_comic_detail(payload: ComicDetailPayload, img_host: Option<&str>) -> ComicDetail {
    let image = cover_image_url(img_host, &payload.id).unwrap_or_default();

    ComicDetail {
        id: payload.id,
        title: payload.name,
        author: payload.author,
        description: payload.description,
        total_views: payload.total_views,
        likes: payload.likes,
        comment_total: payload.comment_total,
        tags: payload.tags,
        actors: payload.actors,
        works: payload.works,
        is_favorite: payload.is_favorite,
        liked: payload.liked,
        related_list: payload
            .related_list
            .into_iter()
            .map(|item| map_related_comic(item, img_host))
            .collect(),
        series: payload
            .series
            .into_iter()
            .map(|item| ComicChapter {
                id: item.id,
                title: item.name,
                sort: item.sort,
            })
            .collect(),
        series_id: payload.series_id,
        price: payload.price,
        purchased: payload.purchased,
        image,
    }
}

fn map_related_comic(item: ComicDetailRelatedPayload, img_host: Option<&str>) -> RelatedComic {
    let image = cover_image_url(img_host, &item.id).unwrap_or(item.image);

    RelatedComic {
        id: item.id,
        title: item.name,
        author: item.author,
        image,
    }
}

fn map_comment(payload: CommentPayload, img_host: Option<&str>) -> ComicComment {
    ComicComment {
        id: payload.cid,
        comic_id: payload.aid,
        user_id: payload.uid,
        username: payload.username,
        nickname: payload.nickname,
        content: payload.content,
        like_count: payload.likes,
        time: payload.addtime,
        updated_at: payload.update_at,
        avatar: user_avatar_url(img_host, &payload.photo).unwrap_or_default(),
        parent_id: payload.parent_cid,
        spoiler: payload.spoiler,
        replies: payload
            .replys
            .unwrap_or_default()
            .into_iter()
            .map(|reply| map_comment(reply, img_host))
            .collect(),
    }
}

fn map_login_user(payload: LoginPayload, img_host: Option<&str>) -> UserProfile {
    let avatar_url = user_avatar_url(img_host, &payload.photo).unwrap_or_default();

    UserProfile {
        id: payload.uid,
        username: payload.username,
        email: payload.email,
        avatar: payload.photo,
        avatar_url,
        level: payload.level,
        level_name: payload.level_name,
        current_level_exp: payload.exp,
        next_level_exp: payload.next_level_exp,
        exp_percent: payload.exp_percent,
        current_collect_count: payload.album_favorites,
        max_collect_count: payload.album_favorites_max,
        j_coin: payload.coin,
    }
}

fn map_sign_in_records(records: Vec<Vec<SignInRecordPayload>>) -> Vec<SignInRecord> {
    records
        .into_iter()
        .flatten()
        .enumerate()
        .map(|(index, record)| SignInRecord {
            day: index as u32 + 1,
            date: record.date,
            signed: record.signed,
            bonus: record.bonus,
        })
        .collect()
}

fn map_week_categories(categories: Vec<WeekCategoryPayload>) -> Vec<WeekCategory> {
    categories
        .into_iter()
        .map(|item| WeekCategory {
            label: if item.time.is_empty() {
                item.title.clone()
            } else {
                format!("{} ({})", item.title, item.time)
            },
            id: item.id,
            time: item.time,
            title: item.title,
        })
        .collect()
}

fn map_week_types(types: Vec<WeekTypePayload>) -> Vec<WeekType> {
    types
        .into_iter()
        .map(|item| WeekType {
            id: item.id,
            title: item.title,
        })
        .collect()
}

fn decrypt_data(data: &str, ts: &str) -> Result<String, String> {
    let key = md5_hex(&format!("{ts}{API_SECRET}"));
    decrypt_base64_with_key(data, &key)
}

fn decrypt_base64_with_key(data: &str, key: &str) -> Result<String, String> {
    let encrypted = BASE64_STANDARD
        .decode(data)
        .map_err(|error| format!("Invalid encrypted data: {error}"))?;
    let decrypted = Aes256EcbDec::new_from_slice(key.as_bytes())
        .map_err(|error| format!("Invalid AES key: {error}"))?
        .decrypt_padded_vec_mut::<Pkcs7>(&encrypted)
        .map_err(|error| format!("Failed to decrypt response: {error}"))?;

    String::from_utf8(decrypted).map_err(|error| format!("Invalid decrypted text: {error}"))
}

pub(crate) fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn current_millis_timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

fn current_seconds_timestamp() -> String {
    current_timestamp().to_string()
}

fn android_user_agent() -> &'static str {
    "Mozilla/5.0 (Linux; Android 13; jm-boom Build/TQ1A.230305.002; wv) AppleWebKit/537.36 (KHTML, like Gecko) Version/4.0 Chrome/120.0.6099.230 Mobile Safari/537.36"
}

fn request_url_host(url: &str) -> Option<String> {
    reqwest::Url::parse(url)
        .ok()
        .and_then(|url| url.host_str().map(str::to_string))
        .filter(|host| !host.is_empty())
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

fn deserialize_u32_from_any<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;

    match value {
        serde_json::Value::Number(number) => number
            .as_u64()
            .and_then(|value| u32::try_from(value).ok())
            .ok_or_else(|| serde::de::Error::custom("expected a valid u32 number")),
        serde_json::Value::String(value) => {
            let value = value.trim();

            if value.is_empty() {
                return Ok(0);
            }

            value.parse::<u32>().map_err(|error| {
                serde::de::Error::custom(format!("expected a u32 string: {error}"))
            })
        }
        serde_json::Value::Null => Ok(0),
        _ => Err(serde::de::Error::custom("expected a u32-compatible value")),
    }
}

fn deserialize_f32_from_any<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;

    match value {
        serde_json::Value::Number(number) => number
            .as_f64()
            .map(|value| value as f32)
            .ok_or_else(|| serde::de::Error::custom("expected a valid f32 number")),
        serde_json::Value::String(value) => {
            let value = value.trim().trim_end_matches('%');

            if value.is_empty() {
                return Ok(0.0);
            }

            value.parse::<f32>().map_err(|error| {
                serde::de::Error::custom(format!("expected an f32 string: {error}"))
            })
        }
        serde_json::Value::Null => Ok(0.0),
        _ => Err(serde::de::Error::custom("expected a f32-compatible value")),
    }
}

fn deserialize_string_from_string_or_number<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;

    match value {
        serde_json::Value::String(value) => Ok(value),
        serde_json::Value::Number(value) => Ok(value.to_string()),
        _ => Err(serde::de::Error::custom("expected a string or number")),
    }
}

fn deserialize_string_from_any<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;

    match value {
        serde_json::Value::String(value) => Ok(value),
        serde_json::Value::Number(value) => Ok(value.to_string()),
        serde_json::Value::Bool(value) => Ok(value.to_string()),
        serde_json::Value::Null => Ok(String::new()),
        _ => Err(serde::de::Error::custom("expected a scalar value")),
    }
}

fn deserialize_optional_string_from_any<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<serde_json::Value>::deserialize(deserializer)?;
    let Some(value) = value else {
        return Ok(None);
    };

    match value {
        serde_json::Value::String(value) => {
            if value.trim().is_empty() {
                Ok(None)
            } else {
                Ok(Some(value))
            }
        }
        serde_json::Value::Number(value) => Ok(Some(value.to_string())),
        serde_json::Value::Bool(value) => Ok(Some(value.to_string())),
        serde_json::Value::Null => Ok(None),
        _ => Err(serde::de::Error::custom(
            "expected an optional scalar value",
        )),
    }
}

fn deserialize_string_vec_from_any<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;

    match value {
        serde_json::Value::Array(items) => {
            let values = items
                .into_iter()
                .filter_map(|item| match item {
                    serde_json::Value::String(value) => Some(value),
                    serde_json::Value::Number(value) => Some(value.to_string()),
                    serde_json::Value::Bool(value) => Some(value.to_string()),
                    _ => None,
                })
                .filter(|value| !value.trim().is_empty())
                .collect::<Vec<_>>();

            Ok(values)
        }
        serde_json::Value::String(value) => {
            if value.trim().is_empty() {
                Ok(Vec::new())
            } else {
                Ok(vec![value])
            }
        }
        serde_json::Value::Number(value) => Ok(vec![value.to_string()]),
        serde_json::Value::Null => Ok(Vec::new()),
        _ => Err(serde::de::Error::custom("expected a string list")),
    }
}

fn deserialize_bool_from_any<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;

    match value {
        serde_json::Value::Bool(value) => Ok(value),
        serde_json::Value::Number(value) => Ok(value.as_u64().unwrap_or_default() != 0),
        serde_json::Value::String(value) => {
            let value = value.trim().to_ascii_lowercase();

            Ok(matches!(value.as_str(), "1" | "true" | "yes" | "ok"))
        }
        serde_json::Value::Null => Ok(false),
        _ => Err(serde::de::Error::custom("expected a bool-compatible value")),
    }
}

fn deserialize_optional_i64_from_string_or_number<'de, D>(
    deserializer: D,
) -> Result<Option<i64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<serde_json::Value>::deserialize(deserializer)?;
    let Some(value) = value else {
        return Ok(None);
    };

    match value {
        serde_json::Value::Number(number) => number
            .as_i64()
            .map(Some)
            .ok_or_else(|| serde::de::Error::custom("expected a valid i64 number")),
        serde_json::Value::String(value) => {
            if value.trim().is_empty() {
                return Ok(None);
            }

            value.parse::<i64>().map(Some).map_err(|error| {
                serde::de::Error::custom(format!("expected an i64 string: {error}"))
            })
        }
        _ => Err(serde::de::Error::custom("expected an i64 number or string")),
    }
}

pub(crate) fn resolve_api_endpoint(endpoint: Option<String>) -> ApiResult<String> {
    let Some(endpoint) = endpoint else {
        return Ok(DEFAULT_API_ENDPOINT.to_string());
    };
    normalize_api_endpoint(&endpoint)
}

fn normalize_api_endpoint(endpoint: &str) -> ApiResult<String> {
    let endpoint = endpoint.trim().trim_end_matches('/');

    if endpoint.is_empty() {
        return Ok(DEFAULT_API_ENDPOINT.to_string());
    }

    let endpoint = if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
        endpoint.to_string()
    } else {
        format!("https://{endpoint}")
    };
    let url = reqwest::Url::parse(&endpoint).map_err(|error| {
        ApiError::new(
            ApiErrorKind::UnsupportedEndpoint,
            format!("Invalid API endpoint {endpoint}: {error}"),
        )
    })?;

    match url.scheme() {
        "http" | "https" if url.host_str().is_some() => {
            let mut normalized = format!("{}://{}", url.scheme(), url.host_str().unwrap());
            if let Some(port) = url.port() {
                normalized.push_str(&format!(":{port}"));
            }
            Ok(normalized)
        }
        _ => Err(ApiError::new(
            ApiErrorKind::UnsupportedEndpoint,
            format!("Unsupported API endpoint: {endpoint}"),
        )),
    }
}

fn cover_image_url(img_host: Option<&str>, comic_id: &str) -> Option<String> {
    let img_host = img_host?.trim().trim_end_matches('/');

    if img_host.is_empty() {
        return None;
    }

    Some(format!("{img_host}/media/albums/{comic_id}_3x4.jpg"))
}

fn user_avatar_url(img_host: Option<&str>, photo: &str) -> Option<String> {
    let photo = photo.trim();

    if photo.is_empty() {
        return None;
    }

    if photo.starts_with("http://") || photo.starts_with("https://") {
        return Some(photo.to_string());
    }

    let img_host = img_host?.trim().trim_end_matches('/');

    if img_host.is_empty() {
        return None;
    }

    if photo.starts_with('/') {
        Some(format!("{img_host}{photo}"))
    } else {
        Some(format!("{img_host}/media/users/{photo}"))
    }
}

fn response_preview(value: &str) -> String {
    value
        .chars()
        .take(180)
        .collect::<String>()
        .replace('\n', "\\n")
}
