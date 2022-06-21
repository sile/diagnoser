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
    pub fn add_edge(&mut self, kind: EdgeKind, producer: NodeId, consumer: NodeId) -> EdgeId {
        let id = self.next_edge_id();
        let edge = Edge {
            id: id,
            kind: kind,
            producer: producer,
            consumer: consumer,
        };
        self.edges.insert(id, edge);
        self.nodes.get_mut(&producer).unwrap().edges.insert(id);
        self.nodes.get_mut(&consumer).unwrap().edges.insert(id);
        id
    }

    pub fn new_conj(&mut self, nodes: Vec<NodeId>) {
        let conj = Conj::new(self, nodes.clone());
        let conj_id = self.new_node(Content::Conj(conj));
        for id in nodes {
            self.add_edge(EdgeKind::Conj, id, conj_id);
        }
    }

    pub fn new_external_fun_node(&mut self, arity: Arity) -> NodeId {
        let fun = Fun::new(self, arity);
        self.new_node(Content::Fun(fun))
    }

    pub fn new_remote_call_node(&mut self,
                                module: NodeId,
                                fun: NodeId,
                                args: Vec<NodeId>)
                                -> NodeId {
        let call = RemoteCall::new(self, module, fun, args);
        self.new_node(Content::RemoteCall(call))
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
                Content::RemoteCall(ref x) => Some(x.return_value),
                _ => None,
            }
        })
    }
    pub fn get_args(&mut self, node_id: NodeId) -> Option<&[NodeId]> {
        self.nodes.get(&node_id).and_then(|node| {
            match node.content {
                Content::Fun(ref x) => Some(x.args.as_slice()),
                _ => None,
            }
        })
    }
    // pub fn get_nth_arg(&mut self, node_id: NodeId, index: usize) -> Option<NodeId> {
    //     self.nodes.get(&node_id).and_then(|node| {
    //         match node.content {
    //             Content::Fun(ref x) => Some(x.args[index]),
    //             _ => None,
    //         }
    //     })
    // }

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
    pub fn write_as_dot<W: ::std::io::Write>(&self, writer: W) -> ::std::io::Result<()> {
        ::graph_dot::DotWriter::new(writer).write(self)
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
pub struct RemoteCall {
    pub module: NodeId,
    pub fun: NodeId,
    pub args: Vec<NodeId>,
    pub return_value: NodeId,
}
impl RemoteCall {
    pub fn new(graph: &mut Graph, module: NodeId, fun: NodeId, args: Vec<NodeId>) -> Self {
        let return_value = graph.new_value_node(Val::new());
        RemoteCall {
            module: module,
            fun: fun,
            args: args,
            return_value: return_value,
        }
    }
}

#[derive(Debug)]
pub struct Conj {
    pub nodes: Vec<NodeId>,
}
impl Conj {
    pub fn new(_graph: &mut Graph, nodes: Vec<NodeId>) -> Self {
        Conj { nodes: nodes }
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
    pub fn new_any() -> Self {
        Val {
            producible_type: From::from(ty::AnyType),
            consumable_type: From::from(ty::AnyType),
        }
    }
    pub fn new_var() -> Self {
        Val {
            producible_type: From::from(ty::NoneType),
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
    RemoteCall(RemoteCall),
    Conj(Conj),
}
impl Content {
    pub fn label(&self) -> String {
        match *self {
            Content::Fun(ref _x) => format!("fun"),
            Content::Val(ref x) => format!("({}=>{})", x.producible_type, x.consumable_type),
            Content::LocalCall(ref _x) => format!("local"),
            Content::RemoteCall(ref _x) => format!("remote"),
            Content::Conj(ref _x) => format!("conj"),
        }
    }
    pub fn link_nodes(&self) -> Vec<(EdgeKind, NodeId)> {
        let mut nodes = Vec::new();
        match *self {
            Content::Fun(ref x) => {
                for (i, a) in x.args.iter().enumerate() {
                    nodes.push((EdgeKind::Param(i), *a));
                }
                nodes.push((EdgeKind::Return, x.return_value));
            }
            Content::Val(_) => {}
            Content::LocalCall(ref x) => {
                nodes.push((EdgeKind::Fun, x.fun));
                for (i, a) in x.args.iter().enumerate() {
                    nodes.push((EdgeKind::Arg(i), *a));
                }
                nodes.push((EdgeKind::Return, x.return_value));
            }
            Content::RemoteCall(ref x) => {
                nodes.push((EdgeKind::Module, x.module));
                nodes.push((EdgeKind::Fun, x.fun));
                for (i, a) in x.args.iter().enumerate() {
                    nodes.push((EdgeKind::Arg(i), *a));
                }
                nodes.push((EdgeKind::Return, x.return_value));
            }
            Content::Conj(ref x) => {
                for (_i, n) in x.nodes.iter().enumerate() {
                    nodes.push((EdgeKind::Conj, *n));
                }
            }
        }
        nodes
    }
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
    pub fn label(&self) -> String {
        let content = self.content.label();
        format!("{}@{}", self.id, content)
    }
}

#[derive(Debug)]
pub struct Edge {
    pub id: EdgeId,
    pub kind: EdgeKind,
    pub producer: NodeId,
    pub consumer: NodeId,
}

#[derive(Debug)]
pub enum EdgeKind {
    Param(usize),
    Arg(usize),
    Return,
    Conj,
    Match,
    Fun,
    Module,
    Unknown,
}
impl EdgeKind {
    pub fn label(&self) -> String {
        match *self {
            EdgeKind::Param(i) => format!("p[{}]", i),
            EdgeKind::Arg(i) => format!("a[{}]", i),
            EdgeKind::Return => format!("ret"),
            EdgeKind::Conj => format!("conj"),
            EdgeKind::Match => format!("mat"),
            EdgeKind::Fun => format!("fun"),
            EdgeKind::Module => format!("mod"),
            EdgeKind::Unknown => format!("unk"),
        }
    }
}

#[derive(Debug)]
pub enum Target {
    Type(module::Remote),
    Fun(module::Remote),
}
