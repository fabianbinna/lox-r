use std::{rc::Rc, cell::RefCell, collections::HashMap, fmt::Display};

use crate::{class::Class, object::Object};

pub struct Instance {
    pub class: Rc<RefCell<Class>>,
    fields: HashMap<String, Object>,
}

impl Instance {

    pub fn new(class: Rc<RefCell<Class>>) -> Self {
        Instance {
            class,
            fields: HashMap::new()
        }
    }

    pub fn get(&self, name: &str, instance: &Object) -> Object {
        if let Some(value) = self.fields.get(name) {
            return value.clone();
        } 
        
        if let Some(method) = self.class.borrow().find_method(name) {
            if let Object::Function(function) = method {
                let function = function.borrow_mut().bind(instance.clone());
                return Object::Function(Rc::new(RefCell::new(function)));
            }
        }

        panic!("undefined property '{}'", name);
    }

    pub fn set(&mut self, name: String, value: Object) {
        self.fields.insert(name, value);
    }

}

impl Display for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.class.borrow())
    }
}