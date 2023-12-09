use core::fmt::Write;

use crate::body::{BODY_CHUNKED, BODY_LENGTH, BODY_NONE};
use crate::util::cast_buf_for_headers;
use crate::vars::private;
use crate::Result;
use crate::{Call, OVERFLOW};

use crate::method::*;
use crate::state::*;
use crate::version::*;
use httparse::parse_headers;
use private::*;

impl<'a> Call<'a, INIT, (), (), ()> {
    pub fn http_10(self) -> Call<'a, SEND_LINE, HTTP_10, (), ()> {
        self.transition()
    }

    pub fn http_11(self) -> Call<'a, SEND_LINE, HTTP_11, (), ()> {
        self.transition()
    }
}

macro_rules! send_method {
    ($meth:ident, $meth_up:tt, $ver:ty) => {
        pub fn $meth(mut self, path: &str) -> Result<Call<'a, SEND_HEADERS, $ver, $meth_up, ()>> {
            self.out
                .write_send_line(stringify!($meth_up), path, <$ver>::version_str())?;
            Ok(self.transition())
        }
    };
}

impl<'a> Call<'a, SEND_LINE, HTTP_10, (), ()> {
    send_method!(get, GET, HTTP_10);
    send_method!(head, HEAD, HTTP_10);
    send_method!(post, POST, HTTP_10);
}

impl<'a> Call<'a, SEND_LINE, HTTP_11, (), ()> {
    send_method!(get, GET, HTTP_11);
    send_method!(head, HEAD, HTTP_11);
    send_method!(post, POST, HTTP_11);
    send_method!(put, PUT, HTTP_11);
    send_method!(delete, DELETE, HTTP_11);
    // CONNECT
    send_method!(options, OPTIONS, HTTP_11);
    send_method!(trace, TRACE, HTTP_11);
}

impl<'a, M: Method, V: Version> Call<'a, SEND_HEADERS, V, M, ()> {
    pub fn header(self, name: &str, value: &str) -> Result<Self> {
        self.header_bytes(name, value.as_bytes())
    }

    pub fn header_bytes(mut self, name: &str, bytes: &[u8]) -> Result<Self> {
        // Attempt writing the header
        let mut w = self.out.writer();
        write!(w, "{}: ", name).or(OVERFLOW)?;
        w.write_bytes(bytes)?;
        write!(w, "\r\n").or(OVERFLOW)?;

        // Parse the written result to see if httparse would validate it.
        let (written, buf) = w.split_and_borrow();
        let headers = cast_buf_for_headers(buf)?;

        parse_headers(written, headers)?;

        // If nothing error before this, commit the result to Out.
        w.commit();

        Ok(self)
    }
}

impl<'a, M: MethodWithBody> Call<'a, SEND_HEADERS, HTTP_10, M, ()> {
    pub fn with_body(
        mut self,
        length: u64,
    ) -> Result<Call<'a, SEND_BODY, HTTP_10, M, BODY_LENGTH>> {
        let mut w = self.out.writer();
        write!(w, "Content-Length: {}\r\n\r\n", length).or(OVERFLOW)?;
        w.commit();
        Ok(self.transition())
    }

    pub fn without_body(mut self) -> Result<Call<'a, RECV_STATUS, HTTP_11, M, BODY_NONE>> {
        let mut w = self.out.writer();
        write!(w, "\r\n").or(OVERFLOW)?;
        w.commit();
        Ok(self.transition())
    }
}

impl<'a, M: MethodWithBody> Call<'a, SEND_HEADERS, HTTP_11, M, ()> {
    pub fn with_body(
        mut self,
        length: u64,
    ) -> Result<Call<'a, SEND_BODY, HTTP_11, M, BODY_LENGTH>> {
        let mut w = self.out.writer();
        write!(w, "Content-Length: {}\r\n\r\n", length).or(OVERFLOW)?;
        w.commit();
        Ok(self.transition())
    }

    pub fn with_body_chunked(mut self) -> Result<Call<'a, SEND_BODY, HTTP_11, M, BODY_CHUNKED>> {
        let mut w = self.out.writer();
        write!(w, "Transfer-Encoding: chunked\r\n\r\n").or(OVERFLOW)?;
        w.commit();
        Ok(self.transition())
    }

    pub fn without_body(mut self) -> Result<Call<'a, RECV_STATUS, HTTP_11, M, BODY_NONE>> {
        let mut w = self.out.writer();
        write!(w, "\r\n").or(OVERFLOW)?;
        w.commit();
        Ok(self.transition())
    }
}

impl<'a, V: Version, M: MethodWithoutBody> Call<'a, SEND_HEADERS, V, M, ()> {
    pub fn finish(mut self) -> Result<Call<'a, RECV_STATUS, V, M, BODY_NONE>> {
        let mut w = self.out.writer();
        write!(w, "\r\n").or(OVERFLOW)?;
        w.commit();
        Ok(self.transition())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::HootError;

    #[test]
    pub fn test_illegal_header_name() -> Result<()> {
        let mut buf = [0; 1024];

        let x = Call::new(&mut buf)
            .http_11()
            .get("/path")?
            .header(":bad:", "fine value");

        let e = x.unwrap_err();
        assert_eq!(e, HootError::ParseError(httparse::Error::HeaderName));

        Ok(())
    }

    #[test]
    pub fn test_illegal_header_value() -> Result<()> {
        let mut buf = [0; 1024];

        let x = Call::new(&mut buf)
            .http_11()
            .get("/path")?
            .header_bytes("x-broken", b"value\0xx");

        let e = x.unwrap_err();
        assert_eq!(e, HootError::ParseError(httparse::Error::HeaderValue));

        Ok(())
    }
}
