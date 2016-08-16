use std::collections::HashMap;

pub type NodeId = u64;
pub type EdgeId = u64;

#[derive(Debug)]
pub struct Graph<N, E> {
    pub nodes: HashMap<NodeId, N>,
    pub edges: HashMap<EdgeId, E>,
}
impl<N, E> Graph<N, E> {
    pub fn new() -> Self {
        Graph {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        }
    }
}

pub trait Node {
    fn id(&self) -> &NodeId;
}

pub trait Edge {
    fn id(&self) -> &EdgeId;
    fn head_node_id(&self) -> &NodeId;
    fn tail_node_id(&self) -> &NodeId;
}
