use crate::{error::{Spaned, TErr}, lexer::{Token, self, SignatureElement}, env::{EnvAction, Bindings}};

use std::{fmt::{Display, Debug, Formatter}, rc::Rc, cell::RefCell, collections::HashMap, sync::Arc};


type VarContent = Rc<RefCell<Option<Type>>>;

#[derive(Clone, PartialEq, Debug)]
pub enum Type {
    Kind(String, Vec<Type>),
    Variable(String, VarContent)
}

impl Type {
    pub fn unify<'a>(&'a self, other: &'a Self) -> Result<(), String> {
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

    pub fn refresh_vars(&self, env: &mut TypeEnv) -> Type {
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
            Kind(_, types) =>
                types
                    .into_iter()
                    .for_each(|typ| typ.clear_vars()),

            Variable(_, content) => { 
                content.replace(None); 
            },
        }
    }

    pub fn has_bound_vars(&self) -> bool {
        use Type::*;

        match self {
            Kind(_, types) =>
                types
                    .into_iter()
                    .any(|typ| typ.has_bound_vars()),

            Variable(_, content) => 
                content.borrow().is_some(),
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

pub fn tlist_to_str(list: &Vec<Type>) -> String {
    format!("[{}]", list.into_iter()
        .map(|t| t.to_string())
        .collect::<Vec<String>>()
        .join(" "))
}

pub trait TypeEnvMod: Debug {
    fn apply(&self, env: &TypeEnv) -> Result<TypeEnv, String>;

    fn arguments(&self) -> Vec<Type>;

    fn results(&self) -> Vec<Type>;
}

#[derive(Clone, PartialEq, Debug)]
pub struct Signature {
    arguments: Vec<Type>,
    results: Vec<Type>
}

impl TypeEnvMod for Signature {
    fn apply(&self, env: &TypeEnv) -> Result<TypeEnv, String> {
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

    fn arguments(&self) -> Vec<Type> {
        self.arguments.clone()
    }

    fn results(&self) -> Vec<Type> {
        self.results.clone()
    }
}

impl Signature {
    // Only to be called with fixed input
    pub fn new(src: &str) -> Self {
        Self::from_sig(lexer::lex_sig(src).unwrap())
    }

    pub fn from_func_type(func: Type) -> Option<Self> {
        let args = Rc::new(RefCell::new(None));
        let res = Rc::new(RefCell::new(None));

        let typ = Type::Kind("fun".to_string(), vec![
            Type::Variable("_a".to_string(), args.clone()),
            Type::Variable("_b".to_string(), res.clone())
        ]);

        if typ.unify(&func).is_err() {
            println!("{:?}", typ.unify(&func));
            return None;
        }
            
        let a = args.borrow().clone();
        let b = res.borrow().clone();

        if let (Some(Type::Kind(aname, arguments)), Some(Type::Kind(bname, returns))) = (a, b) {
            if aname != "arg" || bname != "ret" {
                panic!("Invalid func type")
            }

            Some(Signature::from_types(arguments.clone(), returns.clone()))
        }
        else {
            panic!("Invalid func type")
        }
    }

    pub fn from_types(arguments: Vec<Type>, results: Vec<Type>) -> Self {
        Signature { arguments, results }
    }

    pub fn from_sig(sig: SignatureElement) -> Self {
        if let SignatureElement::Function(arg, ret) = sig {
            let mut vars = HashMap::new();

            Self::from_types(
                sig_elems_to_type(arg, &mut vars),
                sig_elems_to_type(ret, &mut vars)
            )

        }
        else {
            panic!("Welp, not a function")
        }
    }

    pub fn to_type(&self) -> Type {
        func_type(self.arguments.clone(), self.results.clone())
    }

    pub fn has_bound_vars(&self) -> bool {
        let has_binds = |list: &Vec<Type>| list.into_iter().any(|t| t.has_bound_vars());

        has_binds(&self.arguments) || has_binds(&self.results)
    }

    pub fn clear_vars(&self){
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
            Type::Kind("fun".to_string(), vec![
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

#[derive(Debug, Clone)]
pub struct Assigment {
    var_name: String
}

impl Assigment {
    pub fn new(var_name: String) -> Self {
        Assigment { var_name }
    }
}

impl TypeEnvMod for Assigment {
    fn apply(&self, env: &TypeEnv) -> Result<TypeEnv, String> {
        let mut copy = env.clone();

        let maybe_elem = copy.stack.pop();

        if let Some(typ) = maybe_elem {

            let maybe_func = Signature::from_func_type(typ.clone());

            let sig = if let Some(inner) = maybe_func { 
                inner 
            }
            else {
                Signature::from_types(vec![], vec![typ])
            };

            copy.bindings.insert(self.var_name.to_string(), Arc::new(sig));
            
            Ok(copy)

        } else {
            Err(format!("Empty stack, no value to assign to {}", self.var_name))
        }
    }

    fn arguments(&self) -> Vec<Type> {
        vec![string_type(), var_type("a")]
    }

    fn results(&self) -> Vec<Type> {
        vec![]
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

pub fn var_type(name: &str) -> Type {
    Type::Variable(name.to_string(), Rc::new(RefCell::new(None)))
}

#[derive(Clone, Debug)]
pub struct TypeEnv {
    pub var_counter: u32,
    pub stack: Vec<Type>,
    pub bindings: HashMap<String, Arc<dyn TypeEnvMod>>
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
    pub signature: Arc<dyn TypeEnvMod>
}

pub fn typecheck(tokens: Vec<Spaned<Token>>, init_env: TypeEnv) -> Result<(TypeEnv, Vec<TypeNode>), TErr> {
    let token_to_node = |acc: (TypeEnv, Vec<TypeNode>), token: Spaned<Token>| {
        let (current_env, mut buffer) = acc;
        let maybe_sig = token.signature(&current_env);

        match maybe_sig.clone().and_then(|sig| sig.apply(&current_env)) {
            Ok(env) => {
                buffer.push(TypeNode {
                    token,
                    signature: maybe_sig.unwrap()
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

    let (tenv, nodes) = tokens
        .into_iter()
        .try_fold((init_env, vec![]), token_to_node)?;

    Ok((tenv, nodes))
}