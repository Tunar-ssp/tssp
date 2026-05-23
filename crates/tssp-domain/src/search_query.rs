//! Search tokenization and FTS5 query building.

/// Builds an FTS5 `MATCH` expression from user input.
///
/// Each token becomes a prefix term (`token*`) joined with `OR` so partial
/// queries match. Special FTS characters are stripped before quoting.
#[must_use]
pub fn build_fts_query(input: &str) -> String {
    let tokens = search_terms(input);
    if tokens.is_empty() {
        return String::new();
    }
    tokens
        .into_iter()
        .map(|token| format!("\"{token}\"*"))
        .collect::<Vec<_>>()
        .join(" OR ")
}

/// Returns normalized search terms used by ranking and bounded fuzzy fallback.
#[must_use]
pub fn search_terms(input: &str) -> Vec<String> {
    let mut out = Vec::new();
    for raw in input.split_whitespace() {
        let cleaned = sanitize_token(raw);
        if !cleaned.is_empty() {
            out.push(cleaned);
        }
    }
    out
}

fn sanitize_token(token: &str) -> String {
    token
        .chars()
        .filter(|ch| ch.is_alphanumeric() || *ch == '_' || *ch == '-')
        .collect::<String>()
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::{build_fts_query, search_terms};

    #[test]
    fn prefix_per_token() {
        let q = build_fts_query("repo rt");
        assert!(q.contains("\"repo\"*"));
        assert!(q.contains("\"rt\"*"));
        assert!(q.contains(" OR "));
    }

    #[test]
    fn strips_fts_operators() {
        let q = build_fts_query("foo\" OR bar");
        assert!(!q.contains(" OR  OR "));
    }

    #[test]
    fn empty_input_returns_empty() {
        assert!(build_fts_query("  ").is_empty());
    }

    #[test]
    fn search_terms_are_lowercase_and_operator_safe() {
        assert_eq!(search_terms("Annual \"Report\""), vec!["annual", "report"]);
    }
}
