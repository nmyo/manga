use serde::{Deserialize, Serialize};

use super::serde_ext::*;

#[derive(Debug, Deserialize)]
pub(crate) struct ApiResponse<T> {
    pub(crate) code: i32,
    pub(crate) data: Option<T>,
    #[serde(rename = "errorMsg")]
    pub(crate) error_msg: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct SearchCategory {
    #[serde(default, deserialize_with = "string_or_number_or_empty")]
    pub(crate) id: String,
    pub(crate) title: Option<String>,
}
