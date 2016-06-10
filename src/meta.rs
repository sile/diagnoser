use std::collections::HashMap;
use num::traits::ToPrimitive;
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
        let args = Vec::from(self.graph.get_args(fun_node_id).unwrap());
        let fun_return = self.graph.get_return_node(fun_node_id).unwrap();
        for c in &decl.clauses {
            self.parse_clause(&args, fun_return, c);
        }

        {
            use std::fs;
            use std::io::Write;
            let f = fs::File::create(format!("/tmp/graph_{}_{}.dot", decl.name, arity)).unwrap();
            self.graph.write_as_dot(f).unwrap();
        }

        self.graph
    }
    pub fn parse_clause(&mut self,
                        args: &[graph::NodeId],
                        result: graph::NodeId,
                        clause: &ast::clause::Clause) {
        if args.len() != clause.patterns.len() {
            panic!("args.len={}, clause={:?}", args.len(), clause);
        }
        self.scope_in();

        for (i, p) in clause.patterns.iter().enumerate() {
            let pattern = self.parse_pattern(p);
            let arg = args[i];
            self.graph.add_edge(graph::EdgeKind::Match, arg, pattern);
        }

        // NOTE:
        // guardの場合には、内部的に専用の関数に
        // 変換してあげる必要があるかもしれない.
        // (他と同じ仕組みで扱えるようにするには)
        // e.g., `is_atom() => -spec guard_is_atom(atom()) -> true.`
        for g in &clause.guards {
            self.parse_and_guards(&g.and_guards);
        }
        let clause_result = self.parse_body(&clause.body);
        self.graph.add_edge(graph::EdgeKind::Return, clause_result, result);
        self.scope_out();
    }
    pub fn parse_body(&mut self, body: &[ast::expr::Expression]) -> graph::NodeId {
        let mut return_value = None;
        for e in body {
            let tmp_return = self.parse_expr(e);
            return_value = Some(tmp_return);
        }
        return_value.unwrap()
    }
    pub fn parse_and_guards(&mut self, guards: &Vec<ast::guard::Guard>) {
        let mut conjunctions = Vec::with_capacity(guards.len());
        for g in guards {
            conjunctions.push(self.parse_guard(g));
        }
        self.graph.new_conj(conjunctions);
    }
    pub fn parse_guard(&mut self, guard: &ast::guard::Guard) -> graph::NodeId {
        use erl_ast::ast::guard::Guard as G;
        match *guard {
            G::Atom(ref x) => {
                let value = graph::Val::with_type(ty::atom(&x.value));
                self.graph.new_value_node(value)
            }
            G::Integer(ref x) => {
                let value = if let Some(v) = x.value.to_i64() {
                    graph::Val::with_type(From::from(ty::integer().value(v)))
                } else {
                    graph::Val::with_type(From::from(ty::integer()))
                };
                self.graph.new_value_node(value)
            }
            G::Nil(_) => {
                let value = graph::Val::with_type(From::from(ty::NilType));
                self.graph.new_value_node(value)
            }
            G::Var(ref x) => {
                let var = self.find_binding(&x.name).unwrap();
                var
            }
            G::BinaryOp(ref x) => {
                let name = {
                    let name = graph::Val::with_type(ty::atom(&format!("__op_{}", x.operator)));
                    self.graph.new_value_node(name)
                };
                let arg0 = self.parse_guard(&x.left_operand);
                let arg1 = self.parse_guard(&x.right_operand);
                let node_id = self.graph.new_local_call_node(name, vec![arg0, arg1]);
                self.graph.get_return_node(node_id).unwrap()
            }
            G::LocalCall(ref x) => {
                let fun = self.parse_guard(&x.function);
                let mut args = Vec::with_capacity(x.args.len());
                for a in &x.args {
                    args.push(self.parse_guard(a));
                }
                let node_id = self.graph.new_local_call_node(fun, args);
                self.graph.get_return_node(node_id).unwrap()
            }
            _ => panic!("GUARD: {:?}", guard),
        }
    }

    // NOTE: Returns pattern node (i.e., consumer)
    pub fn parse_pattern(&mut self, pattern: &ast::pat::Pattern) -> graph::NodeId {
        use erl_ast::ast::pat::Pattern as P;
        match *pattern {
            P::Atom(ref x) => {
                let value = graph::Val::with_type(ty::atom(&x.value));
                self.graph.new_value_node(value)
            }
            P::Integer(ref x) => {
                let value = if let Some(v) = x.value.to_i64() {
                    graph::Val::with_type(From::from(ty::integer().value(v)))
                } else {
                    graph::Val::with_type(From::from(ty::integer()))
                };
                self.graph.new_value_node(value)
            }
            P::Nil(_) => {
                let value = graph::Val::with_type(From::from(ty::NilType));
                self.graph.new_value_node(value)
            }
            P::Var(ref x) => self.intern(&x.name),
            P::Match(ref x) => {
                let left = self.parse_pattern(&x.left);
                let right = self.parse_pattern(&x.right);
                self.graph.add_edge(graph::EdgeKind::Match, right, left);
                left
            }
            P::Cons(ref x) => {
                let name = {
                    let name = graph::Val::with_type(ty::atom("__cons"));
                    self.graph.new_value_node(name)
                };
                let arg0 = self.parse_pattern(&x.head);
                let arg1 = self.parse_pattern(&x.tail);
                let node_id = self.graph.new_local_call_node(name, vec![arg0, arg1]);
                self.graph.get_return_node(node_id).unwrap()
            }
            P::Record(ref x) => {
                let name = {
                    let name = graph::Val::with_type(ty::atom(&format!("__record_{}", x.name)));
                    self.graph.new_value_node(name)
                };
                let mut args = Vec::with_capacity(x.fields.len());
                for f in &x.fields {
                    let field_id = {
                        let name = f.name.clone().expect("Not Implemented");
                        let name = self.graph
                            .new_value_node(graph::Val::with_type(ty::atom(&format!("__field_{}_{}",
                                                                                    x.name,
                                                                                    name))));
                        let arg = self.parse_pattern(&f.value);
                        self.graph.new_local_call_node(name, vec![arg])
                    };
                    args.push(field_id);
                }
                let node_id = self.graph.new_local_call_node(name, args);
                self.graph.get_return_node(node_id).unwrap()
            }
            P::Tuple(ref x) => {
                let name = {
                    let name = graph::Val::with_type(ty::atom("__tuple"));
                    self.graph.new_value_node(name)
                };
                let mut args = Vec::with_capacity(x.elements.len());
                for e in &x.elements {
                    args.push(self.parse_pattern(e));
                }
                let node_id = self.graph.new_local_call_node(name, args);
                self.graph.get_return_node(node_id).unwrap()
            }
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
            E::Integer(ref x) => {
                let value = if let Some(v) = x.value.to_i64() {
                    graph::Val::with_type(From::from(ty::integer().value(v)))
                } else {
                    graph::Val::with_type(From::from(ty::integer()))
                };
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
            E::Match(ref x) => {
                let left = self.parse_pattern(&x.left);
                let right = self.parse_expr(&x.right);
                self.graph.add_edge(graph::EdgeKind::Match, right, left);
                left
            }
            E::Case(ref x) => {
                let result_value = self.graph.new_value_node(graph::Val::new_var());
                let expr_value = self.parse_expr(&x.expr);
                for clause in &x.clauses {
                    self.parse_clause(&[expr_value], result_value, clause);
                }
                result_value
            }
            E::Try(ref x) => {
                let result_value = self.graph.new_value_node(graph::Val::new_var());
                let body_value = self.parse_body(&x.body);
                for clause in &x.case_clauses {
                    self.parse_clause(&[body_value], result_value, clause);
                }

                // FIXME: Pass possible catch value type
                let catch_value = self.graph.new_value_node(graph::Val::new_any());
                for clause in &x.catch_clauses {
                    self.parse_clause(&[catch_value], result_value, clause);
                }
                if !x.after.is_empty() {
                    self.parse_body(&x.after);
                }
                result_value
            }
            E::If(ref x) => {
                let result_value = self.graph.new_value_node(graph::Val::new_var());
                for c in &x.clauses {
                    self.parse_clause(&[], result_value, c);
                }
                result_value
            }
            E::Record(ref x) => {
                // NOTE: record_foo(field1(value1), field2(value2), ...) => record()
                let name = {
                    let name = graph::Val::with_type(ty::atom(&format!("__record_{}", x.name)));
                    self.graph.new_value_node(name)
                };
                let mut args = Vec::with_capacity(x.fields.len());
                for f in &x.fields {
                    let field_id = {
                        let name = f.name.clone().expect("Not Implemented");
                        let name = self.graph
                            .new_value_node(graph::Val::with_type(ty::atom(&format!("__field_{}_{}",
                                                                                    x.name,
                                                                                    name))));
                        let arg = self.parse_expr(&f.value);
                        self.graph.new_local_call_node(name, vec![arg])
                    };
                    args.push(field_id);
                }
                let node_id = self.graph.new_local_call_node(name, args);
                self.graph.get_return_node(node_id).unwrap()
            }
            E::Cons(ref x) => {
                let name = {
                    let name = graph::Val::with_type(ty::atom("__cons"));
                    self.graph.new_value_node(name)
                };
                let arg0 = self.parse_expr(&x.head);
                let arg1 = self.parse_expr(&x.tail);
                let node_id = self.graph.new_local_call_node(name, vec![arg0, arg1]);
                self.graph.get_return_node(node_id).unwrap()
            }
            E::Tuple(ref x) => {
                let name = {
                    let name = graph::Val::with_type(ty::atom("__tuple"));
                    self.graph.new_value_node(name)
                };
                let mut args = Vec::with_capacity(x.elements.len());
                for e in &x.elements {
                    args.push(self.parse_expr(e));
                }
                let node_id = self.graph.new_local_call_node(name, args);
                self.graph.get_return_node(node_id).unwrap()
            }
            E::BinaryOp(ref x) => {
                let name = {
                    let name = graph::Val::with_type(ty::atom(&format!("__op_{}", x.operator)));
                    self.graph.new_value_node(name)
                };
                let arg0 = self.parse_expr(&x.left_operand);
                let arg1 = self.parse_expr(&x.right_operand);
                let node_id = self.graph.new_local_call_node(name, vec![arg0, arg1]);
                self.graph.get_return_node(node_id).unwrap()
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
            E::RemoteCall(ref x) => {
                let module = self.parse_expr(&x.module);
                let fun = self.parse_expr(&x.function);
                let mut args = Vec::with_capacity(x.args.len());
                for a in &x.args {
                    args.push(self.parse_expr(a));
                }
                let node_id = self.graph.new_remote_call_node(module, fun, args);
                self.graph.get_return_node(node_id).unwrap()
            }
            E::AnonymousFun(ref x) => {
                // TODO: handle escaped case
                // self.scope_in();
                // x.clauses,
                // if let Some(name) = x.name {
                //     //
                // }
                // self.scope_out();

                // TODO: implements
                self.graph.new_value_node(graph::Val::new_any())
            }
            _ => panic!("EXPR: {:?}", expr),
        }
    }
}
// cargo run -- analyze /usr/lib/erlang/lib/stdlib-2.8/ebin/*.beam
