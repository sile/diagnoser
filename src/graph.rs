use std::collections::HashMap;
use std::collections::HashSet;
use module;
use module::Arity;
use ty;

pub type NodeId = usize;
pub type EdgeId = usize;

#[derive(Debug)]
pub struct Graph {
    pub next_node_id: NodeId,
    pub next_edge_id: EdgeId,
    pub nodes: HashMap<NodeId, Node>,
    pub edges: HashMap<EdgeId, Edge>,
}
impl Graph {
    pub fn new() -> Self {
        Graph {
            next_node_id: 0,
            next_edge_id: 0,
            nodes: HashMap::new(),
            edges: HashMap::new(),
        }
    }
    pub fn add_edge(&mut self, producer: NodeId, consumer: NodeId) -> EdgeId {
        let id = self.next_edge_id();
        let edge = Edge {
            id: id,
            producer: producer,
            consumer: consumer,
        };
        self.edges.insert(id, edge);
        self.nodes.get_mut(&producer).unwrap().edges.insert(id);
        self.nodes.get_mut(&consumer).unwrap().edges.insert(id);
        id
    }

    pub fn new_external_fun_node(&mut self, arity: Arity) -> NodeId {
        let fun = Fun::new(self, arity);
        self.new_node(Content::Fun(fun))
    }

    pub fn new_local_call_node(&mut self, fun: NodeId, args: Vec<NodeId>) -> NodeId {
        let call = LocalCall::new(self, fun, args);
        self.new_node(Content::LocalCall(call))
    }

    pub fn new_value_node(&mut self, value: Val) -> NodeId {
        let content = Content::Val(value);
        self.new_node(content)
    }

    pub fn get_return_node(&mut self, node_id: NodeId) -> Option<NodeId> {
        self.nodes.get(&node_id).and_then(|node| {
            match node.content {
                Content::Fun(ref x) => Some(x.return_value),
                Content::LocalCall(ref x) => Some(x.return_value),
                _ => None,
            }
        })
    }

    pub fn get_nth_arg(&mut self, node_id: NodeId, index: usize) -> Option<NodeId> {
        self.nodes.get(&node_id).and_then(|node| {
            match node.content {
                 Content::Fun(ref x) => Some(x.args[index]),
                _ => None 
            }
        })
    }

    fn new_node(&mut self, content: Content) -> NodeId {
        let node_id = self.next_node_id();
        let node = Node::new(node_id, content);
        self.nodes.insert(node_id, node);
        node_id
    }

    fn next_node_id(&mut self) -> NodeId {
        let id = self.next_node_id;
        self.next_node_id += 1;
        id
    }
    fn next_edge_id(&mut self) -> EdgeId {
        let id = self.next_edge_id;
        self.next_edge_id += 1;
        id
    }
}

#[derive(Debug)]
pub struct Fun {
    pub args: Vec<NodeId>,
    pub return_value: NodeId,
}
impl Fun {
    pub fn new(graph: &mut Graph, arity: Arity) -> Self {
        let args = (0..arity).map(|_| graph.new_value_node(Val::new())).collect();
        let return_value = graph.new_value_node(Val::new());
        Fun {
            args: args,
            return_value: return_value,
        }
    }
}

#[derive(Debug)]
pub struct LocalCall {
    pub fun: NodeId,
    pub args: Vec<NodeId>,
    pub return_value: NodeId,
}
impl LocalCall {
    pub fn new(graph: &mut Graph, fun: NodeId, args: Vec<NodeId>) -> Self {
        let return_value = graph.new_value_node(Val::new());
        LocalCall {
            fun: fun,
            args: args,
            return_value: return_value,
        }
    }
}

#[derive(Debug)]
pub struct Val {
    pub producible_type: ty::Type,
    pub consumable_type: ty::Type,
}
impl Val {
    pub fn new() -> Self {
        Val {
            producible_type: From::from(ty::AnyType),
            consumable_type: From::from(ty::AnyType),
        }
    }
    pub fn with_type(ty: ty::Type) -> Self {
        Val {
            producible_type: ty.clone(),
            consumable_type: ty,
        }
    }
}

#[derive(Debug)]
pub enum Content {
    Fun(Fun),
    Val(Val),
    LocalCall(LocalCall),
}

#[derive(Debug)]
pub struct Node {
    // TODO: pub ast: &::erl_ast::ast::Node,
    pub id: NodeId,
    pub content: Content,
    pub edges: HashSet<EdgeId>,
    pub depends_on: Vec<Target>,
}
impl Node {
    pub fn new(id: NodeId, content: Content) -> Self {
        Node {
            id: id,
            content: content,
            edges: HashSet::new(),
            depends_on: Vec::new(),
        }
    }
    // pub fn add_producer(&mut self, graph: &mut Graph, producer: NodeId) {
    //     let edge_id = graph.add_edge(producer, self.id);
    //     self.edges.insert(edge_id);
    // }
}

#[derive(Debug)]
pub struct Edge {
    pub id: EdgeId,
    pub producer: NodeId,
    pub consumer: NodeId,
}

#[derive(Debug)]
pub enum Target {
    Type(module::Remote),
    Fun(module::Remote),
}
