use crate::shared::FileUpload;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;

#[derive(Serialize, Deserialize, Debug, Default, Builder, Clone, PartialEq)]
#[builder(name = "CreateImageParametersBuilder")]
#[builder(setter(into, strip_option), default)]
pub struct CreateImageParameters {
    /// 所需图像的文字描述。对于gpt-image-1最大长度为32000个字符，对于dall-e-2为1000个字符，对于dall-e-3为4000个字符。
    pub prompt: String,
    /// 允许为生成的图像设置背景透明度。此参数仅支持gpt-image-1。
    /// 必须是transparent、opaque或auto（默认值）之一。当使用auto时，模型将自动确定图像的最佳背景。
    /// 如果设置为transparent，输出格式需要支持透明度，因此应设置为png（默认值）或webp。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<BackgroundStyle>,
    /// 用于图像生成的模型。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// 控制gpt-image-1生成图像的内容审核级别。必须是low（较少限制性过滤）或auto（默认值）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moderation: Option<ModerationLevel>,
    /// 要生成的图像数量。必须介于1到10之间。对于dall-e-3，仅支持n=1。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    /// 生成图像的压缩级别（0-100%）。此参数仅支持gpt-image-1与webp或jpeg输出格式，并默认为100。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_compression: Option<u32>,
    /// 返回生成图像的格式。此参数仅支持gpt-image-1。必须是png、jpeg或webp之一。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<OutputFormat>,
    /// 要生成的部分图像数量。此参数用于返回部分图像的流式响应。
    /// 值必须在0到3之间。设置为0时，响应将是单个图像在一个流事件中发送。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partial_images: Option<u32>,
    /// 将生成的图像质量。hd创建具有更精细细节和更大一致性的图像。
    /// gpt-image-1支持high、medium和low。dall-e-3支持hd和standard。dall-e-2只支持standard选项。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<ImageQuality>,
    /// 返回生成图像的格式。必须是url或b64_json之一。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,
    /// 生成图像的尺寸。对于gpt-image-1必须是1024x1024、1536x1024（横向）、1024x1536（纵向）或auto（默认值），
    /// 对于dall-e-2是256x256、512x512或1024x1024之一，对于dall-e-3是1024x1024、1792x1024或1024x1792之一。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<ImageSize>,
    /// 在流模式下生成图像。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    /// 生成图像的风格。此参数仅支持dall-e-3。必须是vivid或natural之一。
    /// Vivid使模型倾向于生成超现实和戏剧性图像。
    /// Natural使模型产生更自然、不太超现实的图像。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<ImageStyle>,
    /// 一个稳定标识符，用于帮助检测可能违反OpenAI使用政策的应用程序用户。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_identifier: Option<String>,
    /// OpenAI用于缓存类似请求响应以优化缓存命中率。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_key: Option<String>,
    /// 表示您的最终用户的唯一标识符，可以帮助OpenAI监控和检测滥用行为。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    /// 附加参数
    pub ext: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Serialize, Deserialize, Debug, Default, Builder, Clone, PartialEq)]
#[builder(name = "EditImageParametersBuilder")]
#[builder(setter(into, strip_option), default)]
pub struct EditImageParameters {
    /// 要编辑的图像。可以是base64编码的图像数据或者URL。
    pub image: String,
    /// 提示词。
    pub prompt: Option<String>,
    /// 允许为生成的图像设置背景透明度。此参数仅支持gpt-image-1。
    /// 必须是transparent、opaque或auto（默认值）之一。当使用auto时，模型将自动确定图像的最佳背景。
    /// 如果设置为transparent，输出格式需要支持透明度，因此应设置为png（默认值）或webp。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<BackgroundStyle>,
    /// 控制模型在匹配输入图像的风格和特征（尤其是面部特征）上付出的努力程度。
    /// 此参数仅支持gpt-image-1。不支持gpt-image-1-mini。支持high和low。默认为low。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_fidelity: Option<InputFidelity>,
    /// 另一个图像，其完全透明区域（例如alpha为零的地方）指示应该编辑图像的位置。
    /// 如果提供了多个图像，则遮罩将应用于第一个图像。
    /// 必须是有效的PNG文件，小于4MB，并且具有与图像相同的尺寸。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mask: Option<FileUpload>,
    /// 图像的mime类型。如果未提供，mime类型将被设置为application/octet-stream。
    /// gpt-image-1期望`image/png`、`image/jpeg`或`image/webp`。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<MimeType>,
    /// 用于图像生成的模型。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// 要生成的图像数量。必须介于1到10之间。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    /// 生成图像的压缩级别（0-100%）。此参数仅支持gpt-image-1与webp或jpeg输出格式，并默认为100。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_compression: Option<u32>,
    /// 返回生成图像的格式。此参数仅支持gpt-image-1。必须是png、jpeg或webp之一。默认值是png。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<OutputFormat>,
    /// 要生成的部分图像数量。此参数用于返回部分图像的流式响应。
    /// 值必须在0到3之间。设置为0时，响应将是单个图像在一个流事件中发送。
    /// 注意，如果完整图像生成得更快，则最终图像可能会在生成完全部分图像之前发送。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partial_images: Option<u32>,
    /// 将生成的图像质量。hd创建具有更精细细节和更大一致性的图像。
    /// gpt-image-1支持high、medium和low。dall-e-2仅支持standard质量。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<ImageQuality>,
    /// 返回生成图像的格式。必须是url或b64_json之一。URL在图像生成后仅在60分钟内有效。此参数仅支持dall-e-2，因为gpt-image-1将始终返回base64编码的图像。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,
    /// 生成图像的尺寸。对于gpt-image-1必须是1024x1024、1536x1024（横向）、1024x1536（纵向）或auto（默认值），
    /// 对于dall-e-2是256x256、512x512或1024x1024之一，对于dall-e-3是1024x1024、1792x1024或1024x1792之一。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<ImageSize>,
    /// 在流模式下编辑图像。默认为false。请参阅图像生成指南以获取更多信息。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    /// 表示您的最终用户的唯一标识符，可以帮助OpenAI监控和检测滥用行为。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default, Builder, Clone, PartialEq)]
#[builder(name = "CreateImageVariationParametersBuilder")]
#[builder(setter(into, strip_option), default)]
pub struct CreateImageVariationParameters {
    /// 用作变体基础的图像。必须是有效的PNG文件，小于4MB，且为正方形。
    pub image: FileUpload,
    /// 用于图像生成的模型。目前仅支持dall-e-2。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// 要生成的图像数量。必须介于1到10之间。对于dall-e-3，仅支持n=1。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    /// 返回生成图像的格式。必须是url或b64_json之一。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,
    /// 生成图像的尺寸。对于gpt-image-1必须是1024x1024、1536x1024（横向）、1024x1536（纵向）或auto（默认值），
    /// 对于dall-e-2是256x256、512x512或1024x1024之一，对于dall-e-3是1024x1024、1792x1024或1024x1792之一。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<ImageSize>,
    /// 表示您的最终用户的唯一标识符，可以帮助OpenAI监控和检测滥用行为。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ImageResponse {
    /// 用于图像生成的背景参数。可以是transparent或opaque。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<BackgroundStyle>,
    /// 图像创建时的Unix时间戳（以秒为单位）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<u32>,
    /// 生成图像的列表。
    pub data: Vec<ImageData>,
    /// 图像生成的输出格式。可以是png、webp或jpeg。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<OutputFormat>,
    /// 生成图像的质量。可以是low、medium或high。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<ImageQuality>,
    /// 生成图像的尺寸。可以是1024x1024、1024x1536或1536x1024。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<ImageSizeResponse>,
    /// 仅针对gpt-image-1，图像生成的令牌使用信息。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<ImageUsage>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ImageSize {
    #[serde(rename = "256x256")]
    Size256X256,
    #[serde(rename = "512x512")]
    Size512X512,
    #[serde(rename = "1024x1024")]
    Size1024X1024,
    #[serde(rename = "1024x1536")]
    Size1024X1536,
    #[serde(rename = "1536x1024")]
    Size1536X1024,
    #[serde(rename = "1792x1024")]
    Size1792X1024,
    #[serde(rename = "1024x1792")]
    Size1024X1792,
    #[serde(rename = "auto")]
    Auto,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ImageSizeResponse {
    #[serde(rename = "256x256")]
    Size256X256,
    #[serde(rename = "512x512")]
    Size512X512,
    #[serde(rename = "1024x1024")]
    Size1024X1024,
    #[serde(rename = "1024x1536")]
    Size1024X1536,
    #[serde(rename = "1536x1024")]
    Size1536X1024,
    #[serde(rename = "1792x1024")]
    Size1792X1024,
    #[serde(rename = "1024x1792")]
    Size1024X1792,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BackgroundStyle {
    Transparent,
    Opaque,
    Auto,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ImageQuality {
    Standard,
    Hd,
    High,
    Medium,
    Low,
    Auto,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MimeType {
    #[serde(rename = "image/png")]
    Png,
    #[serde(rename = "image/jpeg")]
    Jpeg,
    #[serde(rename = "image/webp")]
    Webp,
    #[serde(rename = "application/octet-stream")]
    OctetStream,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ImageStyle {
    Vivid,
    Natural,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ModerationLevel {
    Auto,
    Low,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    Png,
    Jpeg,
    Webp,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum InputFidelity {
    High,
    Low,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ResponseFormat {
    Url,
    B64Json,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ImageUsage {
    /// 输入提示中的令牌数（图像和文本）。
    #[serde(default = "zero")]
    pub input_tokens: u32,
    /// 图像生成的输入令牌详细信息。
    pub input_tokens_details: Option<InputTokensDetails>,
    /// 模型生成的输出令牌数。
    #[serde(default = "zero")]
    pub output_tokens: u32,
    /// 用于图像生成的总令牌数（图像和文本）。
    #[serde(default = "zero")]
    pub total_tokens: u32,
}
fn zero() -> u32 {
    0
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct InputTokensDetails {
    /// 输入中的图像令牌数。
    pub image_tokens: u32,
    /// 输入中的文本令牌数。
    pub text_tokens: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum ImageData {
    Url {
        url: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        revised_prompt: Option<String>,
    },
    B64Json {
        b64_json: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        revised_prompt: Option<String>,
    },
}

impl Display for InputFidelity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                InputFidelity::High => "high",
                InputFidelity::Low => "low",
            }
        )
    }
}

impl Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                OutputFormat::Png => "png",
                OutputFormat::Jpeg => "jpeg",
                OutputFormat::Webp => "webp",
            }
        )
    }
}

impl Display for BackgroundStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BackgroundStyle::Transparent => "transparent",
                BackgroundStyle::Opaque => "opaque",
                BackgroundStyle::Auto => "auto",
            }
        )
    }
}

impl Display for ImageQuality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ImageQuality::Standard => "standard",
                ImageQuality::Hd => "hd",
                ImageQuality::High => "high",
                ImageQuality::Medium => "medium",
                ImageQuality::Low => "low",
                ImageQuality::Auto => "auto",
            }
        )
    }
}

impl Display for ImageSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ImageSize::Size256X256 => "256x256",
                ImageSize::Size512X512 => "512x512",
                ImageSize::Size1024X1024 => "1024x1024",
                ImageSize::Size1536X1024 => "1536x1024",
                ImageSize::Size1024X1536 => "1024x1536",
                ImageSize::Size1792X1024 => "1792x1024",
                ImageSize::Size1024X1792 => "1024x1792",
                ImageSize::Auto => "auto",
            }
        )
    }
}

impl Display for MimeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                MimeType::Png => "image/png",
                MimeType::Jpeg => "image/jpeg",
                MimeType::Webp => "image/webp",
                MimeType::OctetStream => "application/octet-stream",
            }
        )
    }
}

impl Display for ResponseFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ResponseFormat::Url => "url",
                ResponseFormat::B64Json => "b64_json",
            }
        )
    }
}
