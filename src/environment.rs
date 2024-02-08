use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::object::Object;

#[derive(Clone)]
pub struct Environment {
    pub parent: Option<Rc<RefCell<Environment>>>,
    env: HashMap<String, Object>
}

impl Environment {

    pub fn new(parent: Option<Rc<RefCell<Environment>>>) -> Self {
        Environment {
            parent,
            env: HashMap::new()
        }
    }

    pub fn define(&mut self, name: String, value: Object) {
        self.env.insert(name, value);
    }

    pub fn assign(&mut self, name: String, value: Object) {
        if self.env.contains_key(&name) {
            self.env.insert(name, value);
        } else {
            if let Some(p) = self.parent.as_mut() {
                p.borrow_mut().assign(name, value);
            }
        }
    }

    pub fn get(&self, name: &String) -> Object {
        if self.env.contains_key(name) {
            return self.env.get(name).unwrap().clone();
        } else {
            if let Some(p) = self.parent.as_ref() {
                return p.borrow().get(name);
            }
        }

        panic!("Variable {} not defined.", name);
    }

    pub fn get_return_value(&mut self) -> Object {
        match self.env.get("return_value") {
            Some(value) => {
                let tmp = value.clone();
                self.env.insert(String::from("return_value"), Object::Nil);
                return tmp;
            },
            None => Object::Nil
        }
    }

    pub fn assign_at(&mut self, distance: usize, name: String, value: Object) {
        if distance == 0 {
            self.env.insert(name, value);
        } else {
            if let Some(p) = self.parent.as_mut() {
                p.borrow_mut().assign_at(distance-1, name, value);
            }
        }
    }

    pub fn get_at(&self, distance: usize, name: &String) -> Object {
        if distance == 0 {
            return self.env.get(name).unwrap().clone();
        } else {
            if self.parent.is_some() {
                return self.parent.as_ref().unwrap().borrow().get_at(distance-1, name);
            }
        }

        panic!("Variable {} not defined.", name);
    }

}
