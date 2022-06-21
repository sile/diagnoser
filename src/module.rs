use std::path::Path;
use std::error;
use std::collections::HashSet;
use std::collections::HashMap;
use erl_ast::AST;
use erl_ast::ast;
use ty;
use ty::TypeClass;
use ast::FromAst;
use meta;

pub type Arity = u8;
pub type Result<T> = ::std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug)]
pub struct Module {
    pub name: String,
    pub behaviours: HashSet<String>,
    pub exports: HashSet<Local>,
    pub callbacks: HashSet<Local>,
    pub export_types: HashSet<Local>,
    pub imports: HashSet<Remote>,
    pub types: HashMap<Local, Box<dyn TypeClass>>,
    pub specs: HashMap<Local, Spec>,
    pub functions: HashMap<Local, meta::Function>,
}
impl Module {
    pub fn from_beam_file<P: AsRef<Path>>(beam_file: P) -> Result<Self> {
        let ast = try!(AST::from_beam_file(beam_file));
        ModuleBuilder::new().build(ast)
    }
}

#[derive(Default)]
struct ModuleBuilder {
    name: Option<String>,
    behaviours: HashSet<String>,
    exports: HashSet<Local>,
    callbacks: HashSet<Local>,
    export_types: HashSet<Local>,
    imports: HashSet<Remote>,
    types: HashMap<Local, Box<dyn TypeClass>>,
    specs: HashMap<Local, Spec>,
    functions: HashMap<Local, meta::Function>,
}
impl ModuleBuilder {
    pub fn new() -> Self {
        ModuleBuilder::default()
    }
    pub fn build(mut self, ast: AST) -> Result<Module> {
        for form in &ast.module.forms {
            try!(self.handle_form(form));
        }
        // TODO: Resolve imported functions

        let name = try!(self.name.ok_or("No `-module(...)` directive"));
        Ok(Module {
            name: name,
            behaviours: self.behaviours,
            exports: self.exports,
            callbacks: self.callbacks,
            export_types: self.export_types,
            imports: self.imports,
            types: self.types,
            specs: self.specs,
            functions: self.functions,
        })
    }
    fn handle_form(&mut self, form: &ast::form::Form) -> Result<()> {
        use erl_ast::ast::form::Form;
        match *form {
            Form::Module(ref x) => {
                self.name = Some(x.name.to_string());
            }
            Form::Behaviour(ref x) => {
                self.behaviours.insert(x.name.clone());
            }
            Form::Export(ref x) => {
                self.exports.extend(x.funs.iter().map(|f| Local::new(&f.fun, f.arity as Arity)));
            }
            Form::ExportType(ref x) => {
                self.export_types
                    .extend(x.types.iter().map(|t| Local::new(&t.typ, t.arity as Arity)));
            }
            Form::Import(ref x) => {
                self.imports.extend(x.funs
                    .iter()
                    .map(|f| Remote::new(&x.module, &f.fun, f.arity as Arity)))
            }
            Form::Type(ref x) => {
                let key = Local::new(&x.name, x.vars.len() as Arity);
                let value = FromAst::from_ast(x);
                self.types.insert(key, value);
            }
            Form::Spec(ref x) => {
                assert!(!x.types.is_empty());
                if x.module.is_some() {
                    unimplemented!();
                }
                let arity = x.types[0].args.len() as Arity;
                let key = Local::new(&x.name, arity);
                if x.is_callback {
                    self.callbacks.insert(key.clone());
                }
                let clauses = x.types
                    .iter()
                    .map(|c| {
                        assert_eq!(arity as usize, c.args.len());
                        let constraints = c.constraints
                            .iter()
                            .map(|c| {
                                assert!(!c.var.is_anonymous());
                                Constraint {
                                    var: c.var.name.clone(),
                                    subtype: FromAst::from_ast(&c.subtype),
                                }
                            })
                            .collect();
                        SpecClause {
                            args: c.args.iter().map(FromAst::from_ast).collect(),
                            return_type: FromAst::from_ast(&c.return_type),
                            constraints: constraints,
                        }
                    })
                    .collect();
                self.specs.insert(key, Spec { clauses: clauses });
            }
            Form::Record(ref _x) => {
                // TODO:
                // panic!("RECORD: {:?}", x),
            }
            Form::Fun(ref x) => {
                assert!(!x.clauses.is_empty());
                let key = Local::new(&x.name, x.clauses[0].patterns.len() as Arity);
                let value = FromAst::from_ast(x);
                self.functions.insert(key, value);
            }
            _ => {}
        }
        Ok(())
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
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

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Remote {
    pub module: String,
    pub name: String,
    pub arity: Arity,
}
impl Remote {
    pub fn new(module: &str, name: &str, arity: Arity) -> Self {
        Remote {
            module: module.to_string(),
            name: name.to_string(),
            arity: arity,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Spec {
    pub clauses: Vec<SpecClause>,
}

#[derive(Debug, Clone)]
pub struct SpecClause {
    pub args: Vec<ty::Type>,
    pub return_type: ty::Type,
    pub constraints: Vec<Constraint>,
}

#[derive(Debug, Clone)]
pub struct Constraint {
    pub var: String,
    pub subtype: ty::Type,
}
