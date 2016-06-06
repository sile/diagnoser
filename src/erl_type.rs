//! See: [Types and Function Specifications](http://erlang.org/doc/reference_manual/typespec.html)
use std::collections::HashSet;
use std::ops::Range;

pub struct TypeDictionary;
impl TypeDictionary {
    pub fn new() -> Self {
        // TODO: Add built-in functions
        TypeDictionary
    }
    pub fn get(&self, name: String, args: Vec<String>) -> Option<&Type> {
        unimplemented!()
    }
}

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
    Tuple(Box<TupleType>),
    Union(Box<UnionType>),
    UserDefined(Box<UserDefinedType>),
    Var(Box<Var>),
}

pub struct AnyType;
pub struct NoneType;
pub struct PidType;
pub struct PortType;
pub struct ReferenceType;
pub struct NilType;

pub struct Var {
    pub name: String,
    pub value: Option<Type>,
}

pub struct AtomType {
    pub members: Option<HashSet<String>>, // `None` means "any atoms"
}

pub struct BitstringType {
    pub bytes: Option<usize>,
    pub bits: Option<usize>,
}

pub struct FloatType;

pub struct FunType {
    pub spec: Option<FunSpec>,
}

pub struct FunSpec {
    pub args: Option<Vec<Type>>,
    pub return_type: Type,
}

pub struct IntegerType {
    pub ranges: Option<Vec<Range<i64>>>,
}

pub enum ListType {
    Proper(ProperListType),
    MaybeImproper(MaybeImproperListType),
    NonEmpty(NonEmptyListType),
    NonEmptyImproper(NonEmptyImproperListType),
}

pub struct ProperListType {
    pub element: Type,
}

pub struct MaybeImproperListType {
    pub element: Type,
    pub last: Type,
}

pub struct NonEmptyListType {
    pub element: Type,
}

pub struct NonEmptyImproperListType {
    pub element: Type,
    pub last: Type,
}

pub struct MapType {
    pub pairs: Vec<MapPair>,
}

pub struct MapPair {
    pub key: Type,
    pub value: Type,
}

pub struct TupleType {
    pub elements: Option<Vec<Type>>,
}

pub struct UnionType {
    pub types: Vec<Type>,
}

pub struct UserDefinedType {
    pub is_opaque: bool,
    pub name: String,
    pub vars: Vec<String>,
    pub body: Type,
}
