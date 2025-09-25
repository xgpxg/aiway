use protocol::common::req::Pagination;
use rbatis::PageRequest;

/// 分页trait
pub trait RbPage: Pagination {
    /// 转换为rbatis的分页参数
    fn to_rb_page(&self) -> PageRequest {
        PageRequest::new(self.page_num(), self.page_size())
    }
}

#[macro_export]
macro_rules! impl_rb_page {
    ($s:ty) => {
        impl $crate::server::common::page::RbPage for $s {}
    };
}
