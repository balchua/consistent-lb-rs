use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};

use hyper::{
    client::HttpConnector, http::request::Parts, service::Service, Body, Client, Request, Response,
    Version,
};

use crate::service::consistent::Consistent;

const REQUEST_KEY: &str = "x-request-key";

const RANDOM_CHARSET: &str = "abcdefghijklmnopqrstuvwxyz";
pub struct ConsistentProxy {
    c: Arc<Consistent>,
}

impl Service<Request<Body>> for ConsistentProxy {
    type Response = Response<Body>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, orig_req: Request<Body>) -> Self::Future {
        let (parts, body) = orig_req.into_parts();
        let client = self.create_client();
        let req = self.create_request(parts, body);
        Box::pin(async move { client.request(req).await })
    }
}

impl ConsistentProxy {
    fn create_client(&self) -> Client<HttpConnector> {
        Client::builder()
            .pool_idle_timeout(Duration::from_secs(30))
            .http2_only(true)
            .build_http()
    }

    fn create_request(&self, parts: Parts, body: Body) -> Request<Body> {
        let orig_headers = parts.headers;
        let orig_uri = parts.uri;

        let host = match orig_headers.get(REQUEST_KEY) {
            Some(k) => {
                let key = k.to_str().unwrap();
                let node = self.c.pick(&String::from(key));
                format!("{}:{}", node.host, node.port)
            }
            None => {
                let k = random_string::generate(10, RANDOM_CHARSET);
                let node = self.c.pick(&String::from(k));
                format!("{}:{}", node.host, node.port)
            }
        };
        let uri = format!("http://{}{}", host, orig_uri.path_and_query().unwrap());
        println!("{:?}", uri);
        let mut builder = hyper::Request::builder().version(Version::HTTP_2).uri(uri);
        for header in orig_headers {
            builder = builder.header(header.0.unwrap(), header.1.to_owned());
        }
        let req = builder.method("POST").body(body).unwrap();
        req
    }
}
pub struct MakeSvc {
    pub c: Arc<Consistent>,
}

impl<T> Service<T> for MakeSvc {
    type Response = ConsistentProxy;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: T) -> Self::Future {
        let x = self.c.clone();
        let fut = async move { Ok(ConsistentProxy { c: x }) };
        Box::pin(fut)
    }
}
