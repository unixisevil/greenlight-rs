use std::collections::HashMap;


pub struct Validator {
    errors: HashMap<&'static str, &'static str>,
}

impl From<HashMap<&'static str, &'static str>> for Validator {
    fn from(value: HashMap<&'static str, &'static str>) -> Self {
         Self{ errors: value }
    }
}

impl Validator {
    pub fn new()  -> Self {
        Self{ errors:  HashMap::new() }
    }

    pub fn valid(&self) -> bool {
        self.errors.is_empty() 
    }

    pub fn add_err(&mut self, field: &'static str,  err_desc: &'static str) {
        if !self.errors.contains_key(field) {
            self.errors.insert(field, err_desc);
        }
    }

    pub fn check(&mut self, ok: bool, field: &'static str,  err_desc: &'static str) {
        if !ok {
            self.add_err(field, err_desc);
        }
    }

    pub fn get_err(self) -> HashMap<&'static str, &'static str> {
        self.errors
    }
}

