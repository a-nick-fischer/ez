use cranelift::prelude::FunctionBuilder;
use cranelift_module::{Module, Linkage};

use crate::{parser::{signature_parser::TypedSignature, node::Node, parse, types::type_env::TypeEnv}, codegen::{function_translator::{FunctionTranslator, FunctionOptions}, codegen_module::CodeGenModule}, error::Error, match_nodes, lexer::lex};

pub trait CodeTransformation<M: Module> {
    fn try_apply<'b>(
        &self,
        nodes: &mut Vec<Node>,
        translator: &mut FunctionTranslator<'b, M>,
        builder: &mut FunctionBuilder
    ) -> Result<bool, Error>;
}

pub trait EzFun<M: Module>: CodeTransformation<M> {
    fn init(&self, codegen: &mut CodeGenModule<M>) -> Result<(), Error>;

    fn name(&self) -> &str;

    fn signature(&self) -> TypedSignature;

    fn should_inline(&self) -> bool { 
        false
    }

    fn try_apply_inline<'b>(
        &self,
        nodes: &mut Vec<Node>,
        translator: &mut FunctionTranslator<'b, M>,
        builder: &mut FunctionBuilder
    ) -> Result<bool, Error> { 
        Ok(false) 
    }
}

// Black magic f*ckery
impl<M: Module, T: EzFun<M>> CodeTransformation<M> for T {
    fn try_apply<'b>(
        &self,
        nodes: &mut Vec<Node>,
        translator: &mut FunctionTranslator<'b, M>,
        builder: &mut FunctionBuilder
    ) -> Result<bool, Error> {
        match_nodes!(
            nodes(1): [Node::Call { name, .. }] if name == self.name() => {
                if self.should_inline() {
                    return self.try_apply_inline(nodes, translator, builder);
                }

                translator.ins_call(name, self.signature().arguments().len(), builder)?;
            }
        )
    }
}

pub struct NativeFun<'a> {
    name: &'a str,

    sig: TypedSignature
}

impl<'a> NativeFun<'a> {
    pub fn new(name: &'a str, sig: &str) -> Result<Self, Error> {
        Ok(Self {
            name,

            sig: sig.parse()?
        })
    }
}

impl<'a, M: Module> EzFun<M> for NativeFun<'a> {
    fn init(&self, codegen: &mut CodeGenModule<M>) -> Result<(), Error> {
        let mut sig = codegen.build_cranelift_signature(&self.sig)?;
        sig.call_conv = codegen.module.target_config().default_call_conv;
    
        codegen.module
            .declare_function(self.name, Linkage::Import, &sig)?;

        Ok(())
    }

    fn name(&self) -> &str {
        self.name
    }

    fn signature(&self) -> TypedSignature {
        self.sig.clone()
    }
}

pub struct UserFun<'a> {
    name: &'a str,

    sig: TypedSignature,

    src: Vec<Node>,

    inline: bool
}

impl<'a> UserFun<'a> {
    pub fn new(name: &'a str, sig: &str, src: &str, tenv: &mut TypeEnv) -> Result<Self, Error> {
        UserFun::new_raw(name, sig, src, tenv, false)
    }

    pub fn new_inline(name: &'a str, sig: &str, src: &str, tenv: &mut TypeEnv) -> Result<Self, Error> {
        UserFun::new_raw(name, sig, src, tenv, true)
    }

    fn new_raw(name: &'a str, sig: &str, src: &str, tenv: &mut TypeEnv, inline: bool) -> Result<Self, Error> {
        let tokens = lex(src.to_string())?;
        let nodes = parse(tokens, tenv)?;

        Ok(Self {
            name,
            inline,
            src: nodes,
            sig: sig.parse()?
        })
    }
}

impl<'a, M: Module> EzFun<M> for UserFun<'a> {
    fn init(&self, codegen: &mut CodeGenModule<M>) -> Result<(), Error> {
        FunctionTranslator::new(codegen)
            .with_signature(self.sig.clone())
            .with_body(self.src.clone())?
            .finish_func(self.name, FunctionOptions::internal())?;

        Ok(())
    }

    fn name(&self) -> &str {
        self.name
    }

    fn signature(&self) -> TypedSignature {
        self.sig.clone()
    }

    fn should_inline(&self) -> bool { self.inline }

    fn try_apply_inline<'b>(
            &self,
            _nodes: &mut Vec<Node>,
            translator: &mut FunctionTranslator<'b, M>,
            builder: &mut FunctionBuilder
        ) -> Result<bool, Error> {
        
        for node in self.src.clone() {
            translator.translate_node(node, builder)?;
        }

        Ok(true)
    }
}