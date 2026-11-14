//! `mailto:` handling for the `.desktop` entry and single-instance arguments.
//!
//! The URL is parsed by hand rather than handed to the shell or the webview:
//! only `to`, `cc`, `bcc`, `subject` and `body` are accepted, and the result is
//! passed to the composer as plain strings.

use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct MailtoRequest {
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub subject: String,
    pub body: String,
}

pub fn parse(href: &str) -> Option<MailtoRequest> {
    let rest = href
        .strip_prefix("mailto:")
        .or_else(|| href.strip_prefix("MAILTO:"))?;
    // Tolerate the `//` form some clients emit.
    let rest = rest.strip_prefix("//").unwrap_or(rest);
    let (address, query) = rest.split_once('?').unwrap_or((rest, ""));

    let decode = |value: &str| -> String {
        percent_encoding::percent_decode_str(value)
            .decode_utf8_lossy()
            .replace('+', " ")
    };

    let split = |value: &str| -> Vec<String> {
        value
            .split(',')
            .map(decode)
            .map(|part| part.trim().to_string())
            .filter(|part| !part.is_empty())
            .collect()
    };

    let mut request = MailtoRequest {
        to: split(address),
        cc: Vec::new(),
        bcc: Vec::new(),
        subject: String::new(),
        body: String::new(),
    };

    for (key, value) in form_urlencoded::parse(query.as_bytes()) {
        match key.as_ref() {
            "cc" => request.cc = split(value.as_ref()),
            "bcc" => request.bcc = split(value.as_ref()),
            "subject" => request.subject = value.into_owned(),
            "body" => request.body = value.into_owned(),
            _ => {}
        }
    }

    Some(request)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_addresses_and_fields() {
        let request = parse(
            "mailto:ada@example.com,grace@example.com?cc=team@example.com&subject=Hello%20there&body=Line%201%0ALine%202",
        )
        .unwrap();
        assert_eq!(request.to, ["ada@example.com", "grace@example.com"]);
        assert_eq!(request.cc, ["team@example.com"]);
        assert_eq!(request.subject, "Hello there");
        assert!(request.body.contains("Line 1"));
    }

    #[test]
    fn tolerates_plus_and_empty_values() {
        let request = parse("mailto:?subject=a+b&cc=").unwrap();
        assert!(request.to.is_empty());
        assert_eq!(request.subject, "a b");
        assert!(request.cc.is_empty());
    }

    #[test]
    fn rejects_other_schemes() {
        assert!(parse("https://example.com").is_none());
    }
}
