use reqwest::{
    blocking::{Request, Response},
    header::{HeaderMap, HeaderValue},
};

pub fn req_to_evidence(req: &Request) -> String {
    let method = req.method();
    let url = req.url().path();
    let query = if let Some(query) = req.url().query() {
        &format!("?{query}")
    } else {
        ""
    };
    let version = req.version();
    let mut headers = req.headers().clone();
    if !headers.contains_key("accept") {
        // SAFETY: */* is always valid
        headers.append("accept", HeaderValue::from_str("*/*").unwrap());
    }
    if !headers.contains_key("host")
        && let Some(host) = req.url().host_str()
    {
        // SAFETY: host is always a valid string
        headers.append("host", HeaderValue::from_str(host).unwrap());
    }
    let headers = headers_to_evidence(&headers);
    let body = req.body();
    format!(
        "{method} {url}{query} {version:?}\r\n{headers}{}",
        if let Some(body) = body {
            // SAFETY: requests are never streamed, so this will always be present
            &format!("\r\n{}", String::from_utf8_lossy(body.as_bytes().unwrap()))
        } else {
            ""
        }
    )
}

pub fn res_to_evidence(res: Response, body: &mut String) -> String {
    let version = res.version();
    let status = res.status();
    let headers = headers_to_evidence(res.headers());
    *body = if let Ok(by) = res.bytes() {
        String::from_utf8(by.to_vec()).unwrap_or("<unable to decode response body>".to_string())
    } else {
        "<unable to decode response body>".to_string()
    };
    format!(
        "{version:?} {status}\r\n{headers}{}",
        if body.is_empty() {
            ""
        } else {
            &format!("\r\n{body}")
        }
    )
}

fn headers_to_evidence(headers: &HeaderMap) -> String {
    let mut s = String::new();
    for (key, val) in headers {
        if let Ok(val) = val.to_str() {
            s.push_str(&format!("{key}: {val}\r\n"));
        } else {
            s.push_str(&format!("{key}: <header data cannot be displayed>\r\n"));
        }
    }
    s
}
