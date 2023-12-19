use rust_embed::RustEmbed;

pub use generator::SwaggerGeneratorExt;

mod generator;

#[derive(RustEmbed)]
#[folder = "statics/swagger"]
struct Asset;
