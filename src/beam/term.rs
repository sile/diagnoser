use std::fmt;
use std::fmt::Display;
use std::rc::Rc;
use num::bigint::BigInt;
use num::traits::FromPrimitive;

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
    pub fn new_nil() -> Self {
        Term::Nil(Nil)
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
