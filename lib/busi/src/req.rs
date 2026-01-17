//! # 公共请求参数
//!
//! - 单个ID请求
//! - 多个ID请求
//! - 分页请求

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdReq {
    pub id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdsReq {
    pub ids: Vec<i64>,
}

/// 分页
pub trait Pagination {
    /// 页码
    fn page_num(&self) -> u64;
    /// 每页数量
    fn page_size(&self) -> u64;

    #[cfg(feature = "rbatis")]
    fn to_rb_page(&self) -> rbatis::PageRequest {
        rbatis::PageRequest::new(self.page_num(), self.page_size())
    }
}

/// 分页请求参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageReq {
    pub page_num: u64,
    pub page_size: u64,
}

impl Default for PageReq {
    fn default() -> Self {
        PageReq {
            page_num: 1,
            page_size: 10,
        }
    }
}

/// 转换分页参数
///
/// 使用方式：
/// ```rust
/// struct SomePageReq {
///     page: PageReq
/// }
/// impl_pagination!(SomePageReq);
/// ```
#[macro_export]
macro_rules! impl_pagination {
    ($s:ty) => {
        impl $crate::req::Pagination for $s {
            fn page_num(&self) -> u64 {
                self.page.page_num
            }

            fn page_size(&self) -> u64 {
                self.page.page_size
            }
        }
    };
}
