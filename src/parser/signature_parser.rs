use std::{collections::HashMap, cell::RefCell, rc::Rc};

use crate::{lexer::sig_lexer::{Signature, SignatureElement}};

use super::{types::{types::{VarContent, Type}, typelist::TypeList, *}};

pub fn parse_signature(sig: Signature) -> (TypeList, TypeList) {
    let mut vars = HashMap::new();

    (
        TypeList::from(build_signature(sig.get_args().clone(), &mut vars)),
        TypeList::from(build_signature(sig.get_returns().clone(), &mut vars))
    )
}

fn build_signature(elems: Vec<SignatureElement>, vars: &mut HashMap<String, VarContent>) -> Vec<Type> {
    let convert = |elem| match elem {
        SignatureElement::Kind(name, inner) =>
            typ(&name.to_string(), build_signature(inner, vars)),

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