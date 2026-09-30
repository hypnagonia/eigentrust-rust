use std::collections::HashMap;

pub fn init_logger() {
    #[cfg(target_arch = "wasm32")]
    {
        // the demo recomputes on every slider move, keep the browser console quiet
        console_log::init_with_level(log::Level::Warn).expect("Failed to initialize logger");
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        // RUST_LOG overrides, e.g. RUST_LOG=trace
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
            .format_timestamp_millis()
            .init();
        log::debug!("Logger initialized for native environment");
    }
}

pub struct PeersMap {
    pub map: HashMap<String, usize>,
    // index -> peer name
    pub names: Vec<String>,
}

impl PeersMap {
    pub fn new() -> Self {
        PeersMap {
            map: HashMap::new(),
            names: Vec::new(),
        }
    }

    pub fn insert_or_get(&mut self, key: &str) -> usize {
        if let Some(&existing_value) = self.map.get(key) {
            return existing_value;
        }

        let index = self.names.len();
        self.map.insert(key.to_string(), index);
        self.names.push(key.to_string());
        index
    }

    pub fn get_max_value(&self) -> usize {
        self.names.len()
    }
}

// Cleans a raw CSV field: surrounding whitespace and quotes.
pub fn clean_field(field: &str) -> &str {
    let field = field.trim();
    field
        .strip_prefix('"')
        .and_then(|f| f.strip_suffix('"'))
        .unwrap_or(field)
        .trim()
}

const HEADER_NAMES: &[&str] = &[
    "i", "j", "v", "from", "to", "value", "weight", "trust", "level", "peer", "id", "score",
    "source", "target", "src", "dst", "truster", "trustee",
];

// Strips a leading UTF-8 BOM and a header line, if present.
// `value_column` is the index of the numeric column (2 for local trust, 1 for pre-trust).
// A first line is a header when its value column is not a number, or, when the value column
// is omitted (implicit weight 1), when every field is a well-known header name.
pub fn strip_headers(csv_content: &str, value_column: usize) -> &str {
    let csv_content = csv_content.trim_start_matches('\u{feff}');
    let (first_line, rest) = match csv_content.find('\n') {
        Some(i) => (&csv_content[..i], &csv_content[i + 1..]),
        None => (csv_content, ""),
    };

    let fields: Vec<&str> = first_line.split(',').map(clean_field).collect();
    let is_header = match fields.get(value_column) {
        Some(value) => value.parse::<f64>().is_err(),
        None => fields
            .iter()
            .all(|f| HEADER_NAMES.contains(&f.to_ascii_lowercase().as_str())),
    };

    if is_header {
        rest
    } else {
        csv_content
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_headers() {
        assert_eq!(strip_headers("i,j,v\na,b,1\n", 2), "a,b,1\n");
        assert_eq!(strip_headers("i,j,v\r\na,b,1\r\n", 2), "a,b,1\r\n");
        assert_eq!(strip_headers("\u{feff}from,to,weight\na,b,1", 2), "a,b,1");
        // headerless two-column local trust keeps its first edge
        assert_eq!(strip_headers("a,b\nb,c", 2), "a,b\nb,c");
        assert_eq!(strip_headers("i,j\na,b", 2), "a,b");
        assert_eq!(strip_headers("a,b,1", 2), "a,b,1");
        assert_eq!(strip_headers("i,v\nalice,1", 1), "alice,1");
        assert_eq!(strip_headers("alice,1\nbob,1", 1), "alice,1\nbob,1");
        assert_eq!(strip_headers("alice\nbob", 1), "alice\nbob");
        assert_eq!(strip_headers("peer\nalice", 1), "alice");
        assert_eq!(strip_headers("", 1), "");
    }

    #[test]
    fn test_clean_field() {
        assert_eq!(clean_field("  alice "), "alice");
        assert_eq!(clean_field("\"alice\""), "alice");
        assert_eq!(clean_field(" \" 0.5 \" "), "0.5");
        assert_eq!(clean_field("\""), "\"");
    }
}
