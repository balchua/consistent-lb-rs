use actix_web::{get, web, Result};
use serde::{Deserialize, Serialize};

use crate::ConsistentHash;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
struct Response {
    previous_node: String,
    current_node: String,
}

#[get("/pick/{node}")]
pub async fn pick(path: web::Path<String>, data: web::Data<ConsistentHash>) -> Result<String> {
    let key = path.into_inner();
    let served_node = data.consistent.pick(&key);
    let res = Response {
        previous_node: "".to_string(),
        current_node: format!("{}:{}", served_node.host.to_string(), served_node.port),
    };

    Ok(serde_json::to_string(&res).unwrap())
}
