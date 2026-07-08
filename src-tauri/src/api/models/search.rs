use serde::Deserialize;

use super::serde_ext::*;
use super::common::SearchCategory;

#[derive(Debug, Deserialize)]
pub(crate) struct SearchPayload {
    #[serde(deserialize_with = "u32_from_string_or_number")]
    pub(crate) total: u32,
    #[serde(default)]
    pub(crate) redirect_aid: Option<String>,
    #[serde(default)]
    pub(crate) content: Vec<SearchComicPayload>,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct SearchComicPayload {
    #[serde(default, deserialize_with = "string_or_number_or_empty")]
    pub(crate) id: String,
    #[serde(default, deserialize_with = "lossy_string_from_scalar")]
    pub(crate) author: String,
    #[serde(default, deserialize_with = "optional_lossy_string_from_scalar")]
    pub(crate) description: Option<String>,
    #[serde(default, deserialize_with = "lossy_string_from_scalar")]
    pub(crate) name: String,
    #[serde(default, deserialize_with = "lossy_string_from_scalar")]
    pub(crate) image: String,
    #[serde(default)]
    pub(crate) category: Option<SearchCategory>,
    #[serde(default)]
    pub(crate) category_sub: Option<SearchCategory>,
    #[serde(default, deserialize_with = "optional_i64_from_string_or_number")]
    pub(crate) update_at: Option<i64>,
    #[serde(default, deserialize_with = "u32_from_string_or_number_or_empty")]
    pub(crate) likes: u32,
    #[serde(
        default,
        alias = "totalViews",
        deserialize_with = "u32_from_string_or_number_or_empty"
    )]
    pub(crate) total_views: u32,
    #[serde(default, deserialize_with = "lossy_string_vec_from_array_or_scalar")]
    pub(crate) tags: Vec<String>,
    #[serde(default, deserialize_with = "lossy_string_vec_from_array_or_scalar")]
    pub(crate) works: Vec<String>,
    #[serde(default, deserialize_with = "lossy_string_vec_from_array_or_scalar")]
    pub(crate) actors: Vec<String>,
    #[serde(default, deserialize_with = "bool_or_int_string_or_empty")]
    pub(crate) liked: bool,
    #[serde(default, deserialize_with = "bool_or_int_string_or_empty")]
    pub(crate) is_favorite: bool,
}
