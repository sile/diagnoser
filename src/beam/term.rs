use std::fmt;
use std::fmt::Display;
use std::rc::Rc;
use num::bigint::BigInt;
use num::traits::FromPrimitive;
use num::traits::ToPrimitive;

pub type Arity = usize;

#[derive(Debug,PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub enum Term {
    Atom(Atom),
    Tuple(Tuple),
    List(List),
    Nil(Nil),
    Integer(Integer),
}
impl Term {
    pub fn new_tuple(elements: Vec<Rc<Term>>) -> Self {
        Term::Tuple(Tuple::new(elements))
    }
    pub fn new_atom(name: String) -> Self {
        Term::Atom(Atom::new(name))
    }
    pub fn new_list(head: Rc<Term>, tail: Rc<Term>) -> Self {
        Term::List(List::new(head, tail))
    }
    pub fn new_integer_from_u64(x: u64) -> Self {
        Term::Integer(Integer::from_u64(x))
    }
    pub fn new_integer_from_i64(x: i64) -> Self {
        Term::Integer(Integer::from_i64(x))
    }
    pub fn new_nil() -> Self {
        Term::Nil(Nil)
    }

    pub fn as_ref_term_level0(&self) -> RefTerm<&Term> {
        RefTerm0::new(self)
    }
    pub fn as_ref_term_level1(&self) -> RefTerm<RefTerm0> {
        RefTerm1::new(self)
    }
    pub fn as_ref_term_level2(&self) -> RefTerm<RefTerm1> {
        RefTerm2::new(self)
    }
}
impl Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use self::Term::*;
        match *self {
            Atom(ref x) => x.fmt(f),
            Tuple(ref x) => x.fmt(f),
            List(ref x) => x.fmt(f),
            Integer(ref x) => x.fmt(f),
            Nil(ref x) => x.fmt(f),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct Nil;
impl Display for Nil {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "[]")
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct Integer {
    pub value: BigInt,
}
impl Integer {
    pub fn from_u64(x: u64) -> Self {
        Integer { value: FromPrimitive::from_u64(x).unwrap() }
    }
    pub fn from_i64(x: i64) -> Self {
        Integer { value: FromPrimitive::from_i64(x).unwrap() }
    }
}
impl Display for Integer {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.value.fmt(f)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct List {
    pub head: Rc<Term>,
    pub tail: Rc<Term>,
}
impl List {
    pub fn new(head: Rc<Term>, tail: Rc<Term>) -> Self {
        List {
            head: head,
            tail: tail,
        }
    }
    fn fmt_elements(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        try!(write!(f, "{}", self.head));
        match *self.tail {
            Term::List(ref list) => {
                try!(write!(f, ","));
                list.fmt_elements(f)
            }
            Term::Nil(_) => Ok(()),
            _ => write!(f, "|{}", self.tail),
        }
    }
}
impl Display for List {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        try!(write!(f, "["));
        try!(self.fmt_elements(f));
        try!(write!(f, "]"));
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct Atom {
    pub name: String,
}
impl Atom {
    pub fn new(name: String) -> Self {
        Atom { name: name }
    }
}
impl Display for Atom {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        // TODO: Quotes characters if needed
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct Tuple {
    pub elements: Vec<Rc<Term>>,
}
impl Tuple {
    pub fn new(elements: Vec<Rc<Term>>) -> Self {
        Tuple { elements: elements }
    }
}
impl Display for Tuple {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        try!(write!(f, "{{"));
        let mut is_first = true;
        for e in &self.elements {
            if !is_first {
                try!(write!(f, ","));
            }
            try!(write!(f, "{}", e));
            is_first = false;
        }
        try!(write!(f, "}}"));
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct ExternalFun {
    pub module: Atom,
    pub function: Atom,
    pub arity: Arity,
}
impl ExternalFun {
    pub fn new(module: Atom, function: Atom, arity: Arity) -> Self {
        ExternalFun {
            module: module,
            function: function,
            arity: arity,
        }
    }
}
impl Display for ExternalFun {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "fun {}:{}/{}", self.module, self.function, self.arity)
    }
}


// TODO: Moves to other module
pub trait Child<'a> {
    type Result;
    fn new(&'a Term) -> Self::Result;
}

pub enum RefTerm<'a, T: Child<'a>> {
    Atom(&'a str),
    Tuple1((T::Result), &'a Tuple),
    Tuple2((T::Result, T::Result), &'a Tuple),
    Tuple3((T::Result, T::Result, T::Result), &'a Tuple),
    Tuple4((T::Result, T::Result, T::Result, T::Result), &'a Tuple),
    TupleN(&'a Tuple),
    Nil,
    List((T::Result, T::Result), &'a List),
    FixInt(i64),
    BigInt(&'a BigInt),
}
impl<'a, T: Child<'a>> RefTerm<'a, T> {
    pub fn new(term: &'a Term) -> RefTerm<T> {
        match *term {
            Term::Atom(Atom{ref name}) => RefTerm::Atom(name),
            Term::Integer(Integer{ref value}) => {
                if let Some(n) = value.to_i64() {
                    RefTerm::FixInt(n)
                } else {
                    RefTerm::BigInt(value)
                }
            }
            Term::Nil(_) => RefTerm::Nil,
            Term::List(ref list) => RefTerm::List((T::new(&list.head), T::new(&list.tail)), list),
            Term::Tuple(ref tuple) => {
                let e = &tuple.elements;
                match e.len() {
                    1 => RefTerm::Tuple1((T::new(&e[0])), tuple),
                    2 => RefTerm::Tuple2((T::new(&e[0]), T::new(&e[1])), tuple),
                    3 => RefTerm::Tuple3((T::new(&e[0]), T::new(&e[1]), T::new(&e[2])), tuple),
                    4 => {
                        RefTerm::Tuple4((T::new(&e[0]),
                                         T::new(&e[1]),
                                         T::new(&e[2]),
                                         T::new(&e[3])),
                                        tuple)
                    }
                    _ => RefTerm::TupleN(tuple),
                }
            }
        }
    }
}

impl<'a> Child<'a> for &'a Term {
    type Result = Self;
    fn new(term: &'a Term) -> Self::Result {
        term
    }
}

pub struct RefTerm0;
impl<'a> Child<'a> for RefTerm0 {
    type Result = RefTerm<'a, &'a Term>;
    fn new(term: &'a Term) -> Self::Result {
        RefTerm::new(term)
    }
}

pub struct RefTerm1;
impl<'a> Child<'a> for RefTerm1 {
    type Result = RefTerm<'a, RefTerm0>;
    fn new(term: &'a Term) -> Self::Result {
        RefTerm::new(term)
    }
}

pub struct RefTerm2;
impl<'a> Child<'a> for RefTerm2 {
    type Result = RefTerm<'a, RefTerm1>;
    fn new(term: &'a Term) -> Self::Result {
        RefTerm::new(term)
    }
}
