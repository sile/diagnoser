use erl_ast::ast;
use ty;
use graph;
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
}
impl GraphBuilder {
    pub fn new() -> Self {
        GraphBuilder { graph: graph::Graph::new() }
    }
    pub fn build(mut self, decl: &ast::form::FunDecl) -> graph::Graph {
        let arity = decl.clauses[0].patterns.len() as Arity; // FIXME
        let fun_node_id = self.graph.new_external_fun_node(arity);
        for c in &decl.clauses {
            let return_value = self.parse_clause(fun_node_id, c);
            let fun_return = self.graph.get_return_node(fun_node_id).unwrap();
            self.graph.add_edge(return_value, fun_return);
        }
        self.graph
    }
    pub fn parse_clause(&mut self,
                        _parent: graph::NodeId,
                        clause: &ast::clause::Clause)
                        -> graph::NodeId {
        assert!(clause.patterns.is_empty());
        assert!(clause.guards.is_empty());
        let mut return_value = None;
        for e in &clause.body {
            let tmp_return = self.parse_expr(e);
            return_value = Some(tmp_return);
        }
        return_value.unwrap()
    }
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
