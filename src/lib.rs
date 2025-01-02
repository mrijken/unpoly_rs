#[cfg(feature = "axum")]
mod axum;
mod headers;
use std::collections::HashSet;

use derive_more::{Display, From};
use http::HeaderMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, From, Display)]
pub enum Error {
    #[from]
    InvalidJson(serde_json::Error),
    EventIsNotSerializableAsObject,
}

/// The mode of a layer
///
/// See <https://unpoly.com/layer-terminology>
#[derive(Debug, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LayerMode {
    #[default]
    /// The initial page
    ROOT,
    /// A modal dialog box
    MODAL,
    /// A drawer sliding in from the side
    DRAWER,
    /// A popup menu anchored to a link
    POPUP,
    ///An overlay covering the entire screen
    COVER,
}

impl LayerMode {
    /// Returns true if the layer is the root layer.
    pub fn is_root(&self) -> bool {
        self == &LayerMode::ROOT
    }

    /// Returns true if the layer is an overlay (ie is not the root layer).
    pub fn is_overlay(&self) -> bool {
        self != &LayerMode::ROOT
    }
}

/// Method to match a layer relative to the current layer
///
/// See <https://unpoly.com/layer-option#matching-relative-to-the-current-layer>
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MatchingLayer {
    /// The current layer
    CURRENT,
    /// The layer that opened the current layer
    PARENT,
    /// The current layer or any ancestor, preferring closer layers
    CLOSEST,
    /// Any overlay
    OVERLAY,
    /// Any ancestor layer of the current layer
    ANCESTOR,
    /// The child layer of the current layer
    CHILD,
    /// Any descendant of the current layer
    DESCENDANT,
    /// The current layer and its descendants
    SUBTREE,
    /// The layer at the given index, where 0 is the root layer
    INDEX(u32),
}

/// An Unpoly object to process the request headers and set the response headers
///
/// When a request header is accessed, it is automatically added to the `Vary` response header.
///
/// Typical usages:
///
/// ## Ommitting content that isn't targeted
/// <https://unpoly.com/optimizing-responses#omitting-content-that-isnt-targeted>
///
/// ```
/// fn handler(unpoly: unpoly::Unpoly) -> impl IntoResponse {
///     let target = unpoly.target();
///     let html = todo!("render content for target only");
///     (unpoly.get_headers().unwrap(), html)
/// }
/// ```
/// ## Rendering different content for overlays
/// <https://unpoly.com/optimizing-responses#rendering-different-content-for-overlays>
///
/// ```
/// fn handler(unpoly: unpoly::Unpoly) -> impl IntoResponse {
///     let mode = unpoly.mode();
///     let target = unpoly.target();
///     let html = todo!("render content for target in mode only");
///     (unpoly.get_headers().unwrap(), html)
/// }
///
/// ```
///
/// ## Rendering different content for unpoly requests
/// <https://unpoly.com/optimizing-responses#rendering-different-content-for-unpoly-requests>
///
/// ```
/// fn handler(unpoly: unpoly::Unpoly) -> impl IntoResponse {
///     let html = if unpoly.is_up() {
///         todo!("render for fragment update");
///     } else {
///         todo!("render for full page load");
///     }
///     (unpoly.get_headers().unwrap(), html)
/// }
/// ```
///
/// ## Rendering content that depends on layer context
/// <https://unpoly.com/optimizing-responses#rendering-content-that-depends-on-layer-context>
///
/// ```
/// fn handler(unpoly: unpoly::Unpoly) -> impl IntoResponse {
///     let context = unpoly.context();
///     let html = todo!("render html for context");
///     (unpoly.get_headers().unwrap(), html)
/// }
/// ```
///
/// ## Set the title of the page via a fragment update
/// ```
/// fn handler(mut unpoly: unpoly::Unpoly) -> impl IntoResponse {
///     unpoly.set_title("My App");
///     let html = todo!();
///     (unpoly.get_headers().unwrap(), html)
/// }
/// ```
///
/// ## Send events to the frontend
/// ```
/// fn handler(mut unpoly: unpoly::Unpoly) -> impl IntoResponse {
///     unpoly.emit_event("user:created", json!({"id": 152}));
///     // or for a specific layer
///     unpoly.emit_event_layer("user:created", json!({"id": 152}), layer:MatchingLayer::CURRENT);
///     let html = todo!();
///     (unpoly.get_headers().unwrap(), html)
/// }
/// ```
///
/// ## Expire cache
/// ```
/// fn handler(mut unpoly: unpoly::Unpoly) -> impl IntoResponse {
///     unpoly.set_expire_cache("/path/to/expire/*");
///     let html = todo!();
///     (unpoly.get_headers().unwrap(), html)
/// }
/// ```
///
///
/// ## Validating a form
/// <https://unpoly.com/up-validate>
/// ```
/// #[derive(Deserialize)]
///     struct SampleForm{
///     name: String,
///     email: String,
/// }
/// fn handler(mut unpoly: unpoly::Unpoly, extract::Form(form): extract::Form<SampleForm>) -> impl IntoResponse {
///     if !unpoly.validate().is_empty() {
///         todo!("Validate form");
///         let html = todo!("render form with optional errors");
///         (unpoly.get_headers().unwrap(), html)
///     } else {
///         todo!("Process form");
///         let html = todo!("render form with optional errors");
///         (unpoly.get_headers().unwrap(), html)
///     }
/// }
/// ```
#[derive(Debug, Default)]
pub struct Unpoly {
    success: Option<bool>,
    request_version: Option<String>,
    request_context: Option<serde_json::Value>,
    request_fail_context: Option<serde_json::Value>,
    request_fail_mode: LayerMode,
    request_mode: LayerMode,
    request_target: Option<String>,
    request_fail_target: Option<String>,
    request_validate: Vec<String>,
    response_context: Option<serde_json::Value>,
    response_accept_layer: Option<serde_json::Value>,
    response_dismiss_layer: Option<serde_json::Value>,
    response_events: Vec<serde_json::Value>,
    response_evict_cache: Option<String>,
    response_expire_cache: Option<String>,
    response_location: Option<String>,
    response_method: Option<String>,
    response_target: Option<String>,
    response_title: Option<String>,
    response_vary: HashSet<String>,
}

use serde_json::Value;

impl Unpoly {
    /// Returns true if the request is from an Unpoly client
    ///
    /// A request is from an Unpoly client if the `X-Up-Version` header is present
    pub fn is_up(&mut self) -> bool {
        if self.request_version.is_some() {
            self.response_vary.insert("X-Up-Version".to_string());
            true
        } else {
            false
        }
    }

    /// Returns:
    /// - Some(true) if we handle a success case
    /// - Some(false) if we handle a failure case
    /// - None if the success status is not known yet
    pub fn success(&mut self) -> Option<bool> {
        self.success
    }

    /// Set the status to success or fail
    ///
    /// This will also set
    /// - `X-Up-Target` to the same value as `X-Up-[Fail]-Target`
    /// - `mode()` will give the `X-Up[Fail]-Mode` value
    pub fn set_success(&mut self, success: bool) {
        self.success = Some(success);
        if success {
            self.response_vary.insert("X-Up-Target".to_string());
            self.response_target = self.request_target.clone();
        } else {
            self.response_vary.insert("X-Up-Fail-Target".to_string());
            self.response_target = self.request_fail_target.clone();
        }
    }

    /// Returns the current mode
    ///
    /// This will return the X-Up-Mode unless succes is false, in which case it will return the X-Up-Fail-Mode
    pub fn mode(&mut self) -> &LayerMode {
        if let Some(false) = self.success {
            self.response_vary.insert("X-Up-Fail-Mode".to_string());
            &self.request_fail_mode
        } else {
            self.response_vary.insert("X-Up-Mode".to_string());
            &self.request_mode
        }
    }

    pub fn emit_event_layer<S: Serialize>(
        &mut self,
        type_: impl Into<String>,
        event: S,
        matching_layer: MatchingLayer,
    ) -> Result<(), Error> {
        let mut event = serde_json::to_value(event)?;
        if !event.is_object() {
            return Err(Error::EventIsNotSerializableAsObject);
        }

        event.as_object_mut().unwrap().insert(
            "layer".to_string(),
            match matching_layer {
                MatchingLayer::INDEX(index) => Value::Number(index.into()),
                other => serde_json::to_value(other).unwrap(),
            },
        );

        self.emit_event(type_, event)?;
        Ok(())
    }

    pub fn accept_layer<S: Serialize>(&mut self, value: S) -> Result<(), Error> {
        self.response_accept_layer = Some(serde_json::to_value(value)?);
        self.response_dismiss_layer = None;
        Ok(())
    }

    pub fn accept_layer_without_value(&mut self) -> Result<(), Error> {
        self.accept_layer("null")?;
        Ok(())
    }

    pub fn dismiss_layer<S: Serialize>(&mut self, value: S) -> Result<(), Error> {
        self.response_dismiss_layer = Some(serde_json::to_value(value).unwrap());
        self.response_accept_layer = None;
        Ok(())
    }

    pub fn dismiss_layer_without_value(&mut self) -> Result<(), Error> {
        self.dismiss_layer("null")?;
        Ok(())
    }

    /// Get the X-Up-Context response header when set (via `set_context()``), or the X-Up-[Fail-]Context request header
    /// when the response header is not set
    pub fn context(&mut self) -> Option<&Value> {
        if self.response_context.is_some() {
            return self.response_context.as_ref();
        }
        if Some(false) == self.success {
            if self.request_fail_context.is_some() && self.is_up() {
                self.response_vary.insert("X-Up-Fail-Context".to_string());
            }
            self.request_fail_context.as_ref()
        } else {
            if self.request_context.is_some() && self.is_up() {
                self.response_vary.insert("X-Up-Context".to_string());
            }
            self.request_context.as_ref()
        }
    }

    pub fn set_context<S: Serialize>(&mut self, layer: S) {
        self.response_context = Some(serde_json::to_value(layer).unwrap());
    }

    pub fn target(&mut self) -> Option<&str> {
        if self.response_target.is_some() {
            return self.response_target.as_deref();
        }
        if let Some(false) = self.success {
            self.response_vary.insert("X-Up-Fail-Target".to_string());
            self.request_fail_target.as_deref()
        } else {
            self.response_vary.insert("X-Up-Target".to_string());
            self.request_target.as_deref()
        }
    }

    pub fn set_target(&mut self, target: impl Into<String>) {
        self.response_target = Some(target.into());
    }

    pub fn validate(&mut self) -> &Vec<String> {
        if !self.request_validate.is_empty() && self.is_up() {
            self.response_vary.insert("X-Up-Validate".to_string());
        }
        &self.request_validate
    }

    pub fn title(&self) -> Option<&str> {
        self.response_title.as_deref()
    }

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.response_title = Some(title.into());
    }

    pub fn location(&self) -> Option<&str> {
        self.response_location.as_deref()
    }

    pub fn set_location(&mut self, location: impl Into<String>) {
        self.response_location = Some(location.into());
    }

    pub fn method(&mut self) -> Option<&str> {
        self.response_method.as_deref()
    }

    pub fn set_method(&mut self, method: impl Into<String>) {
        self.response_method = Some(method.into());
    }

    pub fn emit_event<S: Serialize>(
        &mut self,
        type_: impl Into<String>,
        event: S,
    ) -> Result<(), Error> {
        let mut event = serde_json::to_value(event)?;
        if !event.is_object() {
            return Err(Error::EventIsNotSerializableAsObject);
        }

        let type_: String = type_.into();

        event
            .as_object_mut()
            .unwrap()
            .insert("type".to_string(), Value::String(type_));

        self.response_events.push(event);
        Ok(())
    }

    pub fn set_evict_cache(&mut self, cache: String) {
        self.response_evict_cache = Some(cache);
    }

    pub fn set_expire_cache(&mut self, cache: String) {
        self.response_expire_cache = Some(cache);
    }

    pub fn get_headers(&self) -> Result<HeaderMap, Error> {
        let mut headers = HeaderMap::new();
        if let Some(title) = &self.response_title {
            headers.insert(headers::TITLE, title.parse().unwrap());
        }
        if let Some(location) = &self.response_location {
            headers.insert(headers::LOCATION, location.parse().unwrap());
        }
        if let Some(accept_layer) = &self.response_accept_layer {
            headers.insert(
                headers::ACCEPT_LAYER,
                serde_json::to_string(accept_layer)?.parse().unwrap(),
            );
        }
        if let Some(dismiss_layer) = &self.response_dismiss_layer {
            headers.insert(
                headers::DISMISS_LAYER,
                serde_json::to_string(dismiss_layer)?.parse().unwrap(),
            );
        }
        if let Some(context) = &self.response_context {
            headers.insert(
                headers::CONTEXT,
                serde_json::to_string(context)?.parse().unwrap(),
            );
        }
        if let Some(target) = &self.response_target {
            headers.insert(headers::TARGET, target.parse().unwrap());
        }
        if let Some(method) = &self.response_method {
            headers.insert(headers::METHOD, method.parse().unwrap());
        }
        if let Some(evict_cache) = &self.response_evict_cache {
            headers.insert(headers::EVICT_CACHE, evict_cache.parse().unwrap());
        }
        if let Some(expire_cache) = &self.response_expire_cache {
            headers.insert(headers::EXPIRE_CACHE, expire_cache.parse().unwrap());
        }
        if !self.response_events.is_empty() {
            let events = serde_json::to_value(&self.response_events)?;
            headers.insert(
                headers::EVENTS,
                serde_json::to_string(&events)?.parse().unwrap(),
            );
        }
        if !self.response_vary.is_empty() {
            let mut vary: Vec<&String> = self.response_vary.iter().collect();
            vary.sort();
            let vary = vary.iter().fold(
                "".to_string(),
                |a, b| if !a.is_empty() { a + "," } else { a } + b,
            );
            headers.insert(headers::VARY, vary.parse().unwrap());
        }
        Ok(headers)
    }
}
