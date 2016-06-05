use std::path::Path;
use std::collections::HashSet;
use erl_ast::AST;
use erl_ast::ast;
use erl_ast::result::FromBeamResult;

#[derive(Debug)]
pub struct Module {
    pub dependent_modules: HashSet<String>,
    pub client_modules: HashSet<String>,
    pub ast: AST,
}

impl Module {
    pub fn from_beam_file<P: AsRef<Path>>(beam_file: P) -> FromBeamResult<Self> {
        let ast = try!(AST::from_beam_file(beam_file));
        let dependent_modules = collect_dependent_modules_from_ast(&ast);
        Ok(Module {
            ast: ast,
            dependent_modules: dependent_modules,
            client_modules: HashSet::new(),
        })
    }
}

fn collect_dependent_modules_from_ast(ast: &AST) -> HashSet<String> {
    let mut modules = HashSet::new();
    for f in &ast.module.forms {
        collect_dependent_modules_from_form(f, &mut modules);
    }
    modules
}

fn collect_dependent_modules_from_form(form: &ast::form::Form, modules: &mut HashSet<String>) {
    match *form {
        ast::form::Form::Import(ref i) => {
            modules.insert(i.module.clone());
        }
        ast::form::Form::Fun(ref f) => {
            for c in &f.clauses {
                collect_dependent_modules_from_clause(c, modules);
            }
        }
        _ => {}
    }
}

fn collect_dependent_modules_from_clause(clause: &ast::clause::Clause,
                                         modules: &mut HashSet<String>) {
    for e in &clause.body {
        collect_dependent_modules_from_expr(e, modules);
    }
}

// TODO: Add IterChildExpr trait
fn collect_dependent_modules_from_expr(expr: &ast::expr::Expression,
                                       modules: &mut HashSet<String>) {
    match *expr {
        ast::expr::Expression::Match(ref x) => {
            collect_dependent_modules_from_expr(&x.right, modules);
        }
        ast::expr::Expression::Tuple(ref x) => {
            for e in &x.elements {
                collect_dependent_modules_from_expr(e, modules);
            }
        }
        ast::expr::Expression::Cons(ref x) => {
            collect_dependent_modules_from_expr(&x.head, modules);
            collect_dependent_modules_from_expr(&x.tail, modules);
        }
        ast::expr::Expression::Binary(ref x) => {
            for bin_elem in &x.elements {
                collect_dependent_modules_from_expr(&bin_elem.element, modules);
            }
        }
        ast::expr::Expression::UnaryOp(ref x) => {
            collect_dependent_modules_from_expr(&x.operand, modules);
        }
        ast::expr::Expression::BinaryOp(ref x) => {
            collect_dependent_modules_from_expr(&x.left_operand, modules);
            collect_dependent_modules_from_expr(&x.right_operand, modules);
        }
        ast::expr::Expression::Record(ref x) => {
            if let Some(ref b) = x.base {
                collect_dependent_modules_from_expr(b, modules);
            }
            for f in &x.fields {
                collect_dependent_modules_from_expr(&f.value, modules);
            }
        }
        ast::expr::Expression::RecordIndex(ref x) => {
            if let Some(ref b) = x.base {
                collect_dependent_modules_from_expr(b, modules);
            }
        }
        ast::expr::Expression::Map(ref x) => {
            if let Some(ref b) = x.base {
                collect_dependent_modules_from_expr(b, modules);
            }
            for p in &x.pairs {
                collect_dependent_modules_from_expr(&p.key, modules);
                collect_dependent_modules_from_expr(&p.value, modules);
            }
        }
        ast::expr::Expression::Catch(ref x) => {
            collect_dependent_modules_from_expr(&x.expr, modules);
        }
        ast::expr::Expression::LocalCall(ref x) => {
            collect_dependent_modules_from_expr(&x.function, modules);
            for a in &x.args {
                collect_dependent_modules_from_expr(a, modules);
            }
        }
        ast::expr::Expression::RemoteCall(ref x) => {
            collect_dependent_modules_from_expr(&x.module, modules);
            if let ast::expr::Expression::Atom(ref m) = x.module {
                modules.insert(m.value.clone());
            }
            collect_dependent_modules_from_expr(&x.function, modules);
            for a in &x.args {
                collect_dependent_modules_from_expr(a, modules);
            }
        }
        ast::expr::Expression::Comprehension(ref x) => {
            collect_dependent_modules_from_expr(&x.expr, modules);
            for q in &x.qualifiers {
                match *q {
                    ast::expr::Qualifier::Generator(ref g) => {
                        collect_dependent_modules_from_expr(&g.expr, modules);
                    }
                    ast::expr::Qualifier::BitStringGenerator(ref g) => {
                        collect_dependent_modules_from_expr(&g.expr, modules);
                    }
                    ast::expr::Qualifier::Filter(ref f) => {
                        collect_dependent_modules_from_expr(&f, modules);
                    }
                }
            }
        }
        ast::expr::Expression::Block(ref x) => {
            for e in &x.body {
                collect_dependent_modules_from_expr(e, modules);
            }
        }
        ast::expr::Expression::If(ref x) => {
            for c in &x.clauses {
                collect_dependent_modules_from_clause(c, modules);
            }
        }
        ast::expr::Expression::Case(ref x) => {
            collect_dependent_modules_from_expr(&x.expr, modules);
            for c in &x.clauses {
                collect_dependent_modules_from_clause(c, modules);
            }
        }
        ast::expr::Expression::Try(ref x) => {
            for e in &x.body {
                collect_dependent_modules_from_expr(e, modules);
            }
            for e in &x.after {
                collect_dependent_modules_from_expr(e, modules);
            }
            for c in &x.case_clauses {
                collect_dependent_modules_from_clause(c, modules);
            }
            for c in &x.catch_clauses {
                collect_dependent_modules_from_clause(c, modules);
            }
        }
        ast::expr::Expression::Receive(ref x) => {
            for c in &x.clauses {
                collect_dependent_modules_from_clause(c, modules);
            }
            for e in &x.after {
                collect_dependent_modules_from_expr(e, modules);
            }
        }
        ast::expr::Expression::ExternalFun(ref x) => {
            collect_dependent_modules_from_expr(&x.module, modules);
            collect_dependent_modules_from_expr(&x.function, modules);
            collect_dependent_modules_from_expr(&x.arity, modules);
        }
        ast::expr::Expression::AnonymousFun(ref x) => {
            for c in &x.clauses {
                collect_dependent_modules_from_clause(c, modules);
            }
        }
        _ => {}
    }
}
