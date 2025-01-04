# Unpoly in Rust

## What is Unpoly?

[Unpoly](https://unpoly.com) is an unobtrusive Javascript framework for applications that render on the server. It
allows your views to do things that are not normally possible in HTML, such as having links update only fragments
of a page, or opening links in modal dialogs.

Unpoly can give your server-side application a fast and flexible frontend that feel like a single-page application
(SPA). It also preserves the resilience and simplicity of the server-side programming model.

So with Unpoly you hardly have to write any Javascript for the frontend; your site is almost completely written is
server-side delivered/generated html and css. No need to write code in both the frontend and backend
for getting the data, validating forms, etc.

## How doe Unpoly-rs helps?

Unpoly works with regular html files. If the backend serve html files, static or dynamic, you're good
to go. However, when the backends implements the [Unpoly Server Protocol](https://unpoly.com/up.protocol), the
responses can be exactly suited to match the requests.

## Typical usages

```
use axum::response::IntoResponse;
use axum::extract;
use serde::Deserialize;
use serde_json::json;

///  Omitting content that isn't targeted
/// https://unpoly.com/optimizing-responses#omitting-content-that-isnt-targeted
fn handler_target(mut unpoly: unpoly::Unpoly) -> impl IntoResponse {
    let target = unpoly.target();
    let html = todo!("render content for target only");
    (unpoly.get_headers().unwrap(), html)
}

///  Rendering different content for overlays
/// https://unpoly.com/optimizing-responses#rendering-different-content-for-overlays
fn handler_mode_target(mut unpoly: unpoly::Unpoly) -> impl IntoResponse {
    let mode = unpoly.mode();
    let target = unpoly.target();
    let html = todo!("render content for target in mode only");
    (unpoly.get_headers().unwrap(), html)
}

/// Rendering different content for unpoly requests
/// https://unpoly.com/optimizing-responses#rendering-different-content-for-unpoly-requests
fn handler_full_or_fragment(mut unpoly: unpoly::Unpoly) -> impl IntoResponse {
    let html: String = if unpoly.is_up() {
        todo!("render for fragment update")
    } else {
        todo!("render for full page load")
    };
    (unpoly.get_headers().unwrap(), html)
}

/// Rendering content that depends on layer context
/// https://unpoly.com/optimizing-responses#rendering-content-that-depends-on-layer-context
fn handler_context(mut unpoly: unpoly::Unpoly) -> impl IntoResponse {
    let context = unpoly.context();
    let html = todo!("render html for context");
    (unpoly.get_headers().unwrap(), html)
}

/// Set the title of the page via a fragment update
fn handler_title(mut unpoly: unpoly::Unpoly) -> impl IntoResponse {
    unpoly.set_title("My App");
    let html = todo!();
    (unpoly.get_headers().unwrap(), html)
}

/// Send events to the frontend
fn handler_emit_event(mut unpoly: unpoly::Unpoly) -> impl IntoResponse {
    unpoly.emit_event("user:created", json!({"id": 152}));
    // or for a specific layer
    unpoly.emit_event_layer("user:created", json!({"id": 152}), unpoly::MatchingLayer::CURRENT);
    let html = todo!();
    (unpoly.get_headers().unwrap(), html)
}

/// Expire cache
fn handler_cache(mut unpoly: unpoly::Unpoly) -> impl IntoResponse {
    unpoly.set_expire_cache("/path/to/expire/*");
    let html = todo!();
    (unpoly.get_headers().unwrap(), html)
}

/// Validating a form
/// https://unpoly.com/up-validate
#[derive(Deserialize)]
    struct SampleForm{
    name: String,
    email: String,
}
fn handler_validate(mut unpoly: unpoly::Unpoly, extract::Form(form): extract::Form<SampleForm>) -> impl IntoResponse {
    if !unpoly.validate().is_empty() {
        todo!("Validate form");
        let html = todo!("render form with optional errors");
        (unpoly.get_headers().unwrap(), html)
    } else {
        todo!("Process form");
        let html = todo!("render form with optional errors");
        (unpoly.get_headers().unwrap(), html)
    }
}
```