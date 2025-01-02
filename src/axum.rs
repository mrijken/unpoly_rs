use crate::headers;
use crate::LayerMode;
use crate::Unpoly;

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};

#[async_trait]
impl<S> FromRequestParts<S> for Unpoly
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let request_version = parts
            .headers
            .get(headers::VERSION)
            .map(|v| v.to_str().map_or(None, |v| Some(v.to_string())))
            .unwrap_or(None);

        let request_context: Option<serde_json::Value> = parts
            .headers
            .get(headers::CONTEXT)
            .map(|v| v.to_str().ok())
            .unwrap_or(None)
            .map(|v| serde_json::from_str(v).unwrap_or_default());

        let request_fail_context: Option<serde_json::Value> = parts
            .headers
            .get(headers::FAIL_CONTEXT)
            .map(|v| v.to_str().ok())
            .unwrap_or(None)
            .map(|v| serde_json::from_str(v).unwrap_or_default());

        let request_mode = parts
            .headers
            .get(headers::MODE)
            .map(|v| {
                v.to_str().map_or(LayerMode::ROOT, |v| {
                    serde_json::from_str(&("\"".to_string() + v + "\"")).unwrap_or_default()
                })
            })
            .unwrap_or(LayerMode::ROOT);

        let request_fail_mode = parts
            .headers
            .get(headers::FAIL_MODE)
            .map(|v| {
                v.to_str().map_or(LayerMode::ROOT, |v| {
                    serde_json::from_str(&("\"".to_string() + v + "\"")).unwrap_or_default()
                })
            })
            .unwrap_or(LayerMode::ROOT);

        let request_target = parts
            .headers
            .get(headers::TARGET)
            .map(|v| v.to_str().map_or(None, |v| Some(v.to_string())))
            .unwrap_or(None);

        let request_fail_target = parts
            .headers
            .get(headers::FAIL_TARGET)
            .map(|v| v.to_str().map_or(None, |v| Some(v.to_string())))
            .unwrap_or(None);

        let request_validate = parts.headers.get(headers::VALIDATE).map_or(vec![], |v| {
            v.to_str()
                .unwrap_or("")
                .split_whitespace()
                .map(|v| v.trim().to_string())
                .collect()
        });

        Ok(Unpoly {
            request_version,
            request_context,
            request_fail_context,
            request_fail_mode,
            request_mode,
            request_target,
            request_fail_target,
            request_validate,
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    extern crate axum;
    use axum::{body::Body, http::Request};

    use super::*;

    #[tokio::test]
    async fn test_no_unpoly_request() {
        let request = Request::builder()
            .method("GET")
            .uri("https://www.unpoly.com/")
            .header("X-Custom-Foo", "Bar")
            .body(Body::empty())
            .unwrap();
        let mut parts = request.into_parts();

        let mut unpoly = Unpoly::from_request_parts(&mut parts.0, &()).await.unwrap();

        assert_eq!(unpoly.request_version, None);
        assert_eq!(unpoly.request_context, None);
        assert_eq!(unpoly.request_fail_context, None);
        assert_eq!(unpoly.request_fail_mode, LayerMode::ROOT);
        assert_eq!(unpoly.request_mode, LayerMode::ROOT);
        assert_eq!(unpoly.request_target, None);

        assert!(!unpoly.is_up());

        assert!(!unpoly.get_headers().unwrap().contains_key("Vary"));
    }

    #[tokio::test]
    async fn test_unpoly_default() {
        let request = Request::builder()
            .method("GET")
            .uri("https://www.unpoly.com/")
            .header("X-Up-Version", "1.0.0")
            .body(Body::empty())
            .unwrap();
        let mut parts = request.into_parts();

        let mut unpoly = Unpoly::from_request_parts(&mut parts.0, &()).await.unwrap();

        assert_eq!(unpoly.request_version, Some("1.0.0".to_string()));
        assert_eq!(unpoly.request_context, None);
        assert_eq!(unpoly.request_fail_context, None);
        assert_eq!(unpoly.request_fail_mode, LayerMode::ROOT);
        assert_eq!(unpoly.request_mode, LayerMode::ROOT);
        assert_eq!(unpoly.request_target, None);

        unpoly.is_up();
        unpoly.set_success(true);
        unpoly.mode();

        assert_eq!(
            unpoly.get_headers().unwrap()["Vary"],
            "X-Up-Mode,X-Up-Target,X-Up-Version".to_string()
        );

        let mut unpoly = Unpoly::from_request_parts(&mut parts.0, &()).await.unwrap();
        unpoly.is_up();
        unpoly.set_success(false);
        unpoly.mode();

        assert_eq!(
            unpoly.get_headers().unwrap()["Vary"],
            "X-Up-Fail-Mode,X-Up-Fail-Target,X-Up-Version".to_string()
        );
    }

    #[tokio::test]
    async fn test_unpoly_success() {
        let request = Request::builder()
            .method("GET")
            .uri("https://www.unpoly.com/")
            .header("X-Up-Version", "1.0.0")
            .header("X-Up-Context", "{\"lives\": 42}")
            .header("X-Up-Fail-Context", "{\"lives\": 2}")
            .header("X-Up-Target", "main")
            .header("X-Up-Fail-Target", "root")
            .header("X-Up-Mode", "root")
            .header("X-Up-Fail-Mode", "cover")
            .header("X-Up-Validate", "name")
            .body(Body::empty())
            .unwrap();
        let mut parts = request.into_parts();

        let mut unpoly = Unpoly::from_request_parts(&mut parts.0, &()).await.unwrap();
        unpoly.set_success(true);

        unpoly.is_up();
        assert_eq!(unpoly.context(), Some(&serde_json::json!({"lives": 42})));
        assert_eq!(unpoly.target(), Some("main"));
        assert_eq!(*unpoly.mode(), LayerMode::ROOT);

        assert_eq!(
            unpoly.get_headers().unwrap()["Vary"],
            "X-Up-Context,X-Up-Mode,X-Up-Target,X-Up-Version".to_string()
        );
    }

    #[tokio::test]
    async fn test_unpoly_fail() {
        let request = Request::builder()
            .method("GET")
            .uri("https://www.unpoly.com/")
            .header("X-Up-Version", "1.0.0")
            .header("X-Up-Context", "{\"lives\": 42}")
            .header("X-Up-Fail-Context", "{\"lives\": 2}")
            .header("X-Up-Target", "main")
            .header("X-Up-Fail-Target", "root")
            .header("X-Up-Mode", "root")
            .header("X-Up-Fail-Mode", "cover")
            .header("X-Up-Validate", "name")
            .body(Body::empty())
            .unwrap();
        let mut parts = request.into_parts();

        let mut unpoly = Unpoly::from_request_parts(&mut parts.0, &()).await.unwrap();
        unpoly.set_success(false);

        unpoly.is_up();
        assert_eq!(unpoly.context(), Some(&serde_json::json!({"lives": 2})));
        assert_eq!(unpoly.target(), Some("root"));
        assert_eq!(*unpoly.mode(), LayerMode::COVER);

        assert_eq!(
            unpoly.get_headers().unwrap()["Vary"],
            "X-Up-Fail-Context,X-Up-Fail-Mode,X-Up-Fail-Target,X-Up-Version".to_string()
        );
    }

    #[tokio::test]
    async fn test_unpoly_set_responses() {
        let request = Request::builder()
            .method("GET")
            .uri("https://www.unpoly.com/")
            .header("X-Up-Version", "1.0.0")
            .header("X-Up-Context", "{\"lives\": 42}")
            .header("X-Up-Fail-Context", "{\"lives\": 2}")
            .header("X-Up-Target", "main")
            .header("X-Up-Fail-Target", "root")
            .header("X-Up-Mode", "root")
            .header("X-Up-Fail-Mode", "cover")
            .header("X-Up-Validate", "name")
            .body(Body::empty())
            .unwrap();
        let mut parts = request.into_parts();

        let mut unpoly = Unpoly::from_request_parts(&mut parts.0, &()).await.unwrap();

        unpoly.set_context(serde_json::json!({"lives": 43}));

        unpoly.set_title("Hello");
        unpoly.set_location("https://unpoly.com/");
        unpoly.set_method("PUT");
        unpoly.set_target("main");
        unpoly.set_evict_cache("main".to_string());
        unpoly.set_expire_cache("main".to_string());

        assert_eq!(unpoly.get_headers().unwrap()["X-Up-Title"], "Hello");
        assert_eq!(
            unpoly.get_headers().unwrap()["X-Up-Location"],
            "https://unpoly.com/"
        );
        assert_eq!(unpoly.get_headers().unwrap()["X-Up-Method"], "PUT");
        assert_eq!(unpoly.get_headers().unwrap()["X-Up-Evict-Cache"], "main");
        assert_eq!(unpoly.get_headers().unwrap()["X-Up-Expire-Cache"], "main");
    }
}
