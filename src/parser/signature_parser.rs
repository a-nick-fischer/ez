use std::{collections::HashMap, cell::RefCell, rc::Rc};

use crate::{lexer::sig_lexer::{SignatureElement}};

use super::{types::{types::{VarContent, Type}, typelist::TypeList, *}};

pub fn parse_signature(sig: SignatureElement) -> (TypeList, TypeList) {
    if let SignatureElement::Function(arg, ret) = sig { // TODO Change parsing to guarantee a valid value here
        let mut vars = HashMap::new();
    
        (
            TypeList::from(build_signature(arg, &mut vars)),
            TypeList::from(build_signature(ret, &mut vars))
        )
    }
    else {
        panic!("Welp, not a function")
    }
}

fn build_signature(elems: Vec<SignatureElement>, vars: &mut HashMap<String, VarContent>) -> Vec<Type> {
    let convert = |elem| match elem {
        SignatureElement::Kind(name, inner) =>
            typ(&name.to_string(), build_signature(inner, vars)),
        
        SignatureElement::Function(arg, res) => {
            func_type(build_signature(arg, vars), build_signature(res, vars))
        },

        SignatureElement::Variable(name) => {
            if let Some(content) = vars.get(&name) {
                var_type_raw(&name.to_string(), content.clone())
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