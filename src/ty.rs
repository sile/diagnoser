//! See: [Types and Function Specifications](http://erlang.org/doc/reference_manual/typespec.html)
#![allow(unused_variables)]
use std::collections::HashMap;
use std::fmt;
use std::fmt::Debug;

pub trait ProtoType: Clone {}

pub trait TypeClass: Debug {
    fn make_instance(&self, args: &[Type]) -> Type;
}
impl<T> TypeClass for T
    where T: ProtoType + Debug,
          Type: From<T>
{
    fn make_instance(&self, args: &[Type]) -> Type {
        assert!(args.is_empty());
        From::from(self.clone())
    }
}

#[derive(Debug, Clone)]
pub enum Type {
    Any(Box<AnyType>),
    None(Box<NoneType>),
    Pid(Box<PidType>),
    Port(Box<PortType>),
    Reference(Box<ReferenceType>),
    Nil(Box<NilType>),
    Atom(Box<AtomType>),
    Bitstring(Box<BitstringType>),
    Float(Box<FloatType>),
    Fun(Box<FunType>),
    Integer(Box<IntegerType>),
    List(Box<ListType>),
    Map(Box<MapType>),
    Record(Box<RecordType>),
    Tuple(Box<TupleType>),
    Union(Box<UnionType>),
    UserDefined(Box<UserDefinedType>),
    Local(Box<LocalType>),
    Remote(Box<RemoteType>),
    Var(Box<Var>),
}
impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Type::Any(ref x) => write!(f, "{}", x),
            Type::None(ref x) => write!(f, "{}", x),
            Type::Pid(ref x) => write!(f, "{}", x),
            Type::Port(ref x) => write!(f, "{}", x),
            Type::Reference(ref x) => write!(f, "{}", x),
            Type::Nil(ref x) => write!(f, "{}", x),
            Type::Atom(ref x) => write!(f, "{}", x),
            Type::Bitstring(ref x) => write!(f, "{}", x),
            Type::Float(ref x) => write!(f, "{}", x),
            Type::Fun(ref x) => write!(f, "{}", x),
            Type::Integer(ref x) => write!(f, "{}", x),
            Type::List(ref x) => write!(f, "{}", x),
            Type::Map(ref x) => write!(f, "{}", x),
            Type::Record(ref x) => write!(f, "{}", x),
            Type::Tuple(ref x) => write!(f, "{}", x),
            Type::Union(ref x) => write!(f, "{}", x),
            Type::UserDefined(ref x) => write!(f, "{}", x),
            Type::Local(ref x) => write!(f, "{}", x),
            Type::Remote(ref x) => write!(f, "{}", x),
            Type::Var(ref x) => write!(f, "{}", x),
        }
    }
}
macro_rules! impl_from {
    ($to:ident :: $cons:ident ( $from:ty )) => {
        impl ::std::convert::From<$from> for $to {
            fn from(x: $from) -> Self {
                $to::$cons(Box::new(x))
            }
        }
    }
}
impl_from!(Type::Any(AnyType));
impl_from!(Type::None(NoneType));
impl_from!(Type::Pid(PidType));
impl_from!(Type::Port(PortType));
impl_from!(Type::Reference(ReferenceType));
impl_from!(Type::Nil(NilType));
impl_from!(Type::Atom(AtomType));
impl_from!(Type::Bitstring(BitstringType));
impl_from!(Type::Float(FloatType));
impl_from!(Type::Fun(FunType));
impl_from!(Type::Integer(IntegerType));
impl_from!(Type::List(ListType));
impl_from!(Type::Map(MapType));
impl_from!(Type::Record(RecordType));
impl_from!(Type::Tuple(TupleType));
impl_from!(Type::Union(UnionType));
impl_from!(Type::UserDefined(UserDefinedType));
impl_from!(Type::Var(Var));
impl_from!(Type::Local(LocalType));
impl_from!(Type::Remote(RemoteType));
impl ProtoType for Type {}
impl Type {
    pub fn bind(&self, bindings: HashMap<String, Type>) -> Type {
        unimplemented!()
    }
    pub fn normalize(&self) -> Type {
        // TODO:
        self.clone()
    }
}

#[derive(Debug, Clone)]
pub struct AnyType;
impl ProtoType for AnyType {}
impl fmt::Display for AnyType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "any()")
    }
}

#[derive(Debug, Clone)]
pub struct NoneType;
impl ProtoType for NoneType {}
impl fmt::Display for NoneType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "none()")
    }
}

#[derive(Debug, Clone)]
pub struct PidType;
impl ProtoType for PidType {}
impl fmt::Display for PidType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "pid()")
    }
}

#[derive(Debug, Clone)]
pub struct PortType;
impl ProtoType for PortType {}
impl fmt::Display for PortType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "port()")
    }
}

#[derive(Debug, Clone)]
pub struct ReferenceType;
impl ProtoType for ReferenceType {}
impl fmt::Display for ReferenceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "reference()")
    }
}

#[derive(Debug, Clone)]
pub struct NilType;
impl ProtoType for NilType {}
impl fmt::Display for NilType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[]")
    }
}

#[derive(Debug, Clone)]
pub struct Var {
    pub name: String,
    pub value: Option<Type>,
}
impl Var {
    pub fn new(name: &str) -> Self {
        Var {
            name: name.to_string(),
            value: None,
        }
    }
    pub fn with_value(name: &str, value: Type) -> Self {
        Var {
            name: name.to_string(),
            value: Some(value),
        }
    }
}
impl fmt::Display for Var {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Clone)]
pub struct AtomType {
    pub value: Option<String>, // `None` means "any atoms"
}
impl ProtoType for AtomType {}
impl AtomType {
    pub fn new(value: &str) -> Self {
        AtomType { value: Some(value.to_string()) }
    }
    pub fn any() -> Self {
        AtomType { value: None }
    }
}
impl fmt::Display for AtomType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref name) = self.value {
            write!(f, "'{}'", name)
        } else {
            write!(f, "atom()")
        }
    }
}
pub fn atom(name: &str) -> Type {
    Type::from(AtomType::new(name))
}

#[derive(Default)]
#[derive(Debug, Clone)]
pub struct BitstringType {
    pub bits: Option<usize>,
    pub align: Option<usize>,
}
impl BitstringType {
    pub fn bits(mut self, bits: usize) -> Self {
        self.bits = Some(bits);
        self
    }
    pub fn align(mut self, align: usize) -> Self {
        self.align = Some(align);
        self
    }
}
impl ProtoType for BitstringType {}
impl fmt::Display for BitstringType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match (self.bits, self.align) {
            (None, None) => write!(f, "<<>>"),
            (Some(bits), None) => write!(f, "<<_:{}>>", bits),
            (None, Some(align)) => write!(f, "<<_:_*{}>>", align),
            (Some(bits), Some(align)) => write!(f, "<<_:{}, _:_*{}>>", bits, align),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FloatType;
impl ProtoType for FloatType {}
impl fmt::Display for FloatType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "float()")
    }
}

#[derive(Debug, Clone)]
pub struct FunType {
    pub clauses: Vec<FunSpec>,
}
impl ProtoType for FunType {}
impl FunType {
    pub fn any() -> Self {
        FunType { clauses: Vec::new() }
    }
}
impl fmt::Display for FunType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.clauses.is_empty() {
            write!(f, "fun()")
        } else {
            assert_eq!(1, self.clauses.len());
            let c = &self.clauses[0];
            if let Some(ref args) = c.args {
                try!(write!(f, "fun(("));
                for (i, a) in args.iter().enumerate() {
                    if i > 0 {
                        try!(write!(f, ","));
                    }
                    try!(write!(f, "{}", a));
                }
                try!(write!(f, ") -> {})", c.return_type));
                Ok(())
            } else {
                write!(f, "fun((...) -> {})", c.return_type)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct FunSpec {
    pub args: Option<Vec<Type>>,
    pub return_type: Type,
}

#[derive(Debug, Clone)]
pub struct IntegerType {
    pub min: Option<i64>, // `None` means "infinity"
    pub max: Option<i64>,
}
impl ProtoType for IntegerType {}
impl IntegerType {
    pub fn min(mut self, value: i64) -> Self {
        self.min = Some(value);
        self
    }
    pub fn max(mut self, value: i64) -> Self {
        self.max = Some(value);
        self
    }
    pub fn value(self, value: i64) -> Self {
        self.min(value).max(value)
    }
    pub fn get_single_value(&self) -> Option<i64> {
        if self.min == self.max {
            self.min
        } else {
            None
        }
    }
    pub fn is_any(&self) -> bool {
        self.min.is_none() && self.max.is_none()
    }
}
impl fmt::Display for IntegerType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match (self.min, self.max) {
            (Some(min), Some(max)) if min != max => write!(f, "{}..{}", min, max),
            (Some(value), Some(_)) => write!(f, "{}", value),
            _ => write!(f, "integer()"),
        }
    }
}
pub fn integer() -> IntegerType {
    IntegerType {
        min: None,
        max: None,
    }
}

#[derive(Debug, Clone)]
pub enum ListType {
    Proper(ProperListType),
    MaybeImproper(MaybeImproperListType),
    NonEmpty(NonEmptyListType),
    NonEmptyImproper(NonEmptyImproperListType),
}
impl fmt::Display for ListType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ListType::Proper(ref x) => write!(f, "{}", x),
            ListType::MaybeImproper(ref x) => write!(f, "{}", x),
            ListType::NonEmpty(ref x) => write!(f, "{}", x),
            ListType::NonEmptyImproper(ref x) => write!(f, "{}", x),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProperListType {
    pub element: Type,
}
impl fmt::Display for ProperListType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "list({})", self.element)
    }
}

#[derive(Debug, Clone)]
pub struct ProperListClass;
impl TypeClass for ProperListClass {
    fn make_instance(&self, args: &[Type]) -> Type {
        assert_eq!(1, args.len());
        From::from(ListType::Proper(ProperListType { element: args[0].clone() }))
    }
}

#[derive(Debug, Clone)]
pub struct MaybeImproperListType {
    pub element: Type,
    pub last: Type,
}
impl fmt::Display for MaybeImproperListType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "maybe_improper_list({},{})", self.element, self.last)
    }
}

#[derive(Debug, Clone)]
pub struct MaybeImproperListClass;
impl TypeClass for MaybeImproperListClass {
    fn make_instance(&self, args: &[Type]) -> Type {
        assert_eq!(2, args.len());
        From::from(ListType::MaybeImproper(MaybeImproperListType {
            element: args[0].clone(),
            last: args[1].clone(),
        }))
    }
}

#[derive(Debug, Clone)]
pub struct NonEmptyListType {
    pub element: Type,
}
impl fmt::Display for NonEmptyListType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "nonempty_list({})", self.element)
    }
}

#[derive(Debug, Clone)]
pub struct NonEmptyListClass;
impl TypeClass for NonEmptyListClass {
    fn make_instance(&self, args: &[Type]) -> Type {
        assert_eq!(1, args.len());
        From::from(ListType::NonEmpty(NonEmptyListType { element: args[0].clone() }))
    }
}

#[derive(Debug, Clone)]
pub struct NonEmptyImproperListType {
    pub element: Type,
    pub last: Type,
}
impl fmt::Display for NonEmptyImproperListType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "nonempty_improper_list({},{})", self.element, self.last)
    }
}

#[derive(Debug, Clone)]
pub struct NonEmptyImproperListClass;
impl TypeClass for NonEmptyImproperListClass {
    fn make_instance(&self, args: &[Type]) -> Type {
        assert_eq!(2, args.len());
        From::from(ListType::NonEmptyImproper(NonEmptyImproperListType {
            element: args[0].clone(),
            last: args[1].clone(),
        }))
    }
}

#[derive(Debug, Clone)]
pub struct MapType {
    pub pairs: Vec<MapPair>,
}
impl ProtoType for MapType {}
impl MapType {
    pub fn any() -> Self {
        MapType { pairs: Vec::new() }
    }
}
impl fmt::Display for MapType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "#{{"));
        for (i, p) in self.pairs.iter().enumerate() {
            if i > 0 {
                try!(write!(f, ","));
            }
            try!(write!(f, "{}=>{}", p.key, p.value));
        }
        try!(write!(f, "}}"));
        Ok(())
    }
}
#[derive(Debug, Clone)]
pub struct MapPair {
    pub key: Type,
    pub value: Type,
}

#[derive(Debug, Clone)]
pub struct RecordType {
    pub name: String,
    pub fields: Vec<RecordField>,
}
impl ProtoType for RecordType {}
impl fmt::Display for RecordType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "#{}{{", self.name));
        for (i, p) in self.fields.iter().enumerate() {
            if i > 0 {
                try!(write!(f, ","));
            }
            try!(write!(f, "{} :: {}", p.name, p.value));
        }
        try!(write!(f, "}}"));
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct RecordField {
    pub name: String,
    pub value: Type,
}

#[derive(Debug, Clone)]
pub struct TupleType {
    pub elements: Option<Vec<Type>>,
}
impl ProtoType for TupleType {}
impl fmt::Display for TupleType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref elements) = self.elements {
            try!(write!(f, "{{"));
            for (i, e) in elements.iter().enumerate() {
                if i > 0 {
                    try!(write!(f, ","));
                }
                try!(write!(f, "{}", e));
            }
            try!(write!(f, "}}"));
            Ok(())
        } else {
            write!(f, "tuple()")
        }
    }
}
impl TupleType {
    pub fn any() -> Self {
        TupleType { elements: None }
    }
}
pub fn tuple3(t0: Type, t1: Type, t2: Type) -> TupleType {
    TupleType { elements: Some(vec![t0, t1, t2]) }
}

#[derive(Debug, Clone)]
pub struct UnionType {
    pub types: Vec<Type>,
}
impl ProtoType for UnionType {}
impl fmt::Display for UnionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, t) in self.types.iter().enumerate() {
            if i > 0 {
                try!(write!(f, "|"));
            }
            try!(write!(f, "{}", t));
        }
        Ok(())
    }
}
impl UnionType {
    pub fn new(types: Vec<Type>) -> Self {
        UnionType { types: types }
    }
}
pub fn union(types: &[Type]) -> Type {
    From::from(UnionType::new(Vec::from(types)))
}

#[derive(Debug, Clone)]
pub struct UserDefinedType {
    pub is_opaque: bool,
    pub name: String,
    pub body: Type,
}
impl fmt::Display for UserDefinedType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}()", self.name)
    }
}

#[derive(Debug, Clone)]
pub struct UserDefinedClass {
    pub is_opaque: bool,
    pub name: String,
    pub vars: Vec<String>,
    pub body: Type,
}
impl TypeClass for UserDefinedClass {
    fn make_instance(&self, args: &[Type]) -> Type {
        use std::iter::FromIterator;
        assert_eq!(self.vars.len(), args.len());
        // TODO: Handles anonymous variable
        let bindings = HashMap::from_iter(self.vars.iter().cloned().zip(args.iter().cloned()));
        let ty = UserDefinedType {
            is_opaque: self.is_opaque,
            name: self.name.clone(),
            body: self.body.bind(bindings),
        };
        From::from(ty)
    }
}

#[derive(Debug, Clone)]
pub struct LocalType {
    pub name: String,
    pub args: Vec<Type>,
}
impl ProtoType for LocalType {}
impl fmt::Display for LocalType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: Show arity
        write!(f, "{}()", self.name)
    }
}
pub fn local(name: &str, args: &[Type]) -> Type {
    From::from(LocalType {
        name: name.to_string(),
        args: Vec::from(args),
    })
}
pub fn builtin(name: &str, args: &[Type]) -> Type {
    local(name, args)
}
pub fn builtin0(name: &str) -> Type {
    local(name, &[])
}
pub fn builtin1(name: &str, a0: Type) -> Type {
    local(name, &[a0])
}
pub fn builtin2(name: &str, a0: Type, a1: Type) -> Type {
    local(name, &[a0, a1])
}

#[derive(Debug, Clone)]
pub struct RemoteType {
    pub module: String,
    pub name: String,
    pub args: Vec<Type>,
}
impl fmt::Display for RemoteType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: Show arity
        write!(f, "{}:{}()", self.module, self.name)
    }
}
pub fn remote(module: &str, name: &str, args: &[Type]) -> Type {
    From::from(RemoteType {
        module: module.to_string(),
        name: name.to_string(),
        args: Vec::from(args),
    })
}
