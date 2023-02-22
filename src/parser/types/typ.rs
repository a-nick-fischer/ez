use std::{fmt::{Display, Formatter}, sync::{Mutex, Arc}};

use ariadne::{Color, Fmt};

use super::{typelist::TypeList, type_env::TypeEnv, typ, var_type_raw};

pub type VarContent = Arc<Mutex<Option<Type>>>;

#[derive(Clone, Debug)]
pub enum Type {
    Kind(String, TypeList),
    Variable(String, VarContent)
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Kind(a_name, a_vars), Self::Kind(b_name, b_vars)) => 
                a_name == b_name && a_vars == b_vars,

            // Theoretically it's valid to assume that if two variables
            // with the same name are unified, it's a bug if their contents
            // are not equal; therefore we may only compare the variable names
            (Self::Variable(a_name, _), Self::Variable(b_name, _)) => 
                a_name == b_name,

            _ => false,
        }
    }
}

impl Type {
    pub fn unify<'a>(&'a self, other: &'a Self) -> Result<(), String> {
        use Type::*;

        match (self, other) {
            // Unify types
            (Kind(a, types_a), Kind(b, types_b)) if a == b && types_a.len() == types_b.len() =>
                types_a
                    .vec()
                    .iter()
                    .zip(types_b.vec())
                    .try_for_each(|(a, b)| a.unify(b)),
            
            (Variable(vname, content), other) | (other, Variable(vname, content)) => {
                {
                    if let Some(inner) = content.lock().unwrap().as_ref() {
                        return inner.unify(other)
                    }
                }
                
                if self == other {
                    Ok(())
                }
                else if other.occurs(vname) {
                    Err(format!(
                        "Occurs Check: Type {} contains typevar {}",
                        other.fg(Color::Cyan),
                        vname.fg(Color::Cyan),
                    ))
                }
                else {
                    *content.lock().unwrap() = Some(other.clone());
                    Ok(())
                }
            }
 
            (_, _) => Err(format!(
                "Type Mismatch: {} and {}",
                self.fg(Color::Cyan),
                other.fg(Color::Cyan)
            ))
        }
    }

    pub fn refresh_vars(&self, env: &mut TypeEnv) -> Type {
        use Type::*;

        match self {
            Kind(a, types) =>
                Kind(a.clone(), types.refresh_vars(env)),

            Variable(name, content) => 
                env.new_var(name.clone(), content.lock().unwrap().clone()),
        }
    }

    pub fn clear_vars(&self) {
        use Type::*;

        match self {
            Kind(_, types) => types.clear_vars(),

            Variable(_, content) => { 
                *content.lock().unwrap() = None;
            },
        }
    }

    pub fn has_bound_vars(&self) -> bool {
        use Type::*;

        match self {
            Kind(_, types) =>
                types.has_bound_vars(),

            Variable(_, content) => 
                content.lock().unwrap().is_some(),
        }
    }

    pub fn occurs(&self, var: &String) -> bool {
        use Type::*;
    
        match self {
            Kind(_, types) => types.occurs(var),
            
            Variable(name, content) => 
                var == name || content.lock()
                    .unwrap()
                    .as_ref()
                    .map_or(false, |inner| inner.occurs(var))
        }
    }

    pub fn concretize(&self) -> Type {
        use Type::*;
    
        match self {
            Kind(name, types) => 
                Kind(name.clone(), types.concretize()),
            
            Variable(_, content) => {
                match content.lock().unwrap().as_ref() {
                    Some(inner) => inner.concretize(),

                    None => self.clone()
                }
            }
        }
    }

    pub fn extract_function(&self) -> Option<(TypeList, TypeList)> {
        let args = Arc::new(Mutex::new(None));
        let res = Arc::new(Mutex::new(None));

        let typ = typ("fun", vec![
            var_type_raw("_a", args.clone()),
            var_type_raw("_b", res.clone())
        ]);

        let unified = typ.unify(self);
        if unified.is_err() {
            return None;
        }
            
        let a = args.lock().unwrap().clone();
        let b = res.lock().unwrap().clone();

        if let (Some(Type::Kind(_, arguments)), Some(Type::Kind(_, returns))) = (a, b) {
            Some((arguments, returns))
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
                    .vec()
                    .iter()
                    .map(|t| format!("[{t}]"))
                    .collect::<Vec<String>>()
                    .join("");

                write!(f, "{name}{type_str}")
            },
               
            Variable(name, _) => write!(f, "'{name}")
        }
    }
}
