use axum::http::HeaderName;

pub mod admin;
pub mod auth;
pub mod blogs;
pub mod profile;
pub mod status;
pub mod talks;
pub mod version;

/// HTMX header to redirect client to specific path
pub const HX_REDIRECT: HeaderName = HeaderName::from_static("hx-redirect");

// Note: In axum [example](https://docs.rs/axum/latest/axum/response/index.html#building-responses)
// They show an example to return Html<&'static str>
// Instaed of Html<String>. But using static give me a headache :")
