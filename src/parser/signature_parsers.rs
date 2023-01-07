use std::{str::FromStr, collections::HashMap, cell::RefCell, rc::Rc};

use crate::{lexer::sig_lexer::{SignatureElement, self}, typechecker::{signature::Signature, types::{Type, VarContent}}, error::err_to_str};

impl FromStr for Signature {
    type Err = String;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        sig_lexer::lex_sig(src)
            .map(|sig_elem| sig_elem.into())
            .map_err(|terr| err_to_str(terr))
    }
}

impl From<SignatureElement> for Signature {
    fn from(sig: SignatureElement) -> Self {
        if let SignatureElement::Function(arg, ret) = sig { // TODO Change parsing to guarantee a valid value here
            let mut vars = HashMap::new();
        
            Self::new(
                sig_elems_to_type(arg, &mut vars),
                sig_elems_to_type(ret, &mut vars)
            )
        
        }
        else {
            panic!("Welp, not a function")
        }
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