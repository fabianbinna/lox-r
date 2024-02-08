use std::{rc::Rc, cell::RefCell, fmt::Display};
use crate::{class::Class, function::Function, instance::Instance, native::NativeFunction};

#[derive(Clone)]
pub enum Object {
    Number(f64),
    String(String),
    Boolean(bool),
    Function(Rc<RefCell<Function>>),
    NativeFunction(NativeFunction),
    Class(Rc<RefCell<Class>>),
    Instance(Rc<RefCell<Instance>>),
    Nil
}

impl Object {

    pub fn is_truthy(&self) -> bool {
        match self {
            Object::Boolean(false) => false,
            Object::Nil => false,
            _ => true
        }
    }

}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Number(l), Self::Number(r)) => l == r,
            (Self::String(l), Self::String(r)) => l == r,
            (Self::Boolean(l), Self::Boolean(r)) => l == r,
            (Self::Function(l), Self::Function(r)) => {
                l.borrow().name == r.borrow().name
            },
            (Self::Class(l), Self::Class(r)) => {
                l.borrow().name == r.borrow().name
            },
            _ => false
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Number(n) => write!(f, "{n}"),
            Object::String(s) => write!(f, "{s}"),
            Object::Boolean(b) => write!(f, "{b}"),
            Object::Function(function) => writeln!(f, "{}", function.borrow()),
            Object::Class(class) => write!(f, "{}", class.borrow()),
            Object::Instance(instance) => write!(f, "{}", instance.borrow()),
            Object::Nil => write!(f, "nil"),
            Object::NativeFunction(name) => write!(f, "{name}"),
        }
    }
}