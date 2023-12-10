use core::str::Utf8Error;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HootError {
    /// The borrowed buffer did not have enough space to hold the
    /// data we attempted to write.
    ///
    /// Call `.flush()`, write the output to the transport followed by `Call::resume()`.
    OutputOverflow,

    /// Invalid byte in header name.
    HeaderName,

    /// Invalid byte in header value.
    HeaderValue,

    /// Invalid Response status.
    Status,

    /// Invalid byte in new line.
    NewLine,

    /// Parsed more headers than provided buffer can contain.
    TooManyHeaders,

    /// Parsing headers (for sending or receiving) uses leftover space in the
    /// buffer. This error means there was not enough "spare" space to parse
    /// any headers.
    ///
    /// Call `.flush()`, write the output to the transport followed by `Call::resume()`.
    InsufficientSpaceToParseHeaders,

    /// Encountered a forbidden header name.
    ///
    /// `content-length` and `transfer-encoding` must be set using
    /// `with_body()` and `with_body_chunked()`.
    ForbiddenBodyHeader,

    /// Header is not allowed for HTTP/1.1
    ForbiddenHttp11Header,

    /// The trailer name is not allowed.
    ForbiddenTrailer,

    /// Attempt to send more content than declared in the `Content-Length` header.
    SentMoreThanContentLength,

    /// Attempt to send less content than declared in the `Content-Length` header.
    SentLessThanContentLength,

    /// Failed to read bytes as &str
    ConvertBytesToStr,

    /// The requested HTTP version does not match the response HTTP version.
    HttpVersionMismatch,

    /// If we attempt to call `.complete()` on an AttemptStatus that didn't get full input to succeed.
    StatusIsNotComplete,
}

pub(crate) static OVERFLOW: Result<()> = Err(HootError::OutputOverflow);

pub type Result<T> = core::result::Result<T, HootError>;

impl From<Utf8Error> for HootError {
    fn from(_: Utf8Error) -> Self {
        HootError::ConvertBytesToStr
    }
}