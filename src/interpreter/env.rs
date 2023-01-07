pub type Bindings = HashMap<String, Value>;

#[derive(Debug)]
pub struct Env {
    stack: Vec<Value>,
    vars: Bindings
}

impl Env {
    pub fn new(vars: Bindings) -> Self {
        Env { stack: Vec::new(), vars }
    }

    pub fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    pub fn pop(&mut self) -> Value {
        self.stack.pop().unwrap() // TODO Error handling
    }

    pub fn get(&self, ident: &String) -> Value {
        self.vars.get(ident).unwrap() // TODO Error handling
    }

    pub fn set(&mut self, ident: &String, value: Value){
        self.vars.insert(ident.to_owned(), value);
    }
}