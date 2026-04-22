use std::fmt;
use std::str::FromStr;

/// A generic error status derived from HTTP status codes to ensure compatibility.
#[cfg_attr(feature = "json", derive(serde::Serialize))]
#[derive(PartialEq, Eq, Clone, Copy, Hash)]
#[repr(u16)]
pub enum ErrorStatus {
    // 1xx Informational
    /// 100 Continue
    Continue = 100,
    /// 101 Switching Protocols
    SwitchingProtocols = 101,
    /// 102 Processing
    Processing = 102,
    /// 103 Early Hints
    EarlyHints = 103,

    // 2xx Success
    /// 200 OK | Success
    Ok = 200,
    /// 201 Created
    Created = 201,
    /// 202 Accepted
    Accepted = 202,
    /// 203 Non-Authoritative Information
    NonAuthoritativeInformation = 203,
    /// 204 No Content
    NoContent = 204,
    /// 205 Reset Content
    ResetContent = 205,
    /// 206 Partial Content
    PartialContent = 206,
    /// 207 Multi-Status
    MultiStatus = 207,
    /// 208 Already Reported
    AlreadyReported = 208,
    /// 226 IM Used
    ImUsed = 226,

    // 3xx Redirection
    /// 300 Multiple Choices
    MultipleChoices = 300,
    /// 301 Moved Permanently
    MovedPermanently = 301,
    /// 302 Found
    Found = 302,
    /// 303 See Other
    SeeOther = 303,
    /// 304 Not Modified
    NotModified = 304,
    /// 305 Use Proxy
    UseProxy = 305,
    /// 307 Temporary Redirect
    TemporaryRedirect = 307,
    /// 308 Permanent Redirect
    PermanentRedirect = 308,

    // 4xx Client Error
    /// 400 Bad Request
    BadRequest = 400,
    /// 401 Unauthorized
    Unauthorized = 401,
    /// 402 Payment Required
    PaymentRequired = 402,
    /// 403 Forbidden
    Forbidden = 403,
    /// 404 Not Found
    NotFound = 404,
    /// 405 Method Not Allowed
    MethodNotAllowed = 405,
    /// 406 Not Acceptable
    NotAcceptable = 406,
    /// 407 Proxy Authentication Required
    ProxyAuthenticationRequired = 407,
    /// 408 Request Timeout
    RequestTimeout = 408,
    /// 409 Conflict
    Conflict = 409,
    /// 410 Gone
    Gone = 410,
    /// 411 Length Required
    LengthRequired = 411,
    /// 412 Precondition Failed
    PreconditionFailed = 412,
    /// 413 Payload Too Large
    PayloadTooLarge = 413,
    /// 414 URI Too Long
    UriTooLong = 414,
    /// 415 Unsupported Media Type
    UnsupportedMediaType = 415,
    /// 416 Range Not Satisfiable
    RangeNotSatisfiable = 416,
    /// 417 Expectation Failed
    ExpectationFailed = 417,
    /// 418 I'm a teapot
    ImATeapot = 418,
    /// 421 Misdirected Request
    MisdirectedRequest = 421,
    /// 422 Unprocessable Entity
    UnprocessableEntity = 422,
    /// 423 Locked
    Locked = 423,
    /// 424 Failed Dependency
    FailedDependency = 424,
    /// 425 Too Early
    TooEarly = 425,
    /// 426 Upgrade Required
    UpgradeRequired = 426,
    /// 428 Precondition Required
    PreconditionRequired = 428,
    /// 429 Too Many Requests
    TooManyRequests = 429,
    /// 431 Request Header Fields Too Large
    RequestHeaderFieldsTooLarge = 431,
    /// 451 Unavailable For Legal Reasons
    UnavailableForLegalReasons = 451,

    // 5xx Server Error
    /// 500 Internal Server Error
    InternalServerError = 500,
    /// 501 Not Implemented
    NotImplemented = 501,
    /// 502 Bad Gateway
    BadGateway = 502,
    /// 503 Service Unavailable
    ServiceUnavailable = 503,
    /// 504 Gateway Timeout
    GatewayTimeout = 504,
    /// 505 HTTP Version Not Supported
    HttpVersionNotSupported = 505,
    /// 506 Variant Also Negotiates
    VariantAlsoNegotiates = 506,
    /// 507 Insufficient Storage
    InsufficientStorage = 507,
    /// 508 Loop Detected
    LoopDetected = 508,
    /// 510 Not Extended
    NotExtended = 510,
    /// 511 Network Authentication Required
    NetworkAuthenticationRequired = 511,
}

impl ErrorStatus {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Continue => "Continue",
            Self::SwitchingProtocols => "SwitchingProtocols",
            Self::Processing => "Processing",
            Self::EarlyHints => "EarlyHints",
            Self::Ok => "Ok",
            Self::Created => "Created",
            Self::Accepted => "Accepted",
            Self::NonAuthoritativeInformation => "NonAuthoritativeInformation",
            Self::NoContent => "NoContent",
            Self::ResetContent => "ResetContent",
            Self::PartialContent => "PartialContent",
            Self::MultiStatus => "MultiStatus",
            Self::AlreadyReported => "AlreadyReported",
            Self::ImUsed => "ImUsed",
            Self::MultipleChoices => "MultipleChoices",
            Self::MovedPermanently => "MovedPermanently",
            Self::Found => "Found",
            Self::SeeOther => "SeeOther",
            Self::NotModified => "NotModified",
            Self::UseProxy => "UseProxy",
            Self::TemporaryRedirect => "TemporaryRedirect",
            Self::PermanentRedirect => "PermanentRedirect",
            Self::BadRequest => "BadRequest",
            Self::Unauthorized => "Unauthorized",
            Self::PaymentRequired => "PaymentRequired",
            Self::Forbidden => "Forbidden",
            Self::NotFound => "NotFound",
            Self::MethodNotAllowed => "MethodNotAllowed",
            Self::NotAcceptable => "NotAcceptable",
            Self::ProxyAuthenticationRequired => "ProxyAuthenticationRequired",
            Self::RequestTimeout => "RequestTimeout",
            Self::Conflict => "Conflict",
            Self::Gone => "Gone",
            Self::LengthRequired => "LengthRequired",
            Self::PreconditionFailed => "PreconditionFailed",
            Self::PayloadTooLarge => "PayloadTooLarge",
            Self::UriTooLong => "UriTooLong",
            Self::UnsupportedMediaType => "UnsupportedMediaType",
            Self::RangeNotSatisfiable => "RangeNotSatisfiable",
            Self::ExpectationFailed => "ExpectationFailed",
            Self::ImATeapot => "ImATeapot",
            Self::MisdirectedRequest => "MisdirectedRequest",
            Self::UnprocessableEntity => "UnprocessableEntity",
            Self::Locked => "Locked",
            Self::FailedDependency => "FailedDependency",
            Self::TooEarly => "TooEarly",
            Self::UpgradeRequired => "UpgradeRequired",
            Self::PreconditionRequired => "PreconditionRequired",
            Self::TooManyRequests => "TooManyRequests",
            Self::RequestHeaderFieldsTooLarge => "RequestHeaderFieldsTooLarge",
            Self::UnavailableForLegalReasons => "UnavailableForLegalReasons",
            Self::InternalServerError => "InternalServerError",
            Self::NotImplemented => "NotImplemented",
            Self::BadGateway => "BadGateway",
            Self::ServiceUnavailable => "ServiceUnavailable",
            Self::GatewayTimeout => "GatewayTimeout",
            Self::HttpVersionNotSupported => "HttpVersionNotSupported",
            Self::VariantAlsoNegotiates => "VariantAlsoNegotiates",
            Self::InsufficientStorage => "InsufficientStorage",
            Self::LoopDetected => "LoopDetected",
            Self::NotExtended => "NotExtended",
            Self::NetworkAuthenticationRequired => "NetworkAuthenticationRequired",
        }
    }

    /// Returns the numeric value of the status code.
    #[inline]
    #[must_use]
    pub const fn as_u16(&self) -> u16 {
        *self as u16
    }
}

impl fmt::Display for ErrorStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl fmt::Debug for ErrorStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_u16())
    }
}

impl FromStr for ErrorStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let res = match s {
            "Continue" => Self::Continue,
            "SwitchingProtocols" => Self::SwitchingProtocols,
            "Processing" => Self::Processing,
            "EarlyHints" => Self::EarlyHints,
            "Ok" | "Success" => Self::Ok,
            "Created" => Self::Created,
            "Accepted" => Self::Accepted,
            "NonAuthoritativeInformation" => Self::NonAuthoritativeInformation,
            "NoContent" => Self::NoContent,
            "ResetContent" => Self::ResetContent,
            "PartialContent" => Self::PartialContent,
            "MultiStatus" => Self::MultiStatus,
            "AlreadyReported" => Self::AlreadyReported,
            "ImUsed" => Self::ImUsed,
            "MultipleChoices" => Self::MultipleChoices,
            "MovedPermanently" => Self::MovedPermanently,
            "Found" => Self::Found,
            "SeeOther" => Self::SeeOther,
            "NotModified" => Self::NotModified,
            "UseProxy" => Self::UseProxy,
            "TemporaryRedirect" => Self::TemporaryRedirect,
            "PermanentRedirect" => Self::PermanentRedirect,
            "BadRequest" => Self::BadRequest,
            "Unauthorized" => Self::Unauthorized,
            "PaymentRequired" => Self::PaymentRequired,
            "Forbidden" => Self::Forbidden,
            "NotFound" => Self::NotFound,
            "MethodNotAllowed" => Self::MethodNotAllowed,
            "NotAcceptable" => Self::NotAcceptable,
            "ProxyAuthenticationRequired" => Self::ProxyAuthenticationRequired,
            "RequestTimeout" => Self::RequestTimeout,
            "Conflict" => Self::Conflict,
            "Gone" => Self::Gone,
            "LengthRequired" => Self::LengthRequired,
            "PreconditionFailed" => Self::PreconditionFailed,
            "PayloadTooLarge" => Self::PayloadTooLarge,
            "UriTooLong" => Self::UriTooLong,
            "UnsupportedMediaType" => Self::UnsupportedMediaType,
            "RangeNotSatisfiable" => Self::RangeNotSatisfiable,
            "ExpectationFailed" => Self::ExpectationFailed,
            "ImATeapot" => Self::ImATeapot,
            "MisdirectedRequest" => Self::MisdirectedRequest,
            "UnprocessableEntity" => Self::UnprocessableEntity,
            "Locked" => Self::Locked,
            "FailedDependency" => Self::FailedDependency,
            "TooEarly" => Self::TooEarly,
            "UpgradeRequired" => Self::UpgradeRequired,
            "PreconditionRequired" => Self::PreconditionRequired,
            "TooManyRequests" => Self::TooManyRequests,
            "RequestHeaderFieldsTooLarge" => Self::RequestHeaderFieldsTooLarge,
            "UnavailableForLegalReasons" => Self::UnavailableForLegalReasons,
            "InternalServerError" => Self::InternalServerError,
            "NotImplemented" => Self::NotImplemented,
            "BadGateway" => Self::BadGateway,
            "ServiceUnavailable" => Self::ServiceUnavailable,
            "GatewayTimeout" => Self::GatewayTimeout,
            "HttpVersionNotSupported" => Self::HttpVersionNotSupported,
            "VariantAlsoNegotiates" => Self::VariantAlsoNegotiates,
            "InsufficientStorage" => Self::InsufficientStorage,
            "LoopDetected" => Self::LoopDetected,
            "NotExtended" => Self::NotExtended,
            "NetworkAuthenticationRequired" => Self::NetworkAuthenticationRequired,
            _ => return Err(format!("unknown ErrorCode: {s}")),
        };
        Ok(res)
    }
}

impl From<&str> for ErrorStatus {
    fn from(s: &str) -> Self {
        s.parse().unwrap_or(Self::InternalServerError)
    }
}

impl From<u16> for ErrorStatus {
    fn from(value: u16) -> Self {
        match value {
            100 => Self::Continue,
            101 => Self::SwitchingProtocols,
            102 => Self::Processing,
            103 => Self::EarlyHints,
            200 => Self::Ok,
            201 => Self::Created,
            202 => Self::Accepted,
            203 => Self::NonAuthoritativeInformation,
            204 => Self::NoContent,
            205 => Self::ResetContent,
            206 => Self::PartialContent,
            207 => Self::MultiStatus,
            208 => Self::AlreadyReported,
            226 => Self::ImUsed,
            300 => Self::MultipleChoices,
            301 => Self::MovedPermanently,
            302 => Self::Found,
            303 => Self::SeeOther,
            304 => Self::NotModified,
            305 => Self::UseProxy,
            307 => Self::TemporaryRedirect,
            308 => Self::PermanentRedirect,
            400 => Self::BadRequest,
            401 => Self::Unauthorized,
            402 => Self::PaymentRequired,
            403 => Self::Forbidden,
            404 => Self::NotFound,
            405 => Self::MethodNotAllowed,
            406 => Self::NotAcceptable,
            407 => Self::ProxyAuthenticationRequired,
            408 => Self::RequestTimeout,
            409 => Self::Conflict,
            410 => Self::Gone,
            411 => Self::LengthRequired,
            412 => Self::PreconditionFailed,
            413 => Self::PayloadTooLarge,
            414 => Self::UriTooLong,
            415 => Self::UnsupportedMediaType,
            416 => Self::RangeNotSatisfiable,
            417 => Self::ExpectationFailed,
            418 => Self::ImATeapot,
            421 => Self::MisdirectedRequest,
            422 => Self::UnprocessableEntity,
            423 => Self::Locked,
            424 => Self::FailedDependency,
            425 => Self::TooEarly,
            426 => Self::UpgradeRequired,
            428 => Self::PreconditionRequired,
            429 => Self::TooManyRequests,
            431 => Self::RequestHeaderFieldsTooLarge,
            451 => Self::UnavailableForLegalReasons,
            // 500 => Self::InternalServerError,
            501 => Self::NotImplemented,
            502 => Self::BadGateway,
            503 => Self::ServiceUnavailable,
            504 => Self::GatewayTimeout,
            505 => Self::HttpVersionNotSupported,
            506 => Self::VariantAlsoNegotiates,
            507 => Self::InsufficientStorage,
            508 => Self::LoopDetected,
            510 => Self::NotExtended,
            511 => Self::NetworkAuthenticationRequired,
            // Fallback for unknown values
            _ => Self::InternalServerError,
        }
    }
}

impl From<i32> for ErrorStatus {
    fn from(value: i32) -> Self {
        Self::from(u16::try_from(value).unwrap_or(500))
    }
}

impl std::error::Error for ErrorStatus {}
