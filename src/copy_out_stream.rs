use bytes::{Buf, Bytes};
use pin_project_lite::pin_project;
use std::{
    error::Error,
    io::Cursor,
    task::{ready, Poll},
};
use tokio_postgres::CopyOutStream;
use tokio_stream::Stream;

const MAGIC: &[u8] = b"PGCOPY\n\xff\r\n\0";

pub struct BCORow {
    pub fields: Vec<Option<Bytes>>,
}

struct Header {
    has_oids: bool,
}

pin_project! {
pub struct BCOStream {
    #[pin]
    stream: CopyOutStream,
    header: Option<Header>
}
}

impl BCOStream {
    pub fn new(stream: CopyOutStream) -> BCOStream {
        BCOStream {
            stream,
            header: None,
        }
    }
}

impl Stream for BCOStream {
    type Item = Result<BCORow, Box<dyn Error>>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.project();

        let msg = match ready!(this.stream.poll_next(cx)) {
            Some(Ok(chunk)) => chunk,
            Some(Err(e)) => return Poll::Ready(Some(Err(Box::new(e)))),
            None => return Poll::Ready(Some(Err(Box::from("stream closed")))),
        };

        let mut c = Cursor::new(&msg);

        // Parse the header if we didn't have one yet. The 1st message contains the header then the 1st message
        let has_oids = match &this.header {
            Some(header) => header.has_oids,
            None => {
                c.advance(MAGIC.len());
                let flags = c.get_i32();
                let extension = c.get_u32() as usize;
                c.advance(extension);

                let has_oids = (flags & (1 << 16)) != 0;
                *this.header = Some(Header { has_oids });
                has_oids
            }
        };

        let mut nfields = c.get_i16();
        if nfields == -1 {
            return Poll::Ready(None);
        }

        if has_oids {
            nfields += 1;
        }

        let ret = BCORow {
            fields: (0..nfields)
                .map(|_| {
                    let len = c.get_i32();
                    if len == -1 {
                        None
                    } else {
                        let data =
                            msg.slice(c.position() as usize..c.position() as usize + len as usize);
                        c.advance(len as usize);
                        Some(data)
                    }
                })
                .collect(),
        };

        Poll::Ready(Some(Ok(ret)))
    }
}
