pub enum Value {
    Number(f32),
    Quote(String),
    List(Vec<Value>),
    Function(Vec<Token>),
}

