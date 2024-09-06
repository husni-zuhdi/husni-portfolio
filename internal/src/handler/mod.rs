pub mod blog;
pub mod profile;
pub mod status;
pub mod version;

// Note: In axum [example](https://docs.rs/axum/latest/axum/response/index.html#building-responses)
// They show an example to return Html<&'static str>
// Instaed of Html<String>. But using static give me a headache :")
