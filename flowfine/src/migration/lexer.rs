use crate::migration::MigrationParsingError;
use crate::migration::MigrationParsingError::NoSemicolonsFoundError;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref QUOTE_REGEX: Regex = Regex::new(r"('[^']*')|:\w+").unwrap();
}

pub fn delimit_queries(
    filename: &str,
    content: &str,
) -> Result<Vec<String>, MigrationParsingError> {
    let semicolons = find_valid_semicolons(content);

    match semicolons.len() {
        0 => Err(NoSemicolonsFoundError(filename.to_string())),
        _ => {
            let mut queries = Vec::new();
            let mut start = 0;

            for &end in &semicolons {
                let query = &content[start..=end];
                queries.push(query.trim().to_string());
                start = end + 1;
            }

            Ok(queries)
        }
    }
}

// todo: add support for comments
fn find_valid_semicolons(content: &str) -> Vec<usize> {
    let mut quotes_captures = QUOTE_REGEX.find_iter(content).peekable();
    let mut query_separators = Vec::new();

    let mut in_quote = false;
    let mut quote_end = 0;

    for (i, c) in content.chars().enumerate() {
        if let Some(capture) = quotes_captures.peek() {
            if i == capture.start() {
                in_quote = true;
                quote_end = capture.end();
                quotes_captures.next();
            }
        }

        if in_quote && i == quote_end {
            in_quote = false;
        }

        if !in_quote && c == ';' {
            query_separators.push(i);
        }
    }

    query_separators
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest(query, expected_result,
    case("SELECT * FROM FOO;INSERT INTO FOO(id, text) VALUES (1, 'foo;'), (2, ';;;');DROP TABLE;", vec![
    "SELECT * FROM FOO;".to_string(),
    "INSERT INTO FOO(id, text) VALUES (1, 'foo;'), (2, ';;;');".to_string(),
    "DROP TABLE;".to_string(),
    ]),
    case("SELECT * FROM FOO;INSERT INTO FOO(id, text) VALUES (1, 'foo;'), (2, ';;;');DROP TABLE;", vec![
    "SELECT * FROM FOO;".to_string(),
    "INSERT INTO FOO(id, text) VALUES (1, 'foo;'), (2, ';;;');".to_string(),
    "DROP TABLE;".to_string(),
    ]),
    case("  SELECT * FROM FOO;  INSERT INTO FOO(id, text) VALUES (1, 'foo;'), (2, ';;;');  DROP TABLE;    ", vec![
    "SELECT * FROM FOO;".to_string(),
    "INSERT INTO FOO(id, text) VALUES (1, 'foo;'), (2, ';;;');".to_string(),
    "DROP TABLE;".to_string(),
    ]),
    case(" \n SELECT * FROM FOO; \n INSERT INTO FOO(id, text) VALUES (1, 'foo;'), (2, ';;;'); \n DROP TABLE;  \n  ", vec![
    "SELECT * FROM FOO;".to_string(),
    "INSERT INTO FOO(id, text) VALUES (1, 'foo;'), (2, ';;;');".to_string(),
    "DROP TABLE;".to_string(),
    ]),
    )]
    fn test_delimited_queries(query: &str, expected_result: Vec<String>) {
        assert_eq!(delimit_queries("", query), Ok(expected_result));
    }

    #[rstest(query, expected_error,
        case("", NoSemicolonsFoundError("".to_string())),
        case("SELECT * FROM FOO", NoSemicolonsFoundError("".to_string())),
    )]
    fn test_failed_delimited_queries(query: &str, expected_error: MigrationParsingError) {
        assert_eq!(delimit_queries("", query), Err(expected_error));
    }
}
