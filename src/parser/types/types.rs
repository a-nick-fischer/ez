use std::{fmt::{Display, Debug, Formatter}, rc::Rc, cell::RefCell, borrow::Borrow};

use super::{type_env::TypeEnv, typelist::TypeList, *};

pub type VarContent = Rc<RefCell<Option<Type>>>;

#[derive(Clone, PartialEq, Debug)]
pub enum Type {
    Kind(String, TypeList),
    Variable(String, VarContent)
}

impl Type {
    pub fn unify<'a>(&'a self, other: &'a Self) -> Result<(), String> {
        use Type::*;

        match (self, other) {
            // Unify types
            (Kind(a, types_a), Kind(b, types_b)) if a == b && types_a.len() == types_b.len() =>
                types_a
                    .vec()
                    .into_iter()
                    .zip(types_b.vec())
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
                Kind(a.clone(), types.refresh_vars(env)),

            Variable(name, content) => 
                env.new_var(name.clone(), content.borrow().clone()),
        }
    }

    pub fn clear_vars(&self) {
        use Type::*;

        match self {
            Kind(_, types) => types.clear_vars(),

            Variable(_, content) => { 
                content.replace(None); 
            },
        }
    }

    pub fn has_bound_vars(&self) -> bool {
        use Type::*;

        match self {
            Kind(_, types) =>
                types.has_bound_vars(),

            Variable(_, content) => 
                content.borrow().is_some(),
        }
    }

    pub fn occurs(&self, var: &String) -> bool {
        use Type::*;
    
        match self {
            Kind(_, types) => types.occurs(var),
            
            Variable(name, content) => 
                var == name || content.borrow().as_ref().map_or(false, |inner| inner.occurs(var))
        }
    }

    pub fn concretize(&self) -> Type {
        use Type::*;
    
        match self {
            Kind(name, types) => 
                Kind(name.clone(), types.concretize()),
            
            Variable(_, content) => {
                match content.borrow().as_ref() {
                    Some(inner) => inner.concretize(),

                    None => self.clone()
                }
            }
        }
    }

    pub fn extract_function(&self) -> Option<(TypeList, TypeList)> {
        let args = Rc::new(RefCell::new(None));
        let res = Rc::new(RefCell::new(None));

        let typ = typ("fun", vec![
            var_type_raw("_a", args.clone()),
            var_type_raw("_b", res.clone())
        ]);

        let res = typ.unify(&self);
        if res.is_err() {
            return None;
        }
            
        let a = args.borrow().clone();
        let b = res.borrow().clone();

        if let (Some(Type::Kind(aname, arguments)), Some(Type::Kind(bname, returns))) = (a, b) {
            if aname != "arg" || bname != "ret" {
                panic!("Invalid func type") // TODO Error handling
            }

            Some((arguments.clone(), returns.clone()))
        }
        else { None }
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
