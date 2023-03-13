use poem::handler;

#[handler]
pub fn index() -> &'static str {
    "Hello, world!"
}
