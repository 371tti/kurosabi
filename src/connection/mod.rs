use futures::{AsyncRead, AsyncWrite};

use crate::http::{request::HttpRequest, response::HttpResponse};

pub struct Connection<C, R: AsyncRead + Unpin + 'static, W: AsyncWrite + Unpin + 'static> {
    pub c: C,
    pub req: HttpRequest<R>,
    pub res: HttpResponse<W>,
}


