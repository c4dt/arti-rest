use std::io::Write;

use anyhow::{bail, Context, Result};
use http::{Request, Response};
use tracing::trace;

use crate::{arti::tls_send, DirectoryCache};

pub struct Client {
    dir_cache: DirectoryCache,
}

impl Client {
    pub fn new(dir_cache: DirectoryCache) -> Self {
        Self { dir_cache }
    }

    /// Sends the request to the given URL. It returns the response.
    pub fn send(&self, req: Request<Vec<u8>>) -> Result<Response<Vec<u8>>> {
        trace!("request: {:?}", req);

        let uri = req.uri().clone();
        let host = uri.host().context("no host found")?;

        let raw_req = serialize_request(req).context("serialize request")?;

        let raw_resp = tls_send(
            host,
            &String::from_utf8(raw_req).context("encode serialized as utf-8")?,
            &self.dir_cache,
        )?
        .into_bytes();

        let resp = deserialize_response(raw_resp);

        trace!("response: {:?}", resp);

        resp
    }
}

fn serialize_request(req: Request<Vec<u8>>) -> Result<Vec<u8>> {
    const EOL: &str = "\n";

    let (parts, mut body) = req.into_parts();

    let mut ret = Vec::new();

    write!(
        &mut ret,
        "{} {} {:?}{}",
        parts.method,
        parts
            .uri
            .path_and_query()
            .context("uri without path or query")?,
        parts.version,
        EOL,
    )
    .context("write status line")?;

    for (key, value) in parts.headers {
        write!(
            &mut ret,
            "{}: {}{}",
            key.context("missing header name")?,
            value.to_str().context("serialize header value as string")?,
            EOL,
        )
        .context("write header")?;
    }

    write!(&mut ret, "{}", EOL).context("write last EOL")?;

    ret.append(&mut body);

    Ok(ret)
}

fn deserialize_response(mut raw_resp: Vec<u8>) -> Result<Response<Vec<u8>>> {
    const MAX_HEADERS: usize = 16;

    let mut headers = [httparse::EMPTY_HEADER; MAX_HEADERS];

    let mut http_resp = httparse::Response::new(&mut headers);
    let parsed = http_resp.parse(raw_resp.as_slice())?;
    if parsed.is_partial() {
        bail!("unfinished response");
    }

    let mut builder = Response::builder()
        .status(http_resp.code.context("no status")?)
        .version(if http_resp.version.context("no version")? == 0 {
            http::Version::HTTP_10
        } else {
            http::Version::HTTP_11
        });
    for header in http_resp.headers {
        builder = builder.header(header.name, header.value)
    }
    builder
        .body(raw_resp.split_off(parsed.unwrap()))
        .context("create response")
}

#[cfg(test)]
mod tests {
    use tempdir::TempDir;

    use super::*;

    #[test]
    fn test_get() {
        crate::tests::setup_tracing();

        let tempdir = TempDir::new("tor-cache").expect("create temp dir");
        let resp = Client::new(DirectoryCache {
            tmp_dir: tempdir.path().to_str().map(String::from),
            nodes: None,
            relays: None,
        })
        .send(
            Request::get("https://www.c4dt.org")
                .header("Host", "www.c4dt.org")
                .body(vec![])
                .expect("create get request"),
        )
        .expect("send request");

        assert_eq!(resp.status(), 200);
    }
}
