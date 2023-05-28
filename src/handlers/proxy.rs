use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};

use hyper::{service::Service, Body, Client, Request, Response, Version};

use crate::service::consistent::Consistent;

const REQUEST_KEY: &str = "x-request_key";

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
        let orig_headers = orig_req.headers();
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
        let client = Client::builder()
            .pool_idle_timeout(Duration::from_secs(30))
            .http2_only(true)
            .build_http();

        let uri = format!(
            "http://{}{}",
            host,
            orig_req.uri().path_and_query().unwrap()
        );
        println!("uri is : {}", uri);

        let mut builder = hyper::Request::builder().version(Version::HTTP_2).uri(uri);
        for header in orig_headers {
            builder = builder.header(header.0, header.1.to_owned());
        }

        Box::pin(async move {
            let body_bytes = hyper::body::to_bytes(orig_req.into_body()).await;
            let req = builder
                .method("POST")
                .body(Body::from(body_bytes.unwrap().clone()))
                .unwrap();
            client.request(req).await
        })
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
