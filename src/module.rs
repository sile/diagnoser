use std::error;
use std::result;
use std::path::Path;
use std::collections::HashMap;
use erl_ast;

use ty;

pub type ModuleName = String;
pub type Result<T> = result::Result<T, Box<error::Error>>;
pub type Arity = u8;

#[derive(Debug)]
pub struct Module {
    pub name: ModuleName,
    pub graph: ty::Graph,
    pub funs: HashMap<Local, ty::NodeId>,
}
impl Module {
    pub fn from_beam_file<P: AsRef<Path>>(beam_file: P) -> Result<Self> {
        let ast = try!(erl_ast::AST::from_beam_file(beam_file));
        ModuleBuilder::new().build(ast)
    }
}

#[derive(Hash,PartialEq,Eq,Debug)]
pub struct Local {
    pub name: String,
    pub arity: Arity,
}
impl Local {
    pub fn new(name: &str, arity: Arity) -> Self {
        Local {
            name: name.to_string(),
            arity: arity,
        }
    }
}

#[derive(Debug)]
pub struct Bindings {
    stack: Vec<HashMap<String, ty::NodeId>>,
}
impl Bindings {
    pub fn new() -> Self {
        Bindings { stack: Vec::new() }
    }
    pub fn scope_in(&mut self) {
        self.stack.push(HashMap::new());
    }
    pub fn scope_out(&mut self) {
        self.stack.pop();
    }
    pub fn get(&self, name: &str) -> Option<ty::NodeId> {
        self.stack.iter().rev().filter_map(|s| s.get(name).cloned()).nth(0)
    }
    pub fn bind(&mut self, name: &str, node: ty::NodeId) {
        self.stack.last_mut().unwrap().insert(name.to_string(), node);
    }
}

#[derive(Debug)]
pub struct ModuleBuilder {
    name: Option<ModuleName>,
    graph: ty::Graph,
    specs: HashMap<Local, ty::NodeId>,
    funs: HashMap<Local, ty::NodeId>,
    bindings: Bindings,
}
impl ModuleBuilder {
    pub fn new() -> Self {
        ModuleBuilder {
            name: None,
            graph: ty::Graph::new(),
            specs: HashMap::new(),
            funs: HashMap::new(),
            bindings: Bindings::new(),
        }
    }
    pub fn build(mut self, ast: erl_ast::AST) -> Result<Module> {
        for form in &ast.module.forms {
            try!(self.handle_form(form))
        }
        let name = try!(self.name.ok_or("No -module(...) directive"));

        //
        for (key, spec) in &self.specs {
            if let Some(fun) = self.funs.get(key) {
                self.graph.add_edge_with_label(*fun, *spec, "spec");
            }
        }

        //
        for (key, fun) in &self.funs {
            let node = self.graph.add_node(ty::LocalFunType {
                funame: key.name.clone(),
                arity: key.arity,
            });
            self.graph.add_edge(node, *fun);
        }

        Ok(Module {
            name: name,
            graph: self.graph,
            funs: self.funs,
        })
    }
    fn handle_form(&mut self, form: &erl_ast::ast::form::Form) -> Result<()> {
        use erl_ast::ast::form::Form;
        match *form {
            Form::Module(ref x) => self.name = Some(x.name.clone()),
            Form::Spec(ref x) => {
                assert!(!x.types.is_empty());
                let arity = x.types[0].args.len() as Arity;
                let clauses = x.types
                    .iter()
                    .map(|c| self.spec_clause_to_graph(c))
                    .collect::<Vec<_>>();
                let node = if clauses.len() == 1 {
                    clauses[0]
                } else {
                    self.graph.add_node(ty::UnionType { types: clauses })
                };
                self.specs.insert(Local::new(&x.name, arity), node);
            }
            Form::Fun(ref x) => {
                assert!(!x.clauses.is_empty());
                let arity = x.clauses[0].patterns.len() as Arity;
                let clauses = x.clauses
                    .iter()
                    .map(|c| self.fun_clause_to_graph(c))
                    .collect::<Vec<_>>();
                let node = if clauses.len() == 1 {
                    clauses[0]
                } else {
                    self.graph.add_node(ty::UnionType { types: clauses })
                };
                self.funs.insert(Local::new(&x.name, arity), node);
            }
            Form::Export(_) |
            Form::ExportType(_) => {
                // TODO
            }
            Form::File(_) => {}
            Form::Eof(_) => {}
            _ => panic!("form: {:?}", form),
        }
        Ok(())
    }
    fn spec_clause_to_graph(&mut self, f: &erl_ast::ast::ty::Fun) -> ty::NodeId {
        assert!(f.constraints.is_empty());
        let args = f.args.iter().map(|a| self.add_type(a)).collect::<Vec<_>>();
        let result = self.add_type(&f.return_type);
        for c in &f.constraints {
            panic!("Unimplemented: {:?}", c);
        }
        self.graph.add_node(ty::FunType {
            args: args,
            result: result,
        })
    }
    fn fun_clause_to_graph(&mut self, c: &erl_ast::ast::clause::Clause) -> ty::NodeId {
        assert!(c.guards.is_empty());
        assert!(!c.body.is_empty());
        self.bindings.scope_in();

        let mut args = Vec::new();
        for p in &c.patterns {
            args.push(self.parse_pattern(p));
        }

        let mut result = None;
        for e in &c.body {
            let tmp_result = self.parse_expr(e);
            result = Some(tmp_result);
        }

        self.bindings.scope_out();
        self.graph.add_node(ty::FunType {
            args: args,
            result: result.unwrap(),
        })
    }
    fn parse_pattern(&mut self, p: &erl_ast::ast::pat::Pattern) -> ty::NodeId {
        use erl_ast::ast::pat::*;
        match *p {
            Pattern::Var(ref x) => {
                if let Some(node) = self.bindings.get(&x.name) {
                    node
                } else {
                    let node = self.graph.add_node(ty::VarType { name: x.name.clone() });
                    self.bindings.bind(&x.name, node);
                    node
                }
            }
            _ => panic!("pattern: {:?}", p),
        }
    }
    fn parse_expr(&mut self, e: &erl_ast::ast::expr::Expression) -> ty::NodeId {
        use erl_ast::ast::expr::*;
        match *e {
            Expression::Nil(_) => self.graph.add_node(ty::NilType),
            Expression::String(ref e) => {
                self.graph.add_node(ty::StrType { value: e.value.clone() })
            }
            Expression::Atom(ref e) => self.graph.add_node(ty::AtomType { name: e.value.clone() }),
            Expression::Var(ref e) => {
                // TODO: 共通化
                if let Some(node) = self.bindings.get(&e.name) {
                    node
                } else {
                    let node = self.graph.add_node(ty::VarType { name: e.name.clone() });
                    self.bindings.bind(&e.name, node);
                    node
                }
            }
            Expression::RemoteCall(ref e) => {
                let module = to_atom_name(&e.module).unwrap();
                let funame = to_atom_name(&e.function).unwrap();
                let args = e.args.iter().map(|a| self.parse_expr(a)).collect::<Vec<_>>();
                let result = self.graph.add_node(ty::AnyType);

                let remote = self.graph.add_node(ty::RemoteFunType {
                    module: module,
                    funame: funame,
                    arity: args.len() as Arity,
                });
                let fun = self.graph.add_node(ty::FunType {
                    args: args,
                    result: result,
                });
                self.graph.add_edge(fun, remote);
                result
            }
            Expression::Cons(ref x) => {
                let head = self.parse_expr(&x.head);
                let tail = self.parse_expr(&x.tail);
                self.graph.add_node(ty::ConsType {
                    head: head,
                    tail: tail,
                })
            }
            _ => panic!("expr: {:?}", e),
        }
    }
    fn add_type(&mut self, ty: &erl_ast::ast::ty::Type) -> ty::NodeId {
        use erl_ast::ast::ty::*;
        match *ty {
            Type::Atom(ref x) => {
                let a = ty::AtomType { name: x.value.clone() };
                self.graph.add_node(a)
            }
            Type::BuiltIn(ref x) => {
                let t = ty::BuiltInType {
                    name: x.name.clone(),
                    args: x.args.iter().map(|a| self.add_type(a)).collect(),
                };
                self.graph.add_node(t)
            }
            _ => panic!("{:?}", ty),
        }
    }
}
fn to_atom_name(e: &erl_ast::ast::expr::Expression) -> Option<String> {
    use erl_ast::ast::expr::*;
    if let Expression::Atom(ref x) = *e {
        Some(x.value.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use graphviz;
    use super::*;

    #[test]
    fn it_works() {
        let module = Module::from_beam_file("src/testdata/hello.beam").unwrap();
        let renderer = graphviz::for_dot_file("/tmp/hello.dot", &module.graph).unwrap();
        // renderer.render_tree(*module.funs.values().nth(0).unwrap()).unwrap();
        renderer.render().unwrap();
    }
}
