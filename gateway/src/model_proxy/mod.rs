use rocket::routes;
use crate::Args;

mod proxy;
mod components;

pub fn routes() -> Vec<rocket::Route> {
    routes![
        // 对话补全
            proxy::api::chat_completions,
            // 文本转语音
            proxy::api::audio_speech,
            // 图像生成
            proxy::api::images_generations,
            // 图像编辑
            proxy::api::images_edits,
    ]
}

pub async fn init(args: &Args) {
    // 初始化插件管理器
    plugin_manager::init(&args.console).await;

    // 初始化模型
    components::ModelFactory::init().await;
}

