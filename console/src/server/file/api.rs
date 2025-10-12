use crate::server::file::file_util::make_save_file;
use common::data_dir;
use logging::log;
use protocol::common::res::Res;
use rocket::form::Form;
use rocket::fs::{NamedFile, TempFile};
use rocket::{FromForm, get, post, routes};

pub fn routes() -> Vec<rocket::Route> {
    routes![download_file, upload_file]
}

/// 下载文件接口
///
/// 文件名格式要求：YYYYMMDD-10位随机字符串_原始文件名
#[get("/download/<file_name>")]
pub async fn download_file(file_name: &str) -> Result<NamedFile, rocket::http::Status> {
    // 截取日期
    let date = &file_name[0..8];
    // 文件存储路径
    let file_path = data_dir!("file", date, file_name);
    let file = NamedFile::open(file_path)
        .await
        .map_err(|_| rocket::http::Status::NotFound)?;
    Ok(file)
}

#[derive(Debug, FromForm)]
pub(self) struct UploadFile<'a> {
    file_name: String,
    file: TempFile<'a>,
}
/// 上传文件接口，按天分文件夹
#[post("/upload", data = "<req>")]
async fn upload_file(req: Form<UploadFile<'_>>) -> Result<Res<String>, rocket::http::Status> {
    let UploadFile {
        file_name,
        mut file,
    } = req.into_inner();
    let (save_file_name, save_file) = make_save_file(&file_name).map_err(|e| {
        log::error!("[文件上传]生成文件名失败，原因：{}", e);
        rocket::http::Status::InternalServerError
    })?;
    file.persist_to(&save_file).await.map_err(|e| {
        log::error!("[文件上传]文件保存失败，原因：{}", e);
        rocket::http::Status::InternalServerError
    })?;

    Ok(Res::success(format!("/file/download/{}", save_file_name)))
}
