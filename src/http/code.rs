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

pub struct HttpStatusRichInfo {
    pub code: HttpStatusCode,
    pub color: &'static str,
    pub message: &'static str,
    pub suggest: &'static [&'static str],
}

impl HttpStatusCode {
    pub fn info(&self) -> HttpStatusInfo {
        match self {
            HttpStatusCode::Continue => HttpStatusInfo { code: 100, message: "Continue" },
            HttpStatusCode::SwitchingProtocols => HttpStatusInfo { code: 101, message: "Switching Protocols" },
            HttpStatusCode::Processing => HttpStatusInfo { code: 102, message: "Processing" },
            HttpStatusCode::EarlyHints => HttpStatusInfo { code: 103, message: "Early Hints" },
            HttpStatusCode::OK => HttpStatusInfo { code: 200, message: "OK" },
            HttpStatusCode::Created => HttpStatusInfo { code: 201, message: "Created" },
            HttpStatusCode::Accepted => HttpStatusInfo { code: 202, message: "Accepted" },
            HttpStatusCode::NonAuthoritativeInformation => HttpStatusInfo { code: 203, message: "Non-Authoritative Information" },
            HttpStatusCode::NoContent => HttpStatusInfo { code: 204, message: "No Content" },
            HttpStatusCode::ResetContent => HttpStatusInfo { code: 205, message: "Reset Content" },
            HttpStatusCode::PartialContent => HttpStatusInfo { code: 206, message: "Partial Content" },
            HttpStatusCode::MultiStatus => HttpStatusInfo { code: 207, message: "Multi-Status" },
            HttpStatusCode::AlreadyReported => HttpStatusInfo { code: 208, message: "Already Reported" },
            HttpStatusCode::ThisIsFine => HttpStatusInfo { code: 218, message: "This is fine" },
            HttpStatusCode::IMUsed => HttpStatusInfo { code: 226, message: "IM Used" },
            HttpStatusCode::MultipleChoices => HttpStatusInfo { code: 300, message: "Multiple Choices" },
            HttpStatusCode::MovedPermanently => HttpStatusInfo { code: 301, message: "Moved Permanently" },
            HttpStatusCode::Found => HttpStatusInfo { code: 302, message: "Found" },
            HttpStatusCode::SeeOther => HttpStatusInfo { code: 303, message: "See Other" },
            HttpStatusCode::NotModified => HttpStatusInfo { code: 304, message: "Not Modified" },
            HttpStatusCode::UseProxy => HttpStatusInfo { code: 305, message: "Use Proxy" },
            HttpStatusCode::Unused => HttpStatusInfo { code: 306, message: "(Unused)" },
            HttpStatusCode::TemporaryRedirect => HttpStatusInfo { code: 307, message: "Temporary Redirect" },
            HttpStatusCode::PermanentRedirect => HttpStatusInfo { code: 308, message: "Permanent Redirect" },
            HttpStatusCode::BadRequest => HttpStatusInfo { code: 400, message: "BadRequest" },
            HttpStatusCode::Unauthorized => HttpStatusInfo { code: 401, message: "Unauthorized" },
            HttpStatusCode::PaymentRequired => HttpStatusInfo { code: 402, message: "Payment Required" },
            HttpStatusCode::Forbidden => HttpStatusInfo { code: 403, message: "Forbidden" },
            HttpStatusCode::NotFound => HttpStatusInfo { code: 404, message: "NotFound" },
            HttpStatusCode::MethodNotAllowed => HttpStatusInfo { code: 405, message: "MethodNotAllowed" },
            HttpStatusCode::NotAcceptable => HttpStatusInfo { code: 406, message: "NotAcceptable" },
            HttpStatusCode::ProxyAuthenticationRequired => HttpStatusInfo { code: 407, message: "ProxyAuthenticationRequired" },
            HttpStatusCode::RequestTimeout => HttpStatusInfo { code: 408, message: "RequestTimeout" },
            HttpStatusCode::Conflict => HttpStatusInfo { code: 409, message: "Conflict" },
            HttpStatusCode::Gone => HttpStatusInfo { code: 410, message: "Gone" },
            HttpStatusCode::LengthRequired => HttpStatusInfo { code: 411, message: "LengthRequired" },
            HttpStatusCode::PreconditionFailed => HttpStatusInfo { code: 412, message: "PreconditionFailed" },
            HttpStatusCode::PayloadTooLarge => HttpStatusInfo { code: 413, message: "PayloadTooLarge" },
            HttpStatusCode::URITooLong => HttpStatusInfo { code: 414, message: "URITooLong" },
            HttpStatusCode::UnsupportedMediaType => HttpStatusInfo { code: 415, message: "UnsupportedMediaType" },
            HttpStatusCode::RangeNotSatisfiable => HttpStatusInfo { code: 416, message: "RangeNotSatisfiable" },
            HttpStatusCode::ExpectationFailed => HttpStatusInfo { code: 417, message: "ExpectationFailed" },
            HttpStatusCode::ImATeapot => HttpStatusInfo { code: 418, message: "I'm a Teapot" },
            HttpStatusCode::MisdirectedRequest => HttpStatusInfo { code: 421, message: "Misdirected Request" },
            HttpStatusCode::UnprocessableEntity => HttpStatusInfo { code: 422, message: "UnprocessableEntity" },
            HttpStatusCode::Locked => HttpStatusInfo { code: 423, message: "Locked" },
            HttpStatusCode::FailedDependency => HttpStatusInfo { code: 424, message: "FailedDependency" },
            HttpStatusCode::TooEarly => HttpStatusInfo { code: 425, message: "TooEarly" },
            HttpStatusCode::UpgradeRequired => HttpStatusInfo { code: 426, message: "UpgradeRequired" },
            HttpStatusCode::PreconditionRequired => HttpStatusInfo { code: 428, message: "PreconditionRequired" },
            HttpStatusCode::TooManyRequests => HttpStatusInfo { code: 429, message: "TooManyRequests" },
            HttpStatusCode::RequestHeaderFieldsTooLarge => HttpStatusInfo { code: 431, message: "RequestHeaderFieldsTooLarge" },
            HttpStatusCode::UnavailableForLegalReasons => HttpStatusInfo { code: 451, message: "UnavailableForLegalReasons" },
            HttpStatusCode::InternalServerError => HttpStatusInfo { code: 500, message: "InternalServerError" },
            HttpStatusCode::NotImplemented => HttpStatusInfo { code: 501, message: "NotImplemented" },
            HttpStatusCode::BadGateway => HttpStatusInfo { code: 502, message: "BadGateway" },
            HttpStatusCode::ServiceUnavailable => HttpStatusInfo { code: 503, message: "ServiceUnavailable" },
            HttpStatusCode::GatewayTimeout => HttpStatusInfo { code: 504, message: "GatewayTimeout" },
            HttpStatusCode::HTTPVersionNotSupported => HttpStatusInfo { code: 505, message: "HTTPVersionNotSupported" },
            HttpStatusCode::VariantAlsoNegotiates => HttpStatusInfo { code: 506, message: "VariantAlsoNegotiates" },
            HttpStatusCode::InsufficientStorage => HttpStatusInfo { code: 507, message: "InsufficientStorage" },
            HttpStatusCode::LoopDetected => HttpStatusInfo { code: 508, message: "LoopDetected" },
            HttpStatusCode::NotExtended => HttpStatusInfo { code: 510, message: "NotExtended" },
            HttpStatusCode::NetworkAuthenticationRequired => HttpStatusInfo { code: 511, message: "NetworkAuthenticationRequired" },
            HttpStatusCode::GoodLuck => HttpStatusInfo { code: 777, message: "Good Luck" },
        }
    }

    pub fn rich_info(&self) -> HttpStatusRichInfo {
        match self {
            // 1xx
            HttpStatusCode::Continue => HttpStatusRichInfo {
                code: *self,
                color: "#0099ffff",
                message: "Continue",
                suggest: &["Continue with the request", "Wait for the final response"],
            },
            HttpStatusCode::SwitchingProtocols => HttpStatusRichInfo {
                code: *self,
                color: "#0099ffff",
                message: "Switching Protocols",
                suggest: &["Switch to the requested protocol", "No immediate action needed"],
            },
            HttpStatusCode::Processing => HttpStatusRichInfo {
                code: *self,
                color: "#0099ffff",
                message: "Processing",
                suggest: &["Wait for the server to finish processing"],
            },
            HttpStatusCode::EarlyHints => HttpStatusRichInfo {
                code: *self,
                color: "#0099ffff",
                message: "Early Hints",
                suggest: &["Preload requested resources", "Continue waiting for the final response"],
            },

            // 2xx
            HttpStatusCode::OK => HttpStatusRichInfo {
                code: *self,
                color: "#00ff00bb",
                message: "OK",
                suggest: &["No action needed", "Everything is working correctly"],
            },
            HttpStatusCode::Created => HttpStatusRichInfo {
                code: *self,
                color: "#00ff00bb",
                message: "Created",
                suggest: &["Resource has been created successfully", "No further action required"],
            },
            HttpStatusCode::Accepted => HttpStatusRichInfo {
                code: *self,
                color: "#00ff00bb",
                message: "Accepted",
                suggest: &["Request accepted, processing continues", "Check back later for completion"],
            },
            HttpStatusCode::NonAuthoritativeInformation => HttpStatusRichInfo {
                code: *self,
                color: "#00ff00bb",
                message: "Non-Authoritative Information",
                suggest: &["Data may be modified, but it’s generally fine", "No specific action needed"],
            },
            HttpStatusCode::NoContent => HttpStatusRichInfo {
                code: *self,
                color: "#00ff00bb",
                message: "No Content",
                suggest: &["No additional content", "No action needed"],
            },
            HttpStatusCode::ResetContent => HttpStatusRichInfo {
                code: *self,
                color: "#00ff00bb",
                message: "Reset Content",
                suggest: &["Reset the form or view", "Refresh the current page state"],
            },
            HttpStatusCode::PartialContent => HttpStatusRichInfo {
                code: *self,
                color: "#00ff00bb",
                message: "Partial Content",
                suggest: &["Partial response received", "Continue retrieving more data if needed"],
            },
            HttpStatusCode::MultiStatus => HttpStatusRichInfo {
                code: *self,
                color: "#00ff00bb",
                message: "Multi-Status",
                suggest: &["Multiple resources affected", "Check each resource status individually"],
            },
            HttpStatusCode::AlreadyReported => HttpStatusRichInfo {
                code: *self,
                color: "#00ff00bb",
                message: "Already Reported",
                suggest: &["Already processed resource", "No further action needed"],
            },
            HttpStatusCode::ThisIsFine => HttpStatusRichInfo {
                code: *self,
                color: "#00ff00bb",
                message: "This is fine",
                suggest: &["Everything is okay", "No action needed"],
            },
            HttpStatusCode::IMUsed => HttpStatusRichInfo {
                code: *self,
                color: "#00ff00bb",
                message: "IM Used",
                suggest: &["Instance manipulations applied", "No further action required"],
            },

            // 3xx
            HttpStatusCode::MultipleChoices => HttpStatusRichInfo {
                code: *self,
                color: "#ffff00bb",
                message: "Multiple Choices",
                suggest: &["Select one of the provided options", "Update bookmarks if needed"],
            },
            HttpStatusCode::MovedPermanently => HttpStatusRichInfo {
                code: *self,
                color: "#ffff00bb",
                message: "Moved Permanently",
                suggest: &["Update bookmarks to the new URL", "Follow the redirect"],
            },
            HttpStatusCode::Found => HttpStatusRichInfo {
                code: *self,
                color: "#ffff00bb",
                message: "Found",
                suggest: &["Resource temporarily at another URL", "Follow the new location"],
            },
            HttpStatusCode::SeeOther => HttpStatusRichInfo {
                code: *self,
                color: "#ffff00bb",
                message: "See Other",
                suggest: &["Check the 'Location' header", "Use GET method on the redirected URL"],
            },
            HttpStatusCode::NotModified => HttpStatusRichInfo {
                code: *self,
                color: "#ffff00bb",
                message: "Not Modified",
                suggest: &["Use cached version of the resource", "No action needed"],
            },
            HttpStatusCode::UseProxy => HttpStatusRichInfo {
                code: *self,
                color: "#ffff00bb",
                message: "Use Proxy",
                suggest: &["Access the resource through the given proxy", "Update network settings if necessary"],
            },
            HttpStatusCode::Unused => HttpStatusRichInfo {
                code: *self,
                color: "#ffff00bb",
                message: "(Unused)",
                suggest: &["No action needed (Code is no longer used)"],
            },
            HttpStatusCode::TemporaryRedirect => HttpStatusRichInfo {
                code: *self,
                color: "#ffff00bb",
                message: "Temporary Redirect",
                suggest: &["Use the given URL temporarily", "Request method should not change"],
            },
            HttpStatusCode::PermanentRedirect => HttpStatusRichInfo {
                code: *self,
                color: "#ffff00bb",
                message: "Permanent Redirect",
                suggest: &["Update bookmarks to new URL", "Request method is preserved"],
            },

            // 4xx
            HttpStatusCode::BadRequest => HttpStatusRichInfo {
                code: *self,
                color: "#ff9900ff",
                message: "BadRequest",
                suggest: &[
                    "Check the request syntax",
                    "Verify the request parameters",
                    "Ensure the URL is correct",
                ],
            },
            HttpStatusCode::Unauthorized => HttpStatusRichInfo {
                code: *self,
                color: "#ff9900ff",
                message: "Unauthorized",
                suggest: &[
                    "Check the authentication credentials",
                    "Login again",
                    "Contact the website administrator",
                ],
            },
            HttpStatusCode::PaymentRequired => HttpStatusRichInfo {
                code: *self,
                color: "#ff9900ff",
                message: "Payment Required",
                suggest: &[
                    "Check if payment is required",
                    "Provide valid payment details",
                ],
            },
            HttpStatusCode::Forbidden => HttpStatusRichInfo {
                code: *self,
                color: "#ff9900ff",
                message: "Forbidden",
                suggest: &[
                    "Check the URL for errors",
                    "Request access from the administrator",
                    "Ensure you have the necessary permissions",
                ],
            },
            HttpStatusCode::NotFound => HttpStatusRichInfo {
                code: *self,
                color: "#ff9900ff",
                message: "NotFound",
                suggest: &[
                    "Check the URL",
                    "Reload the page",
                    "Clear the browser cache",
                    "Try using another browser",
                    "Contact customer support",
                ],
            },
            HttpStatusCode::MethodNotAllowed => HttpStatusRichInfo {
                code: *self,
                color: "#ff9900ff",
                message: "MethodNotAllowed",
                suggest: &[
                    "Check the request method (GET, POST, etc.)",
                    "Refer to the website's API documentation",
                    "Ensure the method is supported",
                ],
            },
            HttpStatusCode::NotAcceptable => HttpStatusRichInfo {
                code: *self,
                color: "#ff9900ff",
                message: "NotAcceptable",
                suggest: &[
                    "Check the requested media type",
                    "Ensure server supports the requested format",
                ],
            },
            HttpStatusCode::ProxyAuthenticationRequired => HttpStatusRichInfo {
                code: *self,
                color: "#ff9900ff",
                message: "ProxyAuthenticationRequired",
                suggest: &[
                    "Verify proxy authentication",
                    "Contact network administrator for proxy details",
                ],
            },
            HttpStatusCode::RequestTimeout => HttpStatusRichInfo {
                code: *self,
                color: "#ff9900ff",
                message: "RequestTimeout",
                suggest: &[
                    "Check your internet connection",
                    "Ensure the server is not overloaded",
                    "Retry the request after a moment",
                ],
            },
            HttpStatusCode::Conflict => HttpStatusRichInfo {
                code: *self,
                color: "#ff9900ff",
                message: "Conflict",
                suggest: &[
                    "Resolve conflicting resources",
                    "Ensure request data is consistent",
                ],
            },
            HttpStatusCode::Gone => HttpStatusRichInfo {
                code: *self,
                color: "#ff9900ff",
                message: "Gone",
                suggest: &[
                    "This resource is no longer available",
                    "Contact the website administrator for information",
                ],
            },
            HttpStatusCode::LengthRequired => HttpStatusRichInfo {
                code: *self,
                color: "#ff9900ff",
                message: "LengthRequired",
                suggest: &["Set 'Content-Length' header in request"],
            },
            HttpStatusCode::PreconditionFailed => HttpStatusRichInfo {
                code: *self,
                color: "#ff9900ff",
                message: "PreconditionFailed",
                suggest: &[
                    "Verify request preconditions",
                    "Adjust precondition headers",
                ],
            },
            HttpStatusCode::PayloadTooLarge => HttpStatusRichInfo {
                code: *self,
                color: "#ff9900ff",
                message: "PayloadTooLarge",
                suggest: &[
                    "Reduce the request entity size",
                    "Contact administrator for size limits",
                ],
            },
            HttpStatusCode::URITooLong => HttpStatusRichInfo {
                code: *self,
                color: "#ff9900ff",
                message: "URITooLong",
                suggest: &[
                    "Simplify the URL length",
                    "Use a shorter URL structure",
                ],
            },
            HttpStatusCode::UnsupportedMediaType => HttpStatusRichInfo {
                code: *self,
                color: "#ff9900ff",
                message: "UnsupportedMediaType",
                suggest: &[
                    "Check the media type in request",
                    "Ensure server supports media type",
                ],
            },
            HttpStatusCode::RangeNotSatisfiable => HttpStatusRichInfo {
                code: *self,
                color: "#ff9900ff",
                message: "RangeNotSatisfiable",
                suggest: &["Check requested range headers"],
            },
            HttpStatusCode::ExpectationFailed => HttpStatusRichInfo {
                code: *self,
                color: "#ff9900ff",
                message: "ExpectationFailed",
                suggest: &["Check 'Expect' request header"],
            },
            HttpStatusCode::ImATeapot => HttpStatusRichInfo {
                code: *self,
                color: "#ff9900ff",
                message: "I'm a Teapot",
                suggest: &["I'm a teapot, not a coffee machine"],
            },
            HttpStatusCode::MisdirectedRequest => HttpStatusRichInfo {
                code: *self,
                color: "#ff9900ff",
                message: "Misdirected Request",
                suggest: &[
                    "Check the request host",
                    "Ensure request matches the server's authority",
                ],
            },
            HttpStatusCode::UnprocessableEntity => HttpStatusRichInfo {
                code: *self,
                color: "#ff9900ff",
                message: "UnprocessableEntity",
                suggest: &["Check request syntax and data"],
            },
            HttpStatusCode::Locked => HttpStatusRichInfo {
                code: *self,
                color: "#ff9900ff",
                message: "Locked",
                suggest: &[
                    "Resource is locked",
                    "Try unlocking the resource if you have permissions",
                ],
            },
            HttpStatusCode::FailedDependency => HttpStatusRichInfo {
                code: *self,
                color: "#ff9900ff",
                message: "FailedDependency",
                suggest: &[
                    "Check dependent requests",
                    "Resolve the dependency failures",
                ],
            },
            HttpStatusCode::TooEarly => HttpStatusRichInfo {
                code: *self,
                color: "#ff9900ff",
                message: "TooEarly",
                suggest: &[
                    "Wait before sending early data",
                    "Try again later",
                ],
            },
            HttpStatusCode::UpgradeRequired => HttpStatusRichInfo {
                code: *self,
                color: "#ff9900ff",
                message: "UpgradeRequired",
                suggest: &[
                    "Switch to a different protocol",
                    "Check 'Upgrade' header",
                ],
            },
            HttpStatusCode::PreconditionRequired => HttpStatusRichInfo {
                code: *self,
                color: "#ff9900ff",
                message: "PreconditionRequired",
                suggest: &[
                    "Add required precondition headers",
                    "Check documentation for required conditions",
                ],
            },
            HttpStatusCode::TooManyRequests => HttpStatusRichInfo {
                code: *self,
                color: "#ff9900ff",
                message: "TooManyRequests",
                suggest: &[
                    "Reduce the frequency of requests",
                    "Wait before sending more requests",
                ],
            },
            HttpStatusCode::RequestHeaderFieldsTooLarge => HttpStatusRichInfo {
                code: *self,
                color: "#ff9900ff",
                message: "RequestHeaderFieldsTooLarge",
                suggest: &["Reduce header data size"],
            },
            HttpStatusCode::UnavailableForLegalReasons => HttpStatusRichInfo {
                code: *self,
                color: "#ff9900ff",
                message: "UnavailableForLegalReasons",
                suggest: &[
                    "Content blocked due to legal reasons",
                    "Contact the website administrator",
                ],
            },

            // 5xx
            HttpStatusCode::InternalServerError => HttpStatusRichInfo {
                code: *self,
                color: "#ff0000bb",
                message: "InternalServerError",
                suggest: &[
                    "Wait a few moments and retry the request",
                    "Check the website's social media for updates",
                    "Contact customer support",
                ],
            },
            HttpStatusCode::NotImplemented => HttpStatusRichInfo {
                code: *self,
                color: "#ff0000bb",
                message: "NotImplemented",
                suggest: &[
                    "Verify the request method is correct",
                    "Check if the feature is implemented",
                    "Contact the website administrator",
                ],
            },
            HttpStatusCode::BadGateway => HttpStatusRichInfo {
                code: *self,
                color: "#ff0000bb",
                message: "BadGateway",
                suggest: &[
                    "Check your internet connection",
                    "Wait a few moments and retry the request",
                    "Contact the website if the issue persists",
                ],
            },
            HttpStatusCode::ServiceUnavailable => HttpStatusRichInfo {
                code: *self,
                color: "#ff0000bb",
                message: "ServiceUnavailable",
                suggest: &[
                    "Check if the website is under maintenance",
                    "Wait and retry later",
                    "Contact the website for more information",
                ],
            },
            HttpStatusCode::GatewayTimeout => HttpStatusRichInfo {
                code: *self,
                color: "#ff0000bb",
                message: "GatewayTimeout",
                suggest: &[
                    "Check your internet connection",
                    "Ensure the server is reachable",
                    "Retry the request after a moment",
                ],
            },
            HttpStatusCode::HTTPVersionNotSupported => HttpStatusRichInfo {
                code: *self,
                color: "#ff0000bb",
                message: "HTTPVersionNotSupported",
                suggest: &[
                    "Verify the HTTP version used",
                    "Contact administrator to check version support",
                ],
            },
            HttpStatusCode::VariantAlsoNegotiates => HttpStatusRichInfo {
                code: *self,
                color: "#ff0000bb",
                message: "VariantAlsoNegotiates",
                suggest: &[
                    "Check negotiation headers",
                    "Use a simpler request",
                ],
            },
            HttpStatusCode::InsufficientStorage => HttpStatusRichInfo {
                code: *self,
                color: "#ff0000bb",
                message: "InsufficientStorage",
                suggest: &[
                    "Reduce the size of the resource",
                    "Contact administrator about storage limits",
                ],
            },
            HttpStatusCode::LoopDetected => HttpStatusRichInfo {
                code: *self,
                color: "#ff0000bb",
                message: "LoopDetected",
                suggest: &[
                    "Check for infinite loops in request",
                    "Consult the server administrator",
                ],
            },
            HttpStatusCode::NotExtended => HttpStatusRichInfo {
                code: *self,
                color: "#ff0000bb",
                message: "NotExtended",
                suggest: &[
                    "Check for required extensions",
                    "Add missing extension headers",
                ],
            },
            HttpStatusCode::NetworkAuthenticationRequired => HttpStatusRichInfo {
                code: *self,
                color: "#ff0000bb",
                message: "NetworkAuthenticationRequired",
                suggest: &[
                    "Authenticate to access network",
                    "Check network credentials",
                ],
            },

            // Custom
            HttpStatusCode::GoodLuck => HttpStatusRichInfo {
                code: *self,
                color: "#ff00ffff",
                message: "Good Luck",
                suggest: &[
                    "Everything is up to you",
                    "The Lucky Number",
                ],
            },
        }
    }
}
