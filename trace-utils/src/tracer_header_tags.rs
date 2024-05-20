// Copyright 2024-Present Datadog, Inc. https://www.datadoghq.com/
// SPDX-License-Identifier: Apache-2.0

use hyper::http::HeaderValue;
use hyper::HeaderMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

macro_rules! parse_string_header {
    (
        $header_map:ident,
        { $($header_key:literal => $($field:ident).+ ,)+ }
    ) => {
        $(
            if let Some(header_value) = $header_map.get($header_key) {
                if let Ok(h) = header_value.to_str() {
                    $($field).+ = h;
                }
            }
        )+
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct TracerHeaderTags<'a> {
    pub lang: &'a str,
    pub lang_version: &'a str,
    pub lang_interpreter: &'a str,
    pub lang_vendor: &'a str,
    pub tracer_version: &'a str,
    pub container_id: &'a str,
    // specifies that the client has marked top-level spans, when set. Any non-empty value will
    // mean 'yes'.
    pub client_computed_top_level: bool,
    // specifies whether the client has computed stats so that the agent doesn't have to. Any
    // non-empty value will mean 'yes'.
    pub client_computed_stats: bool,
}

impl<'a> From<TracerHeaderTags<'a>> for HashMap<&'static str, String> {
    fn from(tags: TracerHeaderTags<'a>) -> HashMap<&'static str, String> {
        let mut headers = HashMap::from([
            ("datadog-meta-lang", tags.lang.to_string()),
            ("datadog-meta-lang-version", tags.lang_version.to_string()),
            (
                "datadog-meta-lang-interpreter",
                tags.lang_interpreter.to_string(),
            ),
            ("datadog-meta-lang-vendor", tags.lang_vendor.to_string()),
            (
                "datadog-meta-tracer-version",
                tags.tracer_version.to_string(),
            ),
            ("datadog-container-id", tags.container_id.to_string()),
        ]);
        headers.retain(|_, v| !v.is_empty());
        headers
    }
}

impl<'a> From<&'a HeaderMap<HeaderValue>> for TracerHeaderTags<'a> {
    fn from(headers: &'a HeaderMap<HeaderValue>) -> Self {
        let mut tags = TracerHeaderTags::default();
        parse_string_header!(
            headers,
            {
                "datadog-meta-lang" => tags.lang,
                "datadog-meta-lang-version" => tags.lang_version,
                "datadog-meta-lang-interpreter" => tags.lang_interpreter,
                "datadog-meta-lang-vendor" => tags.lang_vendor,
                "datadog-meta-tracer-version" => tags.tracer_version,
                "datadog-container-id" => tags.container_id,
            }
        );
        if headers.get("datadog-client-computed-top-level").is_some() {
            tags.client_computed_top_level = true;
        }
        if headers.get("datadog-client-computed-stats").is_some() {
            tags.client_computed_stats = true;
        }
        tags
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper::HeaderMap;

    #[test]
    fn tags_to_hashmap() {
        let header_tags = TracerHeaderTags {
            lang: "test-lang",
            lang_version: "2.0",
            lang_interpreter: "interpreter",
            lang_vendor: "vendor",
            tracer_version: "1.0",
            container_id: "id",
            client_computed_top_level: false,
            client_computed_stats: false,
        };

        let map: HashMap<&'static str, String> = header_tags.into();

        assert_eq!(map.len(), 6);
        assert_eq!(map.get("datadog-meta-lang").unwrap(), "test-lang");
        assert_eq!(map.get("datadog-meta-lang-version").unwrap(), "2.0");
        assert_eq!(map.get("datadog-meta-lang-interpreter").unwrap(), "interpreter");
        assert_eq!(map.get("datadog-meta-lang-vendor").unwrap(), "vendor");
        assert_eq!(map.get("datadog-meta-tracer-version").unwrap(), "1.0");
        assert_eq!(map.get("datadog-container-id").unwrap(), "id");
    }
    #[test]
    fn tags_to_hashmap_empty_value() {
        let header_tags = TracerHeaderTags {
            lang: "test-lang",
            lang_version: "2.0",
            lang_interpreter: "interpreter",
            lang_vendor: "vendor",
            tracer_version: "1.0",
            container_id: "",
            client_computed_top_level: false,
            client_computed_stats: false,
        };

        let map: HashMap<&'static str, String> = header_tags.into();

        assert_eq!(map.len(), 5);
        assert_eq!(map.get("datadog-meta-lang").unwrap(), "test-lang");
        assert_eq!(map.get("datadog-meta-lang-version").unwrap(), "2.0");
        assert_eq!(map.get("datadog-meta-lang-interpreter").unwrap(), "interpreter");
        assert_eq!(map.get("datadog-meta-lang-vendor").unwrap(), "vendor");
        assert_eq!(map.get("datadog-meta-tracer-version").unwrap(), "1.0");
        assert_eq!(map.get("datadog-container-id"), None);
    }

    #[test]
    fn header_map_to_tags() {
        let mut header_map = HeaderMap::new();

        header_map.insert("datadog-meta-lang", "test-lang".parse().unwrap());
        header_map.insert("datadog-meta-lang-version", "2.0".parse().unwrap());
        header_map.insert("datadog-meta-lang-interpreter", "interpreter".parse().unwrap());
        header_map.insert("datadog-meta-lang-vendor", "vendor".parse().unwrap());
        header_map.insert("datadog-meta-tracer-version", "1.0".parse().unwrap());
        header_map.insert("datadog-container-id", "id".parse().unwrap());
        header_map.insert("datadog-client-computed-stats", "true".parse().unwrap());

        let tags: TracerHeaderTags = (&header_map).into();

        assert_eq!(tags.lang, "test-lang");
        assert_eq!(tags.lang_vendor, "vendor");
        assert_eq!(tags.lang_version, "2.0");
        assert_eq!(tags.tracer_version, "1.0");
        assert_eq!(tags.lang_interpreter, "interpreter");
        assert_eq!(tags.container_id, "id");
        assert!(tags.client_computed_stats);
        assert!(!tags.client_computed_top_level);
    }
}
