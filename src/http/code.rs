#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum HttpStatusCode {
    // 1xx Informational
    Continue = 100,
    SwitchingProtocols = 101,
    Processing = 102,
    EarlyHints = 103,

    // 2xx Success
    OK = 200,
    Created = 201,
    Accepted = 202,
    NonAuthoritativeInformation = 203,
    NoContent = 204,
    ResetContent = 205,
    PartialContent = 206,
    MultiStatus = 207,
    AlreadyReported = 208,
    ThisIsFine = 218,
    IMUsed = 226,

    // 3xx Redirection
    MultipleChoices = 300,
    MovedPermanently = 301,
    Found = 302,
    SeeOther = 303,
    NotModified = 304,
    UseProxy = 305,
    Unused = 306,
    TemporaryRedirect = 307,
    PermanentRedirect = 308,

    // 4xx Client Error
    BadRequest = 400,
    Unauthorized = 401,
    PaymentRequired = 402,
    Forbidden = 403,
    NotFound = 404,
    MethodNotAllowed = 405,
    NotAcceptable = 406,
    ProxyAuthenticationRequired = 407,
    RequestTimeout = 408,
    Conflict = 409,
    Gone = 410,
    LengthRequired = 411,
    PreconditionFailed = 412,
    PayloadTooLarge = 413,
    URITooLong = 414,
    UnsupportedMediaType = 415,
    RangeNotSatisfiable = 416,
    ExpectationFailed = 417,
    ImATeapot = 418,
    MisdirectedRequest = 421,
    UnprocessableEntity = 422,
    Locked = 423,
    FailedDependency = 424,
    TooEarly = 425,
    UpgradeRequired = 426,
    PreconditionRequired = 428,
    TooManyRequests = 429,
    RequestHeaderFieldsTooLarge = 431,
    UnavailableForLegalReasons = 451,

    // 5xx Server Error
    InternalServerError = 500,
    NotImplemented = 501,
    BadGateway = 502,
    ServiceUnavailable = 503,
    GatewayTimeout = 504,
    HTTPVersionNotSupported = 505,
    VariantAlsoNegotiates = 506,
    InsufficientStorage = 507,
    LoopDetected = 508,
    NotExtended = 510,
    NetworkAuthenticationRequired = 511,

    // Custom
    GoodLuck = 777,
}

pub struct HttpStatusInfo {
    pub code: u16,
    pub message: &'static str,
}

impl HttpStatusCode {
    pub fn as_bytes(&self) -> &[u8; 3] {
        match self {
            HttpStatusCode::Continue => b"100",
            HttpStatusCode::SwitchingProtocols => b"101",
            HttpStatusCode::Processing => b"102",
            HttpStatusCode::EarlyHints => b"103",
            HttpStatusCode::OK => b"200",
            HttpStatusCode::Created => b"201",
            HttpStatusCode::Accepted => b"202",
            HttpStatusCode::NonAuthoritativeInformation => b"203",
            HttpStatusCode::NoContent => b"204",
            HttpStatusCode::ResetContent => b"205",
            HttpStatusCode::PartialContent => b"206",
            HttpStatusCode::MultiStatus => b"207",
            HttpStatusCode::AlreadyReported => b"208",
            HttpStatusCode::ThisIsFine => b"218",
            HttpStatusCode::IMUsed => b"226",
            HttpStatusCode::MultipleChoices => b"300",
            HttpStatusCode::MovedPermanently => b"301",
            HttpStatusCode::Found => b"302",
            HttpStatusCode::SeeOther => b"303",
            HttpStatusCode::NotModified => b"304",
            HttpStatusCode::UseProxy => b"305",
            HttpStatusCode::Unused => b"306",
            HttpStatusCode::TemporaryRedirect => b"307",
            HttpStatusCode::PermanentRedirect => b"308",
            HttpStatusCode::BadRequest => b"400",
            HttpStatusCode::Unauthorized => b"401",
            HttpStatusCode::PaymentRequired => b"402",
            HttpStatusCode::Forbidden => b"403",
            HttpStatusCode::NotFound => b"404",
            HttpStatusCode::MethodNotAllowed => b"405",
            HttpStatusCode::NotAcceptable => b"406",
            HttpStatusCode::ProxyAuthenticationRequired => b"407",
            HttpStatusCode::RequestTimeout => b"408",
            HttpStatusCode::Conflict => b"409",
            HttpStatusCode::Gone => b"410",
            HttpStatusCode::LengthRequired => b"411",
            HttpStatusCode::PreconditionFailed => b"412",
            HttpStatusCode::PayloadTooLarge => b"413",
            HttpStatusCode::URITooLong => b"414",
            HttpStatusCode::UnsupportedMediaType => b"415",
            HttpStatusCode::RangeNotSatisfiable => b"416",
            HttpStatusCode::ExpectationFailed => b"417",
            HttpStatusCode::ImATeapot => b"418",
            HttpStatusCode::MisdirectedRequest => b"421",
            HttpStatusCode::UnprocessableEntity => b"422",
            HttpStatusCode::Locked => b"423",
            HttpStatusCode::FailedDependency => b"424",
            HttpStatusCode::TooEarly => b"425",
            HttpStatusCode::UpgradeRequired => b"426",
            HttpStatusCode::PreconditionRequired => b"428",
            HttpStatusCode::TooManyRequests => b"429",
            HttpStatusCode::RequestHeaderFieldsTooLarge => b"431",
            HttpStatusCode::UnavailableForLegalReasons => b"451",
            HttpStatusCode::InternalServerError => b"500",
            HttpStatusCode::NotImplemented => b"501",
            HttpStatusCode::BadGateway => b"502",
            HttpStatusCode::ServiceUnavailable => b"503",
            HttpStatusCode::GatewayTimeout => b"504",
            HttpStatusCode::HTTPVersionNotSupported => b"505",
            HttpStatusCode::VariantAlsoNegotiates => b"506",
            HttpStatusCode::InsufficientStorage => b"507",
            HttpStatusCode::LoopDetected => b"508",
            HttpStatusCode::NotExtended => b"510",
            HttpStatusCode::NetworkAuthenticationRequired => b"511",
            HttpStatusCode::GoodLuck => b"777",
        }
    }

    pub fn info(&self) -> HttpStatusInfo {
        match self {
            HttpStatusCode::Continue => HttpStatusInfo {
                code: 100,
                message: "Continue",
            },
            HttpStatusCode::SwitchingProtocols => HttpStatusInfo {
                code: 101,
                message: "Switching Protocols",
            },
            HttpStatusCode::Processing => HttpStatusInfo {
                code: 102,
                message: "Processing",
            },
            HttpStatusCode::EarlyHints => HttpStatusInfo {
                code: 103,
                message: "Early Hints",
            },
            HttpStatusCode::OK => HttpStatusInfo {
                code: 200,
                message: "OK",
            },
            HttpStatusCode::Created => HttpStatusInfo {
                code: 201,
                message: "Created",
            },
            HttpStatusCode::Accepted => HttpStatusInfo {
                code: 202,
                message: "Accepted",
            },
            HttpStatusCode::NonAuthoritativeInformation => HttpStatusInfo {
                code: 203,
                message: "Non-Authoritative Information",
            },
            HttpStatusCode::NoContent => HttpStatusInfo {
                code: 204,
                message: "No Content",
            },
            HttpStatusCode::ResetContent => HttpStatusInfo {
                code: 205,
                message: "Reset Content",
            },
            HttpStatusCode::PartialContent => HttpStatusInfo {
                code: 206,
                message: "Partial Content",
            },
            HttpStatusCode::MultiStatus => HttpStatusInfo {
                code: 207,
                message: "Multi-Status",
            },
            HttpStatusCode::AlreadyReported => HttpStatusInfo {
                code: 208,
                message: "Already Reported",
            },
            HttpStatusCode::ThisIsFine => HttpStatusInfo {
                code: 218,
                message: "This is fine",
            },
            HttpStatusCode::IMUsed => HttpStatusInfo {
                code: 226,
                message: "IM Used",
            },
            HttpStatusCode::MultipleChoices => HttpStatusInfo {
                code: 300,
                message: "Multiple Choices",
            },
            HttpStatusCode::MovedPermanently => HttpStatusInfo {
                code: 301,
                message: "Moved Permanently",
            },
            HttpStatusCode::Found => HttpStatusInfo {
                code: 302,
                message: "Found",
            },
            HttpStatusCode::SeeOther => HttpStatusInfo {
                code: 303,
                message: "See Other",
            },
            HttpStatusCode::NotModified => HttpStatusInfo {
                code: 304,
                message: "Not Modified",
            },
            HttpStatusCode::UseProxy => HttpStatusInfo {
                code: 305,
                message: "Use Proxy",
            },
            HttpStatusCode::Unused => HttpStatusInfo {
                code: 306,
                message: "(Unused)",
            },
            HttpStatusCode::TemporaryRedirect => HttpStatusInfo {
                code: 307,
                message: "Temporary Redirect",
            },
            HttpStatusCode::PermanentRedirect => HttpStatusInfo {
                code: 308,
                message: "Permanent Redirect",
            },
            HttpStatusCode::BadRequest => HttpStatusInfo {
                code: 400,
                message: "BadRequest",
            },
            HttpStatusCode::Unauthorized => HttpStatusInfo {
                code: 401,
                message: "Unauthorized",
            },
            HttpStatusCode::PaymentRequired => HttpStatusInfo {
                code: 402,
                message: "Payment Required",
            },
            HttpStatusCode::Forbidden => HttpStatusInfo {
                code: 403,
                message: "Forbidden",
            },
            HttpStatusCode::NotFound => HttpStatusInfo {
                code: 404,
                message: "NotFound",
            },
            HttpStatusCode::MethodNotAllowed => HttpStatusInfo {
                code: 405,
                message: "MethodNotAllowed",
            },
            HttpStatusCode::NotAcceptable => HttpStatusInfo {
                code: 406,
                message: "NotAcceptable",
            },
            HttpStatusCode::ProxyAuthenticationRequired => HttpStatusInfo {
                code: 407,
                message: "ProxyAuthenticationRequired",
            },
            HttpStatusCode::RequestTimeout => HttpStatusInfo {
                code: 408,
                message: "RequestTimeout",
            },
            HttpStatusCode::Conflict => HttpStatusInfo {
                code: 409,
                message: "Conflict",
            },
            HttpStatusCode::Gone => HttpStatusInfo {
                code: 410,
                message: "Gone",
            },
            HttpStatusCode::LengthRequired => HttpStatusInfo {
                code: 411,
                message: "LengthRequired",
            },
            HttpStatusCode::PreconditionFailed => HttpStatusInfo {
                code: 412,
                message: "PreconditionFailed",
            },
            HttpStatusCode::PayloadTooLarge => HttpStatusInfo {
                code: 413,
                message: "PayloadTooLarge",
            },
            HttpStatusCode::URITooLong => HttpStatusInfo {
                code: 414,
                message: "URITooLong",
            },
            HttpStatusCode::UnsupportedMediaType => HttpStatusInfo {
                code: 415,
                message: "UnsupportedMediaType",
            },
            HttpStatusCode::RangeNotSatisfiable => HttpStatusInfo {
                code: 416,
                message: "RangeNotSatisfiable",
            },
            HttpStatusCode::ExpectationFailed => HttpStatusInfo {
                code: 417,
                message: "ExpectationFailed",
            },
            HttpStatusCode::ImATeapot => HttpStatusInfo {
                code: 418,
                message: "I'm a Teapot",
            },
            HttpStatusCode::MisdirectedRequest => HttpStatusInfo {
                code: 421,
                message: "Misdirected Request",
            },
            HttpStatusCode::UnprocessableEntity => HttpStatusInfo {
                code: 422,
                message: "UnprocessableEntity",
            },
            HttpStatusCode::Locked => HttpStatusInfo {
                code: 423,
                message: "Locked",
            },
            HttpStatusCode::FailedDependency => HttpStatusInfo {
                code: 424,
                message: "FailedDependency",
            },
            HttpStatusCode::TooEarly => HttpStatusInfo {
                code: 425,
                message: "TooEarly",
            },
            HttpStatusCode::UpgradeRequired => HttpStatusInfo {
                code: 426,
                message: "UpgradeRequired",
            },
            HttpStatusCode::PreconditionRequired => HttpStatusInfo {
                code: 428,
                message: "PreconditionRequired",
            },
            HttpStatusCode::TooManyRequests => HttpStatusInfo {
                code: 429,
                message: "TooManyRequests",
            },
            HttpStatusCode::RequestHeaderFieldsTooLarge => HttpStatusInfo {
                code: 431,
                message: "RequestHeaderFieldsTooLarge",
            },
            HttpStatusCode::UnavailableForLegalReasons => HttpStatusInfo {
                code: 451,
                message: "UnavailableForLegalReasons",
            },
            HttpStatusCode::InternalServerError => HttpStatusInfo {
                code: 500,
                message: "InternalServerError",
            },
            HttpStatusCode::NotImplemented => HttpStatusInfo {
                code: 501,
                message: "NotImplemented",
            },
            HttpStatusCode::BadGateway => HttpStatusInfo {
                code: 502,
                message: "BadGateway",
            },
            HttpStatusCode::ServiceUnavailable => HttpStatusInfo {
                code: 503,
                message: "ServiceUnavailable",
            },
            HttpStatusCode::GatewayTimeout => HttpStatusInfo {
                code: 504,
                message: "GatewayTimeout",
            },
            HttpStatusCode::HTTPVersionNotSupported => HttpStatusInfo {
                code: 505,
                message: "HTTPVersionNotSupported",
            },
            HttpStatusCode::VariantAlsoNegotiates => HttpStatusInfo {
                code: 506,
                message: "VariantAlsoNegotiates",
            },
            HttpStatusCode::InsufficientStorage => HttpStatusInfo {
                code: 507,
                message: "InsufficientStorage",
            },
            HttpStatusCode::LoopDetected => HttpStatusInfo {
                code: 508,
                message: "LoopDetected",
            },
            HttpStatusCode::NotExtended => HttpStatusInfo {
                code: 510,
                message: "NotExtended",
            },
            HttpStatusCode::NetworkAuthenticationRequired => HttpStatusInfo {
                code: 511,
                message: "NetworkAuthenticationRequired",
            },
            HttpStatusCode::GoodLuck => HttpStatusInfo {
                code: 777,
                message: "Good Luck",
            },
        }
    }
}

impl Into<u16> for HttpStatusCode {
    fn into(self) -> u16 {
        self as u16
    }
}

impl From<u16> for HttpStatusCode {
    fn from(code: u16) -> Self {
        match code {
            100 => HttpStatusCode::Continue,
            101 => HttpStatusCode::SwitchingProtocols,
            102 => HttpStatusCode::Processing,
            103 => HttpStatusCode::EarlyHints,
            200 => HttpStatusCode::OK,
            201 => HttpStatusCode::Created,
            202 => HttpStatusCode::Accepted,
            203 => HttpStatusCode::NonAuthoritativeInformation,
            204 => HttpStatusCode::NoContent,
            205 => HttpStatusCode::ResetContent,
            206 => HttpStatusCode::PartialContent,
            207 => HttpStatusCode::MultiStatus,
            208 => HttpStatusCode::AlreadyReported,
            218 => HttpStatusCode::ThisIsFine,
            226 => HttpStatusCode::IMUsed,
            300 => HttpStatusCode::MultipleChoices,
            301 => HttpStatusCode::MovedPermanently,
            302 => HttpStatusCode::Found,
            303 => HttpStatusCode::SeeOther,
            304 => HttpStatusCode::NotModified,
            305 => HttpStatusCode::UseProxy,
            306 => HttpStatusCode::Unused,
            307 => HttpStatusCode::TemporaryRedirect,
            308 => HttpStatusCode::PermanentRedirect,
            400 => HttpStatusCode::BadRequest,
            401 => HttpStatusCode::Unauthorized,
            402 => HttpStatusCode::PaymentRequired,
            403 => HttpStatusCode::Forbidden,
            404 => HttpStatusCode::NotFound,
            405 => HttpStatusCode::MethodNotAllowed,
            406 => HttpStatusCode::NotAcceptable,
            407 => HttpStatusCode::ProxyAuthenticationRequired,
            408 => HttpStatusCode::RequestTimeout,
            409 => HttpStatusCode::Conflict,
            410 => HttpStatusCode::Gone,
            411 => HttpStatusCode::LengthRequired,
            412 => HttpStatusCode::PreconditionFailed,
            413 => HttpStatusCode::PayloadTooLarge,
            414 => HttpStatusCode::URITooLong,
            415 => HttpStatusCode::UnsupportedMediaType,
            416 => HttpStatusCode::RangeNotSatisfiable,
            417 => HttpStatusCode::ExpectationFailed,
            418 => HttpStatusCode::ImATeapot,
            421 => HttpStatusCode::MisdirectedRequest,
            422 => HttpStatusCode::UnprocessableEntity,
            423 => HttpStatusCode::Locked,
            424 => HttpStatusCode::FailedDependency,
            425 => HttpStatusCode::TooEarly,
            426 => HttpStatusCode::UpgradeRequired,
            428 => HttpStatusCode::PreconditionRequired,
            429 => HttpStatusCode::TooManyRequests,
            431 => HttpStatusCode::RequestHeaderFieldsTooLarge,
            451 => HttpStatusCode::UnavailableForLegalReasons,
            500 => HttpStatusCode::InternalServerError,
            501 => HttpStatusCode::NotImplemented,
            502 => HttpStatusCode::BadGateway,
            503 => HttpStatusCode::ServiceUnavailable,
            504 => HttpStatusCode::GatewayTimeout,
            505 => HttpStatusCode::HTTPVersionNotSupported,
            506 => HttpStatusCode::VariantAlsoNegotiates,
            507 => HttpStatusCode::InsufficientStorage,
            508 => HttpStatusCode::LoopDetected,
            510 => HttpStatusCode::NotExtended,
            511 => HttpStatusCode::NetworkAuthenticationRequired,
            777 => HttpStatusCode::GoodLuck,
            _ => HttpStatusCode::InternalServerError,
        }
    }
}
