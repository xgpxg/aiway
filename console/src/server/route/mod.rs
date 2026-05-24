pub mod api;
mod request;
mod response;
mod service;

use logging::log;
use matchit::InsertError;
pub use request::RouteListReq;

/// 路由路径匹配模式
///
/// 用于将 * 和 ** 格式的路径转换为 matchit 支持的 {p} 和 {*p} 格式
pub struct PathPattern(String);
impl PathPattern {
    pub fn new<P: Into<String>>(path: P) -> Self {
        PathPattern(path.into())
    }

    pub fn to_patterns(&self) -> Vec<String> {
        let mut result = Vec::new();
        let mut param_count = 1;
        let mut chars = self.0.chars().peekable();
        let mut i = 0;
        let mut double_star_pos: Option<usize> = None;

        let mut pattern = String::new();
        while let Some(ch) = chars.next() {
            if ch == '*' {
                // Check if it's a tailing "**" capturing all remaining path
                if chars.peek() == Some(&'*') {
                    chars.next(); // consume the second '*'
                    pattern.push_str("{*p}");
                    double_star_pos = Some(i);
                    break;
                } else {
                    // Single '*' - named parameter
                    pattern.push_str(&format!("{{p{}}}", param_count));
                    param_count += 1;
                }
            } else {
                pattern.push(ch);
            }
            i += 1;
        }

        result.push(pattern);

        // 如果有双星号，添加匹配空路径的
        // 例如：/aa/** -> /aa/, /** -> /
        if let Some(pos) = double_star_pos
            && pos < self.0.len()
        {
            result.push(self.0[..pos].to_string());
        }

        log::debug!("{} => {:?}", self.0, result);

        result
    }
}

struct PathPatterns(Vec<PathPattern>);
impl PathPatterns {
    fn new<S: IntoIterator<Item = String>>(paths: S) -> Self {
        PathPatterns(
            paths
                .into_iter()
                .map(PathPattern::new)
                .collect::<Vec<PathPattern>>(),
        )
    }
}

impl TryFrom<PathPatterns> for matchit::Router<()> {
    type Error = String;

    fn try_from(patterns: PathPatterns) -> Result<Self, Self::Error> {
        let mut router = matchit::Router::new();
        for pattern in patterns.0 {
            let results = pattern.to_patterns();
            for result in results {
                if let Err(e) = router.insert(result, ()) {
                    return match e {
                        InsertError::Conflict { .. } => Err(format!("路由路径冲突：{}", pattern.0)),
                        InsertError::InvalidCatchAll => {
                            Err("通配符 ** 仅支持添加在路径尾部".to_string())
                        }
                        _ => Err(format!("路由路径解析错误：{}", e)),
                    };
                }
            }
        }

        Ok(router)
    }
}

#[cfg(test)]
mod tests {
    use crate::server::route::PathPatterns;

    #[test]
    fn test_match() {
        let router = PathPatterns::new(vec![
            "/users/*/profile".to_string(),
            "/users/*/profile/*".to_string(),
            "/users/*/*/".to_string(),
            "/users/*/hello/*/".to_string(),
            "/users/*/all/**".to_string(),
        ]);

        let router: matchit::Router<()> = router.try_into().unwrap();

        assert!(router.at("/users/123/profile").is_ok());
        assert!(router.at("/users/123/profile/").is_ok());
        assert!(router.at("/users/123/profile/abc").is_ok());
        assert!(router.at("/users/123/hello/abc").is_err());
        assert!(router.at("/users/123/hello/abc/").is_ok());
        assert!(router.at("/users/123/all/abc/").is_ok());
        assert!(router.at("/users/123/all/abc/def").is_ok());
        assert!(router.at("/users/123/all/abc/def/").is_ok());
        assert!(router.at("/hello").is_err());
    }
}
