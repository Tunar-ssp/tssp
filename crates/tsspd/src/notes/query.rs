//! List-query parsing for note endpoints.

use serde::Deserialize;
use tssp_domain::UnixTimestamp;
use tssp_ports::{NoteListQuery, NoteListSort};

const DEFAULT_LIST_LIMIT: u64 = 50;

/// Query parameters for `GET /api/v1/notes`.
#[derive(Debug, Deserialize)]
pub(crate) struct ListNotesQuery {
    #[serde(default = "default_list_limit")]
    pub(crate) limit: u64,
    pub(crate) tag: Option<String>,
    pub(crate) since: Option<i64>,
    pub(crate) until: Option<i64>,
    pub(crate) title: Option<String>,
    #[serde(default)]
    pub(crate) pinned: bool,
    pub(crate) sort: Option<String>,
}

const fn default_list_limit() -> u64 {
    DEFAULT_LIST_LIMIT
}

/// Builds a repository list query from HTTP parameters.
///
/// # Errors
///
/// Returns a client-safe message when parameters are invalid.
pub(crate) fn build_list_query(params: &ListNotesQuery) -> Result<NoteListQuery, String> {
    if params.limit == 0 || params.limit > 500 {
        return Err("limit must be between 1 and 500".to_owned());
    }
    let mut tags = Vec::new();
    if let Some(tag) = &params.tag {
        tags.push(tssp_domain::TagKey::new(tag).map_err(|error| error.to_string())?);
    }
    let since = params
        .since
        .map(UnixTimestamp::new)
        .transpose()
        .map_err(|error| error.to_string())?;
    let until = params
        .until
        .map(UnixTimestamp::new)
        .transpose()
        .map_err(|error| error.to_string())?;
    let sort = params
        .sort
        .as_deref()
        .and_then(NoteListSort::parse)
        .unwrap_or_default();

    Ok(NoteListQuery {
        limit: params.limit,
        tags,
        since,
        until,
        title_substring: params.title.clone(),
        pinned_only: params.pinned,
        sort,
        after_cursor: None,
        owner_id: None,
    })
}

#[cfg(test)]
mod tests {
    use super::{build_list_query, ListNotesQuery};

    #[test]
    fn build_list_query_rejects_out_of_range_limit() {
        let params = ListNotesQuery {
            limit: 0,
            tag: None,
            since: None,
            until: None,
            title: None,
            pinned: false,
            sort: None,
        };
        assert!(build_list_query(&params).is_err());
    }

    #[test]
    fn build_list_query_accepts_sort_alias() {
        let params = ListNotesQuery {
            limit: 10,
            tag: None,
            since: None,
            until: None,
            title: None,
            pinned: false,
            sort: Some("-updated".to_owned()),
        };
        let query = build_list_query(&params).unwrap_or_else(|error| panic!("{error}"));
        assert_eq!(query.limit, 10);
    }
}
