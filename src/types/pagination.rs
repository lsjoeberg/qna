use std::collections::HashMap;

use handle_errors::QueryError;

/// Pagination struct that is getting extracted from query parameters
#[derive(Debug, Default)]
pub struct Pagination {
    /// The index of the first item that has to be returned
    pub limit: Option<i64>,
    /// The index of the last item that has to be returned
    pub offset: i64,
}

/// Extract query parameters from the `/questions` route
/// # Example query
/// GET requests to this route can have a pagination attached so we just
/// return the questions we need
/// `/questions?start=1&end=10`
/// # Example usage
/// ```rust
/// let mut query = HashMap::new();
/// query.insert("start".to_string(), "1".to_string());
/// query.insert("end".to_string(), "10".to_string());
/// let p = types::pagination::extract_pagination(query).unwrap();
/// assert_eq!(p.start, 1);
/// assert_eq!(p.end, 10);
/// ```
pub fn extract_pagination(params: HashMap<String, String>) -> Result<Pagination, QueryError> {
    if params.contains_key("limit") && params.contains_key("offset") {
        // Takes the "limit" parameter in the query and attempts to convert to a number.
        return Ok(Pagination {
            limit: Some(
                params
                    .get("limit")
                    .unwrap()
                    .parse::<i64>()
                    .map_err(QueryError::ParseError)?,
            ),
            offset: params
                .get("offset")
                .unwrap()
                .parse::<i64>()
                .map_err(QueryError::ParseError)?,
        });
    }
    Err(QueryError::MissingParameters)
}
