use std::path::Path;
use erl_ast::AST;
use erl_ast::result::FromBeamResult;
use erl_ast::ast;

#[derive(Debug)]
pub struct Module {
    pub decl: ast::ModuleDecl,
}

impl Module {
    pub fn from_beam_file<P: AsRef<Path>>(beam_file: P) -> FromBeamResult<Self> {
        let ast = try!(AST::from_beam_file(beam_file));
        Ok(Module { decl: ast.module })
    }
}
