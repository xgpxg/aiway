#[derive(Embed)]
#[folder = "ui/"]
struct Asset;

use rocket::get;
use rocket::http::ContentType;
use rocket::http::uri::Segments;
use rocket::http::uri::fmt::Path;
use rust_embed::Embed;

/// 嵌入web处理
#[get("/<path..>", rank = 500)]
pub(crate) async fn web(path: Segments<'_, Path>) -> Option<(ContentType, Vec<u8>)> {
    println!("web: {:?}", path);
    let path = path.collect::<Vec<_>>().join("/");
    let index = Some((ContentType::HTML, Asset::get("index.html")?.data.into()));
    if path == "" {
        return index;
    }
    let ext = path.split('.').last()?;
    if let Some(embedded_file) = Asset::get(&path) {
        let data: Vec<u8> = embedded_file.data.into();
        Some((ContentType::from_extension(ext)?, data))
    } else {
        index
    }
}
