use std::fmt;
use std::fmt::Display;

pub type Arity = usize;

pub enum Term {}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct Atom {
    pub value: String,
}
impl Atom {
    pub fn new(value: String) -> Self {
        Atom { value: value }
    }
}
impl Display for Atom {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.value)
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
