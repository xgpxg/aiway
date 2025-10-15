//! 内部sdk
pub struct SDK;

#[cfg(test)]
mod tests {
    use std::borrow::Cow;
    use std::ffi::{CStr, CString, OsStr, OsString};
    use std::ops::Deref;
    use std::path::Path;

    #[test]
    fn test_str() {
        let s = Cow::from("");

        let s = String::from("hello");
        let s = String::from_utf8(vec![97]).unwrap();
        s.deref();
        let s = String::from_utf8_lossy(&[97]);

        let os_str: &OsStr = "filename.txt".as_ref();

        let path = Path::new("filename.txt");

        let env = std::env::var("PATH").unwrap();

        let mut s: OsString = OsString::from("hello");

        let s: &CStr = CStr::from_bytes_with_nul(b"hello\0").unwrap();
        let s: CString = CString::new("hello").unwrap();

        println!("{}", s.to_string_lossy());
    }
}
