use std::{collections::HashMap, str::FromStr};

use hyper::{
    body,
    client::HttpConnector,
    header::{HeaderName, HeaderValue},
    http::{uri::PathAndQuery, Result as HttpResult},
    Body, Client as HyperClient, HeaderMap, Request, Response, Result as HyperResult, Uri,
};
use iroha_client::http::{Method, RequestBuilder};

pub struct AsyncRequestBuilder {
    method: Method,
    url: String,
    headers: HttpResult<HeaderMap>,
    params: HashMap<String, String>,
    body: Option<Vec<u8>>,
}

impl AsyncRequestBuilder {
    pub fn build(mut self) -> HttpResult<AsyncRequest> {
        let mut uri: Uri = self.url.try_into()?;
        let params = self
            .params
            .into_iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .fold(Option::<String>::None, |acc, pr| match acc {
                Some(mut s) => {
                    s.push('&');
                    s.push_str(&pr[..]);
                    Some(s)
                }
                None => Some(pr),
            });
        if let Some(params) = params {
            let mut parts = uri.into_parts();
            let p_and_q: PathAndQuery = match &parts.path_and_query {
                Some(p_and_q) => {
                    let path = p_and_q.path();
                    let mut s = String::new();
                    s.push_str(path);
                    s.push('?');
                    s.push_str(params.as_str());
                    s.try_into()?
                }
                None => {
                    let mut s = String::new();
                    s.push_str(r"/?");
                    s.push_str(params.as_str());
                    s.try_into()?
                }
            };
            parts.path_and_query.replace(p_and_q);
            uri = Uri::from_parts(parts)?;
        }
        let builder = Request::builder().method(self.method).uri(uri);
        let mut req = match self.body.take() {
            Some(bytes) => builder.body(Body::from(bytes)),
            None => builder.body(Body::default()),
        }?;
        let headers = self.headers?;
        let req_headers = req.headers_mut();
        for (k, v) in headers {
            if let Some(name) = k {
                req_headers.insert(name, v);
            }
        }
        Ok(AsyncRequest(req))
    }
}

impl RequestBuilder for AsyncRequestBuilder {
    fn new(method: Method, url: url::Url) -> Self {
        Self {
            method,
            url: url.as_ref().to_string(),
            headers: Ok(HeaderMap::new()),
            params: HashMap::new(),
            body: None,
        }
    }

    fn param<K, V>(mut self, key: K, value: V) -> Self
    where
        K: AsRef<str>,
        V: ToString,
    {
        self.params
            .insert(key.as_ref().to_string(), value.to_string());
        self
    }

    fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: AsRef<str>,
        V: ToString,
    {
        self.headers = self.headers.and_then(|mut headers| {
            let name = HeaderName::from_str(key.as_ref())?;
            let val = HeaderValue::from_str(&value.to_string())?;
            headers.insert(name, val);
            Ok(headers)
        });
        self
    }

    fn body(mut self, data: Vec<u8>) -> Self {
        self.body.replace(data);
        self
    }
}

pub struct AsyncRequest(Request<Body>);

impl AsyncRequest {
    pub async fn send(self, client: &HyperClient<HttpConnector>) -> HyperResult<Response<Vec<u8>>> {
        let res = client.request(self.0).await?;
        let (parts, body) = res.into_parts();
        let body = body::to_bytes(body).await?.to_vec();
        let res = Response::from_parts(parts, body);
        Ok(res)
    }
}

impl From<Request<Body>> for AsyncRequest {
    fn from(req: Request<Body>) -> Self {
        Self(req)
    }
}
