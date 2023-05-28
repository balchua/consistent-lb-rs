use std::collections::{HashMap, HashSet};

use hash_ring::{HashRing, NodeInfo};

pub struct Consistent {
    nodeset: HashSet<NodeInfo>,
    ring: HashRing<NodeInfo>,
    previous: HashMap<String, String>,
}

impl Consistent {
    pub fn new(replicas: isize, nodes: Vec<NodeInfo>) -> Consistent {
        let hash_ring = HashRing::new(nodes, replicas);
        let previous = HashMap::new();
        Consistent {
            ring: hash_ring,
            nodeset: HashSet::new(),
            previous: previous,
        }
    }

    pub fn pick(&self, key: &String) -> NodeInfo {
        let x = self.ring.get_node(key.to_owned());

        match x {
            Some(n) => {
                let node = n.clone();
                log::debug!("node selected is {}:{}", node.host, node.port);
                node
            }
            None => {
                log::debug!("no node selected");
                NodeInfo { host: "", port: 0 }
            }
        }
    }
}
