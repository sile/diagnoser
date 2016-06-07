//! See: [Types and Function Specifications](http://erlang.org/doc/reference_manual/typespec.html)
use std::collections::HashMap;

pub trait ProtoType: Clone {}

pub trait TypeClass {
    fn make_instance(&self, args: &[Type]) -> Type;
}
impl<T> TypeClass for T
    where T: ProtoType,
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
}

#[derive(Debug, Clone)]
pub struct AnyType;
impl ProtoType for AnyType {}

#[derive(Debug, Clone)]
pub struct NoneType;
impl ProtoType for NoneType {}

#[derive(Debug, Clone)]
pub struct PidType;
impl ProtoType for PidType {}

#[derive(Debug, Clone)]
pub struct PortType;
impl ProtoType for PortType {}

#[derive(Debug, Clone)]
pub struct ReferenceType;
impl ProtoType for ReferenceType {}

#[derive(Debug, Clone)]
pub struct NilType;
impl ProtoType for NilType {}

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

#[derive(Debug, Clone)]
pub struct FloatType;
impl ProtoType for FloatType {}

#[derive(Debug, Clone)]
pub struct FunType {
    pub spec: Option<FunSpec>,
}
impl ProtoType for FunType {}
impl FunType {
    pub fn any() -> Self {
        FunType { spec: None }
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
    pub fn value(mut self, value: i64) -> Self {
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

#[derive(Debug, Clone)]
pub struct ProperListType {
    pub element: Type,
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

#[derive(Debug, Clone)]
pub struct RecordType {
    pub name: String,
    pub fields: Vec<RecordField>,
}
impl ProtoType for RecordType {}

#[derive(Debug, Clone)]
pub struct RecordField {
    pub name: String,
    pub value: Type,
}

#[derive(Debug, Clone)]
pub struct MapPair {
    pub key: Type,
    pub value: Type,
}

#[derive(Debug, Clone)]
pub struct TupleType {
    pub elements: Option<Vec<Type>>,
}
impl ProtoType for TupleType {}
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
pub fn remote(module: &str, name: &str, args: &[Type]) -> Type {
    From::from(RemoteType {
        module: module.to_string(),
        name: name.to_string(),
        args: Vec::from(args),
    })
}
