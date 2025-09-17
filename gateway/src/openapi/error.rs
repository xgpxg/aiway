use rocket::Request;
use rocket::response::Responder;

/// 网关错误定义
pub enum GatewayError {
    /// 网关错误，对应状态码：502
    ///
    /// 当没有匹配的路由、网关本身错误时，返回该错误
    BadGateway,
    /// 服务错误，对应状态码：503
    ///
    /// 当服务本身错误，如无响应时，返回该错误
    ServiceUnavailable,
}

impl<'r> Responder<'r, 'r> for GatewayError {
    fn respond_to(self, _request: &'r Request<'_>) -> rocket::response::Result<'r> {
        match self {
            GatewayError::BadGateway => rocket::response::Response::build()
                .status(rocket::http::Status::BadGateway)
                .ok(),
            GatewayError::ServiceUnavailable => rocket::response::Response::build()
                .status(rocket::http::Status::ServiceUnavailable)
                .ok(),
        }
    }
}
