use std::iter::FromIterator;
use std::collections::HashMap;
use erl_ast::ast;
use erl_type;
use erl_type::Type;
use erl_type::TypeClass;
use erl_type::FunSpec;
use beam::Module;

#[derive(Hash, PartialEq, Eq, Debug)]
pub struct TypeKey {
    pub module: Option<String>, // `None` means "a built-in type"
    pub name: String,
    pub arity: u8,
}
impl TypeKey {
    pub fn builtin(name: &str, arity: u8) -> Self {
        TypeKey {
            module: None,
            name: name.to_string(),
            arity: arity,
        }
    }
    pub fn remote(module: &str, name: &str, arity: u8) -> Self {
        TypeKey {
            module: Some(module.to_string()),
            name: name.to_string(),
            arity: arity,
        }
    }
}

#[derive(Hash, PartialEq, Eq, Debug)]
pub struct SpecKey {
    pub module: String,
    pub function: String,
    pub arity: u8,
}

pub struct Env {
    pub modules: Vec<Module>, // TODO: HashMap<String, Module>
    pub types: HashMap<TypeKey, Box<TypeClass>>,
    pub specs: HashMap<SpecKey, FunSpec>, // TODO: => ftypes(?)
}
impl Env {
    pub fn new() -> Self {
        let types = HashMap::from_iter(built_in_types().into_iter());
        Env {
            modules: Vec::new(),
            types: types,
            specs: HashMap::new(),
        }
    }
    pub fn add_module(&mut self, module: Module) {
        self.load_types(&module);
        self.load_specs(&module);
        self.modules.push(module);
    }
    pub fn load_types(&mut self, module: &Module) {
        // TODO: Add Form::get_module()
        let module_name = module.ast
            .module
            .forms
            .iter()
            .filter_map(|f| {
                if let ast::form::Form::Module(ref m) = *f {
                    Some(m.name.to_string())
                } else {
                    None
                }
            })
            .nth(0)
            .unwrap();
        for f in &module.ast.module.forms {
            let decl = if let ast::form::Form::Type(ref decl) = *f {
                decl
            } else {
                continue;
            };
            let key = TypeKey::remote(&module_name, &decl.name, decl.vars.len() as u8);
            let value = type_decl_to_type_class(decl);
            self.types.insert(key, value);
        }
    }
    pub fn load_specs(&mut self, module: &Module) {
        // NOTE: We assume that functions which have
        // no spec are typed with `-spec Fun(...) -> any()`
        unimplemented!()
    }
}

fn type_decl_to_type_class(decl: &ast::form::TypeDecl) -> Box<TypeClass> {
    Box::new(erl_type::UserDefinedClass {
        is_opaque: decl.is_opaque,
        name: decl.name.clone(),
        vars: decl.vars.iter().map(|v| v.name.clone()).collect(),
        body: ast_type_to_erl_type(&decl.ty),
    })
}
fn ast_type_to_erl_type(ty: &ast::ty::Type) -> Type {
    use erl_ast::ast::ty;
    match *ty {
        ty::Type::Atom(ref x) => erl_type::atom(&x.value),
        ty::Type::Integer(ref x) => {
            use ::num::traits::ToPrimitive;
            if let Some(v) = x.value.to_i64() {
                From::from(erl_type::integer().value(v))
            } else {
                From::from(erl_type::integer())
            }
        }
        ty::Type::Var(ref x) => From::from(erl_type::Var::new(&x.name)),
        ty::Type::Annotated(ref x) => {
            From::from(erl_type::Var::with_value(&x.name.name, ast_type_to_erl_type(&x.ty)))
        }
        ty::Type::UnaryOp(ref x) => {
            assert_eq!("'-'", x.operator);
            let operand = if let erl_type::Type::Integer(operand) =
                                 ast_type_to_erl_type(&x.operand) {
                operand.get_single_value().unwrap()
            } else {
                panic!("{:?}", x.operand);
            };
            From::from(erl_type::integer().value(-operand))
        }
        ty::Type::BinaryOp(ref x) => panic!("{:?}", ty),
        ty::Type::BitString(ref x) => panic!("{:?}", ty),
        ty::Type::Nil(ref x) => From::from(erl_type::NilType),
        ty::Type::AnyFun(ref x) => From::from(erl_type::FunType::any()),
        ty::Type::Function(ref x) => {
            assert!(x.constraints.is_empty());
            From::from(erl_type::FunType {
                spec: Some(erl_type::FunSpec {
                    args: Some(x.args.iter().map(ast_type_to_erl_type).collect()),
                    return_type: ast_type_to_erl_type(&x.return_type),
                }),
            })
        }
        ty::Type::Range(ref x) => {
            let mut range = erl_type::integer();
            if let erl_type::Type::Integer(low) = ast_type_to_erl_type(&x.low) {
                if !low.is_any() {
                    range = range.min(low.get_single_value().unwrap());
                }
            } else {
                panic!("{:?}", x.low);
            };
            let high = if let erl_type::Type::Integer(high) = ast_type_to_erl_type(&x.high) {
                if !high.is_any() {
                    range = range.max(high.get_single_value().unwrap());
                }
            } else {
                panic!("{:?}", x.high);
            };
            From::from(range)
        }
        ty::Type::Map(ref x) => {
            From::from(erl_type::MapType {
                pairs: x.pairs
                    .iter()
                    .map(|p| {
                        erl_type::MapPair {
                            key: ast_type_to_erl_type(&p.key),
                            value: ast_type_to_erl_type(&p.value),
                        }
                    })
                    .collect(),
            })
        }
        ty::Type::BuiltIn(ref x) => {
            From::from(erl_type::builtin(&x.name,
                                         &x.args
                                             .iter()
                                             .map(ast_type_to_erl_type)
                                             .collect::<Vec<_>>()))
        }
        ty::Type::Record(ref x) => {
            Type::from(erl_type::RecordType {
                name: x.name.clone(),
                fields: x.fields
                    .iter()
                    .map(|f| {
                        erl_type::RecordField {
                            name: f.name.clone(),
                            value: ast_type_to_erl_type(&f.ty),
                        }
                    })
                    .collect(),
            })
        }
        ty::Type::Remote(ref x) => {
            erl_type::remote(&x.module,
                             &x.function,
                             &x.args.iter().map(ast_type_to_erl_type).collect::<Vec<_>>())
        }
        ty::Type::AnyTuple(ref x) => From::from(erl_type::TupleType::any()),
        ty::Type::Tuple(ref x) => {
            From::from(erl_type::TupleType {
                elements: Some(x.elements.iter().map(ast_type_to_erl_type).collect()),
            })
        }
        ty::Type::Union(ref x) => {
            From::from(erl_type::UnionType::new(x.types
                .iter()
                .map(ast_type_to_erl_type)
                .collect()))
        }
        ty::Type::User(ref x) => {
            erl_type::local(&x.name,
                            &x.args.iter().map(ast_type_to_erl_type).collect::<Vec<_>>())
        }
    }
}

pub trait WithType {
    fn use_type(&self) -> &Type;
    fn allow_type(&self) -> &Type;
}

pub fn built_in_types() -> Vec<(TypeKey, Box<TypeClass>)> {
    use erl_type::*;
    fn a0(name: &str) -> TypeKey {
        TypeKey::builtin(name, 0)
    }
    fn a1(name: &str) -> TypeKey {
        TypeKey::builtin(name, 1)
    }
    fn a2(name: &str) -> TypeKey {
        TypeKey::builtin(name, 1)
    }
    vec![(a0("any"), Box::new(AnyType)),
         (a0("none"), Box::new(NoneType)),
         (a0("pid"), Box::new(PidType)),
         (a0("port"), Box::new(PortType)),
         (a0("reference"), Box::new(ReferenceType)),
         (a0("nil"), Box::new(NilType)),
         (a0("atom"), Box::new(AtomType::any())),
         (a0("float"), Box::new(FloatType)),
         (a0("fun"), Box::new(FunType::any())),
         (a0("integer"), Box::new(integer())),
         (a1("list"), Box::new(ProperListClass)),
         (a2("maybe_improper_list"), Box::new(MaybeImproperListClass)),
         (a2("nonempty_improper_list"), Box::new(NonEmptyImproperListClass)),
         (a1("nonempty_list"), Box::new(NonEmptyListClass)),
         (a0("map"), Box::new(MapType::any())),
         (a0("tuple"), Box::new(TupleType::any())),
         (a0("non_neg_integer"), Box::new(integer().min(0))),
         (a0("pos_integer"), Box::new(integer().min(1))),
         (a0("neg_integer"), Box::new(integer().max(-1))),

         (a0("term"), Box::new(builtin0("any"))),
         (a0("binary"), Box::new(BitstringType::default().align(8))),
         (a0("bitstring"), Box::new(BitstringType::default().align(1))),
         (a0("boolean"), Box::new(union(&[atom("true"), atom("false")]))),
         (a0("byte"), Box::new(integer().min(0).max(255))),
         (a0("char"), Box::new(integer().min(0).max(0x10ffff))),
         (a0("number"), Box::new(union(&[builtin0("integer"), builtin0("float")]))),
         (a0("list"), Box::new(builtin1("list", builtin0("any")))),
         (a0("maybe_improper_list"),
          Box::new(builtin2("maybe_improper_list", builtin0("any"), builtin0("any")))),
         (a0("nonempty_list"), Box::new(builtin1("nonempty_list", builtin0("any")))),
         (a0("string"), Box::new(builtin1("list", builtin0("char")))),
         (a0("nonempty_string"), Box::new(builtin1("nonempty_list", builtin0("char")))),
         (a0("iodata"), Box::new(union(&[builtin0("iolist"), builtin0("binary")]))),
         (a0("iolist"),
          Box::new(builtin1("maybe_improper_list",
                            union(&[builtin0("byte"),
                                    builtin0("binary"),
                                    builtin0("iolist"),
                                    builtin0("binary"),
                                    builtin0("nil")])))),
         (a0("function"), Box::new(builtin0("fun"))),
         (a0("module"), Box::new(builtin0("atom"))),
         (a0("mfa"), Box::new(tuple3(builtin0("module"), builtin0("atom"), builtin0("arity")))),
         (a0("arity"), Box::new(integer().min(0).max(255))),
         (a0("identifier"),
          Box::new(union(&[builtin0("pid"), builtin0("port"), builtin0("reference")]))),
         (a0("node"), Box::new(builtin0("atom"))),
         (a0("timeout"), Box::new(union(&[atom("infinity"), builtin0("non_neg_integer")]))),
         (a0("no_return"), Box::new(NoneType))]
}
