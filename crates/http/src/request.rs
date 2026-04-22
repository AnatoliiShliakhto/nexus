pub use http::Error as RequestError;
use http::header::{AsHeaderName, HeaderMap};

/// Extension trait to HTTP requests.
pub trait RequestHeadersExt {
    /// Retrieves the value of a request header by name.
    fn header_value(&self, name: impl AsHeaderName) -> Option<String>;
    /// Retrieves the values of a request header by name.
    fn header_values(&self, name: impl AsHeaderName) -> Vec<String>;
}

impl RequestHeadersExt for HeaderMap {
    fn header_value(&self, name: impl AsHeaderName) -> Option<String> {
        self.get(name).and_then(|v| v.to_str().ok()).map(ToOwned::to_owned)
    }

    fn header_values(&self, name: impl AsHeaderName) -> Vec<String> {
        self.get_all(name).iter().filter_map(|v| v.to_str().ok().map(ToOwned::to_owned)).collect()
    }
}

impl<T> RequestHeadersExt for http::Request<T> {
    fn header_value(&self, name: impl AsHeaderName) -> Option<String> {
        self.headers().header_value(name)
    }

    fn header_values(&self, name: impl AsHeaderName) -> Vec<String> {
        self.headers().header_values(name)
    }
}
