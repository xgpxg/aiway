use rust_embed::Embed;

#[derive(Embed)]
#[folder = "resources/"]
pub(crate) struct Asset;

pub(crate) async fn download_ip_region_file() -> anyhow::Result<Vec<u8>> {
    let file = Asset::get("ip2region_v4.xdb").unwrap();
    Ok(file.data.to_vec())
}
