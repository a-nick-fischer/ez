use std::{fmt::{Display, Formatter}, cell::RefCell, rc::Rc};

use crate::{env_modifier::EnvModifier, error::TErr};

use super::{types::Type, type_env::TypeEnv, func_type};

#[derive(Clone, PartialEq, Debug)]
pub struct Signature {
    arguments: Vec<Type>,
    results: Vec<Type>
}

impl Signature {
    pub fn new(arguments: Vec<Type>, results: Vec<Type>) -> Self {
        Signature { arguments, results }
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


impl TryFrom<Type> for Signature {
    type Error = String;

    fn try_from(func: Type) -> Result<Self, Self::Error> {
        let args = Rc::new(RefCell::new(None));
        let res = Rc::new(RefCell::new(None));

        let typ = Type::Kind("fun".to_string(), vec![
            Type::Variable("_a".to_string(), args.clone()),
            Type::Variable("_b".to_string(), res.clone())
        ]);

        typ.unify(&func)?;
            
        let a = args.borrow().clone();
        let b = res.borrow().clone();

        if let (Some(Type::Kind(aname, arguments)), Some(Type::Kind(bname, returns))) = (a, b) {
            if aname != "arg" || bname != "ret" {
                panic!("Invalid func type") // TODO Error handling
            }

            Ok(Signature::new(arguments.clone(), returns.clone()))
        }
        else {
            Err("Invalid func type".to_owned()) // TODO Error handling
        }
    }
}

impl Into<Type> for Signature {
    fn into(self) -> Type {
        func_type(self.arguments.clone(), self.results.clone())
    }
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

impl EnvModifier<TypeEnv> for Signature {
    fn apply(&self, env: &mut TypeEnv) -> Result<(), TErr> {
        let arg_len = self.arguments.len();
        let stack_len = env.stack.len();

        if arg_len > stack_len {
            panic!("Expected {} elem on the stack, got {}", arg_len, stack_len) // TODO Error handling
        }

        let mut tenv = env.clone();

        for i in (0..arg_len).rev() {
            let stack_args = &tenv.stack.pop().unwrap().refresh_vars(&mut tenv);

            self.arguments[i].unify(stack_args).unwrap(); // TODO Error handling
        }

        let concrete: Vec<Type> = self.results
            .clone()
            .into_iter()
            .map(|typ| typ.concretize())
            .collect();

        tenv.stack.extend(concrete);

        self.clear_vars();

        *env = tenv;

        Ok(())
    }
}