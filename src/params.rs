use std::collections::HashMap;

use url::form_urlencoded;

#[derive(Debug, Default)]
pub struct ParsedParams {
    values: HashMap<String, Vec<String>>,
}

impl ParsedParams {
    pub fn from_raw(raw_query: Option<&str>) -> Self {
        let mut values: HashMap<String, Vec<String>> = HashMap::new();

        if let Some(raw) = raw_query {
            for (key, value) in form_urlencoded::parse(raw.as_bytes()) {
                values
                    .entry(key.into_owned())
                    .or_default()
                    .push(value.into_owned());
            }
        }

        Self { values }
    }

    pub fn get_string_value(&self, key: &str, default_value: &str) -> String {
        self.values
            .get(key)
            .and_then(|list| list.first())
            .cloned()
            .unwrap_or_else(|| default_value.to_string())
    }

    pub fn get_optional_string(&self, key: &str) -> Option<String> {
        self.values.get(key).and_then(|list| list.first()).cloned()
    }

    pub fn get_number_value(&self, key: &str, default_value: i32) -> i32 {
        self.values
            .get(key)
            .and_then(|list| list.first())
            .and_then(|value| value.parse::<i32>().ok())
            .unwrap_or(default_value)
    }

    pub fn get_boolean_value(&self, key: &str, default_value: bool) -> bool {
        self.values
            .get(key)
            .and_then(|list| list.first())
            .map(|value| value == "true")
            .unwrap_or(default_value)
    }

    pub fn get_all(&self, key: &str) -> Vec<String> {
        self.values.get(key).cloned().unwrap_or_default()
    }

    pub fn get_all_csv(&self, key: &str) -> Vec<String> {
        self.get_all(key)
            .into_iter()
            .flat_map(|item| {
                item.split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::ParsedParams;

    #[test]
    fn parse_multi_value_csv() {
        let params = ParsedParams::from_raw(Some("title=Stars,Followers&title=-Issues"));
        let titles = params.get_all_csv("title");
        assert_eq!(titles, vec!["Stars", "Followers", "-Issues"]);
    }
}
