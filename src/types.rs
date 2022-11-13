use crate::{error::{Spaned, TResult}, lexer::Token};

use std::fmt::{Display, Formatter};

#[derive(Clone, PartialEq, Debug)]
pub enum Type {
    Kind(String, Vec<Type>),
    Variable(String, Option<Box<Type>>)
}

impl Type {
    fn unify(&self, other: &Self) -> Result<Self, String> {
        use Type::*;

        match (self, other) {
            (Kind(a, types_a), Kind(b, types_b)) if a == b && types_a == types_b => 
                Ok(self.clone()),
            
            (Variable(a, _), Variable(b, _)) if a == b =>
                Ok(self.clone()),
            
            (Variable(a, None), Variable(b, content_b)) | (Variable(b, content_b), Variable(a, None)) =>
                Ok(Type::gen_equal_variables(a, b, content_b.clone().map(|t| *t))),

            (Variable(a, Some(content_a)), Variable(b, Some(content_b))) =>
                content_a.unify(content_b)
                    .map(|content_c| Type::gen_equal_variables(a, b, Some(content_c.clone()))),

            (Variable(a, None), other) | (Variable(a, None), other) if !other.occurs(a) =>
                Ok(Variable(a.clone(), Some(Box::new(other.clone())))),
 
            (a, b) => Err(format!("Type Mismatch: {} and {}", self, other))
        }
    }

    fn refresh_vars(&self, env: &TypeEnv) -> (Type, TypeEnv) {
        use Type::*;

        match self {
            Kind(a, types) => {
                let mut buffer = vec![];
                let last = env.clone();

                for typ in types {
                    let (newt, last) = typ.refresh_vars(env);
                    buffer.push(newt);
                }

                (Kind(a.clone(), buffer), last)
            },

            Variable(_, content) => env.new_var(content.clone()),
        }
    }

    fn occurs(&self, var: &String) -> bool {
        use Type::*;
    
        match self {
            Kind(_, types) => 
                types.into_iter().any(|t| t.occurs(var)),
            
            Variable(name, _) if var == name => 
                true,
            
            Variable(_, Some(t)) => 
                t.occurs(var),

            _ => false
        }
    }

    fn bind_in(name: String, value: Option<Box<Type>>, list: Vec<Type>) -> Vec<Type> {
        use Type::*;

        let mut buffer = vec![];

        for elem in list {
            let res = match elem {
                Kind(a, types) => Kind(a, Type::bind_in(name, value, types)),

                Variable(a, _) if name == a => Variable(name, value),

                _ => elem
            };

            buffer.push(res);
        }

        buffer
    }

    fn gen_equal_variables(a: &String, b: &String, content: Option<Type>) -> Type {
        use Type::*;

        Variable(b.clone(), Some(Box::new(Variable(a.clone(), content.map(Box::new)))))
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use Type::*;

        match self {
            Kind(name, types) => {
                let type_str = types
                    .into_iter()
                    .map(|t| format!("[{t}]"))
                    .collect::<Vec<String>>()
                    .join("");

                write!(f, "{name}{type_str}")
            },
               
            Variable(name, _) => write!(f, "'{name}")
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Signature {
    arguments: Vec<Type>,
    results: Vec<Type>
}

impl Signature {
    pub fn apply(&self, env: &TypeEnv) -> Result<TypeEnv, String> {
        // 1. Refresh typeenv vars
        // 2. Unify with arguments
        // 3. Bind result variables
        // 3.5 Concretise
        // 4. Return result
        // 5. ???
        // 6. Profit! 

        let arg_len = self.arguments.len();
        let stack_len = env.stack.len();

        if arg_len > stack_len {
            return Err(format!("Expected {} elem on the stack, got {}", arg_len, stack_len))
        }

        let mut tenv = env.clone();
        let mut args = self.arguments;
        let mut results = self.results;

        for i in (0..arg_len).rev() {
            let (stack_args, tenv) = &env.stack.pop().unwrap().refresh_vars(&tenv);

            let res = args[i].unify(stack_args)?;

            if let Type::Variable(name, val) = res {
                args = Type::bind_in(name, val, args);
                results = Type::bind_in(name, val, results);
            }
        }

        tenv.stack = [tenv.stack, results].concat();

        Ok(tenv)
    } 
}

impl Display for Signature {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let args = self.arguments.into_iter()
            .map(|arg| arg.to_string())
            .collect::<Vec<String>>()
            .join(" ");
        
        let results = self.results.into_iter()
            .map(|arg| arg.to_string())
            .collect::<Vec<String>>()
            .join(" ");
        
        write!(f, "{args} -> {results}")
    }
}

pub fn string_type() -> Type {
    Type::Kind("str".to_string(), vec![])
}

pub fn number_type() -> Type {
    Type::Kind("num".to_string(), vec![])
}

pub fn func_type(args: Vec<Type>, result: Vec<Type>) -> Type {
    Type::Kind("fun".to_string(), vec![
        Type::Kind("arg".to_string(), args),
        Type::Kind("ret".to_string(), result)
    ])
}

#[derive(Clone, PartialEq, Debug)]
pub struct TypeEnv {
    var_counter: u32,
    stack: Vec<Type>
}

impl TypeEnv {
    pub fn new_var(&self, val: Option<Box<Type>>) -> (Type, TypeEnv) {
        let name = format!("v{}", self.var_counter);
        
        let mut copy = self.clone();
        copy.var_counter += 1;

        (Type::Variable(name, val), copy)
    }
}

pub struct TypeNode {
    token: Spaned<Token>,
    signature: Signature,
    type_env: TypeEnv
}

pub fn typecheck(tokens: Vec<Spaned<Token>>) -> TResult<Vec<TypeNode>> {

}