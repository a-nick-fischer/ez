use crate::{error::{Spaned, TErr}, lexer::{Token, self, SignatureElement}, env::{EnvAction, Bindings}};

use std::{fmt::{Display, Formatter}, rc::Rc, cell::RefCell, collections::HashMap};


type VarContent = Rc<RefCell<Option<Type>>>;

#[derive(Clone, PartialEq, Debug)]
pub enum Type {
    Kind(String, Vec<Type>),
    Variable(String, VarContent)
}

impl Type {
    fn unify<'a>(&'a self, other: &'a Self) -> Result<(), String> {
        use Type::*;

        match (self, other) {
            // Unify types
            (Kind(a, types_a), Kind(b, types_b)) if a == b && types_a.len() == types_b.len() =>
                types_a
                    .into_iter()
                    .zip(types_b)
                    .map(|(a, b)| a.unify(b))
                    .collect(),
            
            (Variable(vname, content), other) | (other, Variable(vname, content)) => {
                {
                    if let Some(inner) = content.borrow().as_ref() {
                        return inner.unify(other)
                    }
                }
                
                if self == other {
                    Ok(())
                }
                else if other.occurs(vname) {
                    Err(format!("Type {other} contains typevar {vname}"))
                }
                else {
                    content.replace(Some(other.clone()));
                    Ok(())
                }
            }
 
            (_, _) => Err(format!("Type Mismatch: {self} and {other}"))
        }
    }

    fn refresh_vars(&self, env: &mut TypeEnv) -> Type {
        use Type::*;

        match self {
            Kind(a, types) =>
                Kind(a.clone(), types
                        .into_iter()
                        .map(|typ| typ.refresh_vars(env))
                        .collect()),

            Variable(name, content) => env.new_var(name.clone(), content.borrow().clone()),
        }
    }

    fn clear_vars(&self) {
        use Type::*;

        match self {
            Kind(a, types) =>
                types
                    .into_iter()
                    .for_each(|typ| typ.clear_vars()),

            Variable(name, content) => { 
                content.replace(None); 
            },
        }
    }

    fn occurs(&self, var: &String) -> bool {
        use Type::*;
    
        match self {
            Kind(_, types) => 
                types.into_iter().any(|t| t.occurs(var)),
            
            Variable(name, content) => 
                var == name || content.borrow().as_ref().map_or(false, |inner| inner.occurs(var))
        }
    }

    fn concretize(&self) -> Type {
        use Type::*;
    
        match self {
            Kind(name, types) => 
                Kind(name.clone(), types
                    .into_iter()
                    .map(|typ| typ.concretize())
                    .collect()),
            
            Variable(_, content) => {
                match content.borrow().as_ref() {
                    Some(inner) => inner.concretize(),

                    None => self.clone()
                }
            }
        }
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
    pub arguments: Vec<Type>,
    pub results: Vec<Type>
}

impl Signature {
    // Only to be called with fixed input
    pub fn new(src: &str) -> Self {
        if let SignatureElement::Function(arg, ret) = lexer::lex_sig(src).unwrap() {
            let mut vars = HashMap::new();

            Signature {
                arguments: sig_elems_to_type(arg, &mut vars),
                results: sig_elems_to_type(ret, &mut vars)
            }

        }
        else {
            panic!("Welp, not a function")
        }
    }

    pub fn apply(&self, env: &TypeEnv) -> Result<TypeEnv, String> {
        let arg_len = self.arguments.len();
        let stack_len = env.stack.len();

        if arg_len > stack_len {
            return Err(format!("Expected {} elem on the stack, got {}", arg_len, stack_len))
        }

        let mut tenv = env.clone();

        for i in (0..arg_len).rev() {
            let stack_args = &tenv.stack.pop().unwrap().refresh_vars(&mut tenv);

            self.arguments[i].unify(stack_args)?;
        }

        let concrete: Vec<Type> = self.results
            .clone()
            .into_iter()
            .map(|typ| typ.concretize())
            .collect();

        tenv.stack.extend(concrete);

        self.clear_vars();

        Ok(tenv)
    }

    fn clear_vars(&self){
        let clear_list = |list: &Vec<Type>| list.into_iter().for_each(|typ| typ.clear_vars());

        clear_list(&self.arguments);
        clear_list(&self.results);
    }
}


fn sig_elems_to_type(elems: Vec<SignatureElement>, vars: &mut HashMap<String, VarContent>) -> Vec<Type> {
    let convert = |elem| match elem {
        SignatureElement::Kind(name, inner) =>
            Type::Kind(name, sig_elems_to_type(inner, vars)),
        
        SignatureElement::Function(arg, res) => {
            Type::Kind("function".to_string(), vec![
                Type::Kind("arg".to_string(), sig_elems_to_type(arg, vars)),
                Type::Kind("ret".to_string(), sig_elems_to_type(res, vars))
            ])
        },

        SignatureElement::Variable(name) => {
            if let Some(content) = vars.get(&name) {
                Type::Variable(name, content.clone())
            }
            else {
                let content = Rc::new(RefCell::new(None));
                vars.insert(name.clone(), content.clone());
                Type::Variable(name, content)
            }
        },
    };

    elems.into_iter()
        .map(convert)
        .collect()
}

impl Display for Signature {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let conv = |list: &Vec<Type>| list.iter()
            .map(|arg| arg.to_string())
            .collect::<Vec<String>>()
            .join(" ");
        
        write!(f, "{} -> {}", conv(&self.arguments), conv(&self.results))
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
    pub var_counter: u32,
    pub stack: Vec<Type>,
    pub bindings: HashMap<String, Signature>
}

impl TypeEnv {
    pub fn new(bindings: &Bindings) -> TypeEnv {
        let mut env = TypeEnv {
            var_counter: 0,
            stack: vec![],
            bindings: HashMap::new()
        };

        env.bindings = bindings
            .clone()
            .into_iter()
            .map(|(key, action)| (key, action.signature(&env).unwrap()))
            .collect();

        env
    }

    pub fn new_var(&mut self, name: String, val: Option<Type>) -> Type {
        let name = format!("{name}{}", self.var_counter);
        self.var_counter += 1;

        Type::Variable(name, Rc::new(RefCell::new(val)))
    }
}

#[derive(Debug, Clone)]
pub struct TypeNode {
    pub token: Spaned<Token>,
    pub type_env: TypeEnv
}

pub fn typecheck(tokens: Vec<Spaned<Token>>, init_env: TypeEnv) -> Result<Vec<TypeNode>, TErr> {
    let token_to_node = |acc: (TypeEnv, Vec<TypeNode>), token: Spaned<Token>| {
        let (current_env, mut buffer) = acc;

        match token.signature(&current_env).and_then(|sig| sig.apply(&current_env)) {
            Ok(env) => {
                buffer.push(TypeNode {
                    token,
                    type_env: env.clone()
                });

                println!("{:?}", env.stack);

                Ok((env, buffer))
            },

            Err(msg) => {
                if let Err(a) = token.err_with(msg) {
                    Err(a)
                }
                else {
                    unreachable!()
                }
            }
        }
    };

    tokens
        .into_iter()
        .try_fold((init_env, vec![]), token_to_node)
        .map(|(_, nodes)| nodes)
}