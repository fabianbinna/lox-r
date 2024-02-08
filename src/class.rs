use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

use crate::object::Object;


#[derive(Clone)]
pub struct Class {
    pub name: String,
    superclass: Option<Rc<RefCell<Class>>>,
    methods: HashMap<String, Object>,
}

impl Class {

    pub fn new(name: String, superclass: Option<Rc<RefCell<Class>>>, methods: HashMap<String, Object>) -> Self {
        Class {
            name, 
            superclass,
            methods
        }
    }

    pub fn find_method(&self, name: &str) -> Option<Object> {
        if let Some(method) = self.methods.get(name).cloned() {
            return Some(method)
        }

        if let Some(superclass) = self.superclass.clone() {
            return superclass.borrow().find_method(name);
        }

        return None;
    }

}

impl Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}