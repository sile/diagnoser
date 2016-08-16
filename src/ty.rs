use std::collections::HashMap;

use graph;

pub use graph::NodeId;
pub use graph::EdgeId;
pub type AtomName = String;

#[derive(Debug)]
pub struct Node {
    pub id: NodeId,
    pub ty: Type,
}
impl graph::Node for Node {
    fn id(&self) -> &NodeId {
        &self.id
    }
}

#[derive(Debug)]
pub struct Edge {
    pub id: EdgeId,
    pub match_from: NodeId,
    pub match_to: NodeId,
    pub label: Option<String>,
}
impl graph::Edge for Edge {
    fn id(&self) -> &EdgeId {
        &self.id
    }
    fn head_node_id(&self) -> &NodeId {
        &self.match_from
    }
    fn tail_node_id(&self) -> &NodeId {
        &self.match_to
    }
}

#[derive(Debug)]
pub struct Graph {
    graph: graph::Graph<Node, Edge>,
    next_node_id: NodeId,
    next_edge_id: EdgeId,
}
impl Graph {
    pub fn new() -> Self {
        Graph {
            graph: graph::Graph::new(),
            next_node_id: 0,
            next_edge_id: 0,
        }
    }
    pub fn nodes(&self) -> &HashMap<NodeId, Node> {
        &self.graph.nodes
    }
    pub fn edges(&self) -> &HashMap<EdgeId, Edge> {
        &self.graph.edges
    }
    pub fn add_node<T>(&mut self, ty: T) -> NodeId
        where Type: From<T>
    {
        let id = self.next_node_id();
        let node = Node {
            id: id,
            ty: From::from(ty),
        };
        self.graph.nodes.insert(id, node);
        id
    }
    pub fn add_edge(&mut self, from: NodeId, to: NodeId) -> EdgeId {
        let id = self.next_edge_id();
        let e = Edge {
            id: id,
            match_from: from,
            match_to: to,
            label: None,
        };
        self.graph.edges.insert(id, e);
        id
    }
    pub fn add_edge_with_label(&mut self, from: NodeId, to: NodeId, label: &str) -> EdgeId {
        let id = self.next_edge_id();
        let e = Edge {
            id: id,
            match_from: from,
            match_to: to,
            label: Some(label.to_string()),
        };
        self.graph.edges.insert(id, e);
        id
    }

    fn next_node_id(&mut self) -> NodeId {
        let id = self.next_node_id;
        self.next_node_id = self.next_node_id.wrapping_add(1);
        id
    }
    fn next_edge_id(&mut self) -> EdgeId {
        let id = self.next_edge_id;
        self.next_edge_id = self.next_edge_id.wrapping_add(1);
        id
    }
}

pub trait Set {
    // fn intersection(&self, &Type, &Graph) -> Type;
    // union
}

macro_rules! impl_from {
    ($to:ident :: $cons:ident ( $from:ty )) => {
        impl ::std::convert::From<$from> for $to {
            fn from(x: $from) -> Self {
                $to::$cons(::std::convert::From::from(x))
            }
        }
    }
}

#[derive(Debug)]
pub enum Type {
    None(NoneType),
    Any(AnyType),
    Nil(NilType),
    Atom(AtomType),
    Int(IntType),
    Cons(ConsType),
    Str(StrType),
    Tuple(TupleType),
    Union(UnionType),
    Fun(FunType),
    LocalFun(LocalFunType),
    RemoteFun(RemoteFunType),
    BuiltIn(BuiltInType),
    Var(VarType),
}
impl Set for Type {}
impl_from!(Type::None(NoneType));
impl_from!(Type::Any(AnyType));
impl_from!(Type::Nil(NilType));
impl_from!(Type::Atom(AtomType));
impl_from!(Type::Int(IntType));
impl_from!(Type::Cons(ConsType));
impl_from!(Type::Str(StrType));
impl_from!(Type::Tuple(TupleType));
impl_from!(Type::Union(UnionType));
impl_from!(Type::Fun(FunType));
impl_from!(Type::LocalFun(LocalFunType));
impl_from!(Type::RemoteFun(RemoteFunType));
impl_from!(Type::BuiltIn(BuiltInType));
impl_from!(Type::Var(VarType));
impl Type {
    pub fn label(&self) -> String {
        match *self {
            Type::None(_) => "none()".to_string(),
            Type::Any(_) => "any()".to_string(),
            Type::Atom(ref x) => format!("'{}'", x.name),
            Type::Nil(_) => "[]".to_string(),
            Type::Int(_) => "todo:int".to_string(),
            Type::Cons(_) => "cons".to_string(),
            Type::Str(ref x) => format!("{:?}", x.value),
            Type::Tuple(_) => "todo:tuple".to_string(),
            Type::Union(_) => "todo:union".to_string(),
            Type::Fun(_) => "fun".to_string(),
            Type::LocalFun(ref x) => x.label(),
            Type::RemoteFun(ref x) => x.label(),
            Type::BuiltIn(ref x) => x.label(),
            Type::Var(ref x) => x.name.clone(),
        }
    }
    pub fn get_children(&self) -> Vec<(String, NodeId)> {
        match *self {
            Type::Fun(ref x) => x.get_children(),
            Type::BuiltIn(ref x) => x.get_children(),
            Type::Cons(ref x) => x.get_children(),
            _ => Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct NoneType;
impl Set for NoneType {}

#[derive(Debug)]
pub struct AnyType;

#[derive(Debug)]
pub struct AtomType {
    pub name: AtomName,
}
impl Set for AtomType {}

#[derive(Debug)]
pub struct IntType {
    pub value: i64,
}

#[derive(Debug)]
pub struct NilType;

#[derive(Debug)]
pub struct ConsType {
    pub head: NodeId,
    pub tail: NodeId,
}
impl ConsType {
    pub fn get_children(&self) -> Vec<(String, NodeId)> {
        vec![("head".to_string(), self.head), ("tail".to_string(), self.tail)]
    }
}

#[derive(Debug)]
pub struct StrType {
    pub value: String,
}

#[derive(Debug)]
pub struct TupleType {
    pub elements: Vec<NodeId>,
}
impl Set for TupleType {}

#[derive(Debug)]
pub struct UnionType {
    pub types: Vec<NodeId>,
}
impl Set for UnionType {}

#[derive(Debug)]
pub struct FunType {
    pub args: Vec<NodeId>,
    pub result: NodeId,
}
impl FunType {
    pub fn get_children(&self) -> Vec<(String, NodeId)> {
        let mut v = Vec::new();
        for (i, &a) in self.args.iter().enumerate() {
            v.push((format!("arg{}", i), a));
        }
        v.push(("res".to_string(), self.result));
        v
    }
}

#[derive(Debug)]
pub struct LocalFunType {
    pub funame: String,
    pub arity: u8,
}
impl LocalFunType {
    pub fn label(&self) -> String {
        format!("fun {}/{}", self.funame, self.arity)
    }
}

#[derive(Debug)]
pub struct RemoteFunType {
    pub module: String,
    pub funame: String,
    pub arity: u8,
}
impl RemoteFunType {
    pub fn label(&self) -> String {
        format!("fun {}:{}/{}", self.module, self.funame, self.arity)
    }
}

#[derive(Debug)]
pub struct BuiltInType {
    pub name: String,
    pub args: Vec<NodeId>,
}
impl BuiltInType {
    pub fn label(&self) -> String {
        format!("{}/{}", self.name, self.args.len())
    }
    pub fn get_children(&self) -> Vec<(String, NodeId)> {
        let mut v = Vec::new();
        for (i, a) in self.args.iter().enumerate() {
            v.push((format!("arg{}", i), *a));
        }
        v
    }
}

#[derive(Debug)]
pub struct VarType {
    pub name: String,
}
