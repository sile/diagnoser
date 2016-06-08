use erl_ast::ast;
use ty;

pub trait FromAst {
    type Input;
    fn from_ast(ast: &Self::Input) -> Self;
}

impl FromAst for Box<ty::TypeClass> {
    type Input = ast::form::TypeDecl;
    fn from_ast(decl: &Self::Input) -> Self {
        Box::new(ty::UserDefinedClass {
            is_opaque: decl.is_opaque,
            name: decl.name.clone(),
            vars: decl.vars.iter().map(|v| v.name.clone()).collect(),
            body: FromAst::from_ast(&decl.ty),
        })
    }
}

impl FromAst for ty::Type {
    type Input = ast::ty::Type;
    fn from_ast(ty: &Self::Input) -> Self {
        use erl_ast::ast::ty::Type as AstType;
        match *ty {
            AstType::Atom(ref x) => ty::atom(&x.value),
            AstType::Integer(ref x) => {
                use ::num::traits::ToPrimitive;
                if let Some(v) = x.value.to_i64() {
                    From::from(ty::integer().value(v))
                } else {
                    From::from(ty::integer())
                }
            }
            AstType::Var(ref x) => From::from(ty::Var::new(&x.name)),
            AstType::Annotated(ref x) => {
                From::from(ty::Var::with_value(&x.name.name, FromAst::from_ast(&x.ty)))
            }
            AstType::UnaryOp(ref x) => {
                assert_eq!("'-'", x.operator);
                let operand = if let ty::Type::Integer(operand) = FromAst::from_ast(&x.operand) {
                    operand.get_single_value().unwrap()
                } else {
                    panic!("{:?}", x.operand);
                };
                From::from(ty::integer().value(-operand))
            }
            AstType::BinaryOp(_) => panic!("{:?}", ty),
            AstType::BitString(_) => panic!("{:?}", ty),
            AstType::Nil(_) => From::from(ty::NilType),
            AstType::AnyFun(_) => From::from(ty::FunType::any()),
            AstType::Function(ref x) => {
                assert!(x.constraints.is_empty());
                From::from(ty::FunType {
                    clauses: vec![ty::FunSpec {
                                      args: Some(x.args.iter().map(FromAst::from_ast).collect()),
                                      return_type: FromAst::from_ast(&x.return_type),
                                  }],
                })
            }
            AstType::Range(ref x) => {
                let mut range = ty::integer();
                if let ty::Type::Integer(low) = FromAst::from_ast(&x.low) {
                    if !low.is_any() {
                        range = range.min(low.get_single_value().unwrap());
                    }
                } else {
                    panic!("{:?}", x.low);
                };
                if let ty::Type::Integer(high) = FromAst::from_ast(&x.high) {
                    if !high.is_any() {
                        range = range.max(high.get_single_value().unwrap());
                    }
                } else {
                    panic!("{:?}", x.high);
                };
                From::from(range)
            }
            AstType::Map(ref x) => {
                From::from(ty::MapType {
                    pairs: x.pairs
                        .iter()
                        .map(|p| {
                            ty::MapPair {
                                key: FromAst::from_ast(&p.key),
                                value: FromAst::from_ast(&p.value),
                            }
                        })
                        .collect(),
                })
            }
            AstType::BuiltIn(ref x) => {
                From::from(ty::builtin(&x.name,
                                       &x.args
                                           .iter()
                                           .map(FromAst::from_ast)
                                           .collect::<Vec<_>>()))
            }
            AstType::Record(ref x) => {
                From::from(ty::RecordType {
                    name: x.name.clone(),
                    fields: x.fields
                        .iter()
                        .map(|f| {
                            ty::RecordField {
                                name: f.name.clone(),
                                value: FromAst::from_ast(&f.ty),
                            }
                        })
                        .collect(),
                })
            }
            AstType::Remote(ref x) => {
                ty::remote(&x.module,
                           &x.function,
                           &x.args.iter().map(FromAst::from_ast).collect::<Vec<_>>())
            }
            AstType::AnyTuple(_) => From::from(ty::TupleType::any()),
            AstType::Tuple(ref x) => {
                From::from(ty::TupleType {
                    elements: Some(x.elements.iter().map(FromAst::from_ast).collect()),
                })
            }
            AstType::Union(ref x) => {
                From::from(ty::UnionType::new(x.types
                    .iter()
                    .map(FromAst::from_ast)
                    .collect()))
            }
            AstType::User(ref x) => {
                ty::local(&x.name,
                          &x.args.iter().map(FromAst::from_ast).collect::<Vec<_>>())
            }
        }
    }
}
