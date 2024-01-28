use std::collections::HashMap;

use handle_errors::Error;

/// Pagination struct that is getting extracted from query parameters
#[derive(Debug, Default, PartialEq)]
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
pub fn extract_pagination(params: HashMap<String, String>) -> Result<Pagination, Error> {
    if params.contains_key("limit") && params.contains_key("offset") {
        // Takes the "limit" parameter in the query and attempts to convert to a number.
        return Ok(Pagination {
            limit: Some(
                params
                    .get("limit")
                    .unwrap()
                    .parse::<i64>()
                    .map_err(Error::ParseError)?,
            ),
            offset: params
                .get("offset")
                .unwrap()
                .parse::<i64>()
                .map_err(Error::ParseError)?,
        });
    }
    Err(Error::MissingParameters)
}

#[cfg(test)]
mod pagination_tests {
    use super::{extract_pagination, Error, HashMap, Pagination};

    #[test]
    fn valid_pagination() {
        let mut params = HashMap::new();
        params.insert("limit".into(), "1".into());
        params.insert("offset".into(), "1".into());

        let pagination_result = extract_pagination(params);
        let expected = Pagination {
            limit: Some(1),
            offset: 1,
        };

        assert_eq!(pagination_result.unwrap(), expected);
    }

    #[test]
    fn missing_offset_parameter() {
        let mut params = HashMap::new();
        params.insert("limit".into(), "1".into());

        let pagination_result = format!("{}", extract_pagination(params).unwrap_err());
        let expected = format!("{}", Error::MissingParameters);

        assert_eq!(pagination_result, expected);
    }

    #[test]
    fn missing_limit_parameter() {
        let mut params = HashMap::new();
        params.insert("offset".into(), "1".into());

        let pagination_result = format!("{}", extract_pagination(params).unwrap_err());
        let expected = format!("{}", Error::MissingParameters);

        assert_eq!(pagination_result, expected);
    }

    #[test]
    fn wrong_offset_type() {
        let mut params = HashMap::new();
        params.insert("limit".into(), "1".into());
        params.insert("offset".into(), "NOT_A_NUMBER".into());
        let pagination_result = format!("{}", extract_pagination(params).unwrap_err());

        let expected = String::from("Cannot parse parameter: invalid digit found in string");

        assert_eq!(pagination_result, expected);
    }

    #[test]
    fn wrong_limit_type() {
        let mut params = HashMap::new();
        params.insert("limit".into(), "NOT_A_NUMBER".into());
        params.insert("offset".into(), "1".into());
        let pagination_result = format!("{}", extract_pagination(params).unwrap_err());

        let expected = String::from("Cannot parse parameter: invalid digit found in string");

        assert_eq!(pagination_result, expected);
    }
}
