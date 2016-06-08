use std::collections::HashMap;
use erl_ast::ast;
use ty;
use graph;
use graph::NodeId;
use module::Arity;

#[derive(Debug)]
pub struct Function {
    pub graph: graph::Graph,
}

impl ::ast::FromAst for Function {
    type Input = ast::form::FunDecl;
    fn from_ast(decl: &Self::Input) -> Self {
        Function { graph: GraphBuilder::new().build(decl) }
    }
}

struct GraphBuilder {
    graph: graph::Graph,
    bindings: Vec<HashMap<String, NodeId>>,
}
impl GraphBuilder {
    pub fn new() -> Self {
        GraphBuilder {
            graph: graph::Graph::new(),
            bindings: Vec::new(),
        }
    }
    fn scope_in(&mut self) {
        self.bindings.push(HashMap::new());
    }
    fn scope_out(&mut self) -> HashMap<String, NodeId> {
        self.bindings.pop().unwrap()
    }
    fn intern(&mut self, name: &str) -> NodeId {
        if let Some(id) = self.find_binding(name) {
            id
        } else {
            let id = self.graph.new_value_node(graph::Val::new());
            self.bindings.last_mut().unwrap().insert(name.to_string(), id);
            id
        }
    }
    fn find_binding(&self, name: &str) -> Option<NodeId> {
        for b in self.bindings.iter().rev() {
            if let Some(id) = b.get(name) {
                return Some(*id);
            }
        }
        None
    }
    pub fn build(mut self, decl: &ast::form::FunDecl) -> graph::Graph {
        let arity = decl.clauses[0].patterns.len() as Arity; // FIXME
        let fun_node_id = self.graph.new_external_fun_node(arity);
        for c in &decl.clauses {
            self.scope_in();
            let return_value = self.parse_function_clause(fun_node_id, c);
            self.scope_out();

            let fun_return = self.graph.get_return_node(fun_node_id).unwrap();
            self.graph.add_edge(return_value, fun_return);
        }
        self.graph
    }
    pub fn parse_function_clause(&mut self,
                                 parent: graph::NodeId,
                                 clause: &ast::clause::Clause)
                                 -> graph::NodeId {
        assert!(clause.guards.is_empty());
        for (i, p) in clause.patterns.iter().enumerate() {
            let pattern = self.parse_pattern(p);
            let arg = self.graph.get_nth_arg(parent, i).unwrap();
            self.graph.add_edge(arg, pattern);
        }

        let mut return_value = None;
        for e in &clause.body {
            let tmp_return = self.parse_expr(e);
            return_value = Some(tmp_return);
        }
        return_value.unwrap()
    }

    // NOTE: Returns pattern node (i.e., consumer)
    pub fn parse_pattern(&mut self, pattern: &ast::pat::Pattern) -> graph::NodeId {
        use erl_ast::ast::pat::Pattern as P;
        match *pattern {
            P::Var(ref x) => self.intern(&x.name),
            _ => panic!("PAT: {:?}", pattern),
        }
    }

    // NOTE: Returns return value node (i.e., supplier)
    pub fn parse_expr(&mut self, expr: &ast::expr::Expression) -> graph::NodeId {
        use erl_ast::ast::expr::Expression as E;
        match *expr {
            E::Atom(ref x) => {
                let value = graph::Val::with_type(ty::atom(&x.value));
                self.graph.new_value_node(value)
            }
            E::Nil(_) => {
                let value = graph::Val::with_type(From::from(ty::NilType));
                self.graph.new_value_node(value)
            }
            E::Var(ref x) => {
                let var = self.find_binding(&x.name).unwrap();
                var
            }
            E::LocalCall(ref x) => {
                let fun = self.parse_expr(&x.function);
                let mut args = Vec::with_capacity(x.args.len());
                for a in &x.args {
                    args.push(self.parse_expr(a));
                }
                let node_id = self.graph.new_local_call_node(fun, args);
                self.graph.get_return_node(node_id).unwrap()
            }
            _ => panic!("EXPR: {:?}", expr),
        }
    }
}
// cargo run -- analyze /usr/lib/erlang/lib/stdlib-2.8/ebin/*.beam
