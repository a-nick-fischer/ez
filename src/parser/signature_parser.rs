use std::{collections::HashMap, cell::RefCell, rc::Rc, str::FromStr};

use crate::{lexer::sig_lexer::{LexedSignature, SignatureElement, lex_signature}, error::Error};

use super::{types::{types::{VarContent, Type}, typelist::TypeList, *}};

#[derive(Debug, Clone)]
pub struct TypedSignature(pub TypeList, pub TypeList);

impl TypedSignature {
    pub fn new(args: TypeList, rets: TypeList) -> Self {
        TypedSignature(args, rets)
    }

    pub fn arguments(&self) -> &TypeList {
        &self.0
    }

    pub fn returns(&self) -> &TypeList {
        &self.1
    }
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

impl From<LexedSignature> for TypedSignature {
    fn from(sig: LexedSignature) -> Self {
        let mut vars = HashMap::new();

        TypedSignature::new(
            TypeList::from(build_signature(sig.get_args().clone(), &mut vars)),
            TypeList::from(build_signature(sig.get_returns().clone(), &mut vars))
        )
    }
}

impl Into<Type> for TypedSignature {
    fn into(self) -> Type {
        func_type(
            self.0.vec().clone(),
            self.1.vec().clone()
        )
    }
}

impl FromStr for TypedSignature {
    type Err = Error;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        Ok(lex_signature(src)?.into())
    }
}