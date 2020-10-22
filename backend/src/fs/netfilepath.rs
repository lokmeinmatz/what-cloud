use std::path::Path;
use std::borrow::Borrow;
use path_slash::PathExt;
use rocket::request::FromFormValue;


#[derive(Debug)]
pub struct NetFilePath(String);

impl NetFilePath {
    #[allow(dead_code)]
    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        Self(path.as_ref().to_slash_lossy())
    }

    pub fn add_prefix<P: AsRef<Path>>(&mut self, prefix: P) {
        let mut n_base: String = prefix.as_ref().to_slash_lossy();
        if n_base.ends_with('/') && self.0.starts_with('/') {
            n_base.push_str(&self.0[1..]);
        } else if n_base.ends_with('/') || self.0.starts_with('/') {
            n_base.push_str(&self.0);
        } else if self.0.len() > 0 {
            n_base.push('/');
            n_base.push_str(&self.0);
        }
        self.0 = n_base;
    }
}

impl Borrow<str> for NetFilePath {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl Borrow<Path> for NetFilePath {
    fn borrow(&self) -> &Path {
        &Path::new(&self.0)
    }
}

impl<'v> FromFormValue<'v> for NetFilePath {
    type Error = ();
    fn from_form_value(raw: &'v rocket::http::RawStr) -> Result<Self, Self::Error> {
        //dbg!(raw);
        let mut raw_path: String = match raw.percent_decode() {
            Ok(s) => s.into_owned(),
            Err(_) => {
                return Err(());
            }
        };

        if raw_path.contains("..") {
            // illegal
            // TODO are there other symbolic links or ways to escape the dir?
            return Err(());
        };
        if raw_path.starts_with('/') {
            raw_path.remove(0);
        }
        Ok(NetFilePath(Path::new(&raw_path).to_slash_lossy()))
    }
}


#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_netfilepath() {
        let mut nfp = NetFilePath::from_path("/folder1/test");
        assert_eq!(Borrow::<str>::borrow(&nfp), "/folder1/test");
        nfp.add_prefix("\\User1\\");
        assert_eq!(Borrow::<str>::borrow(&nfp), "/User1/folder1/test");
    }
}
