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

pub trait EzFun<M: Module> {
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

pub struct FuncCodeTransformation<M> {
    pub inner: Box<dyn EzFun<M>>
}

impl<M: Module> CodeTransformation<M> for FuncCodeTransformation<M> {
    fn try_apply<'b>(
        &self,
        nodes: &mut Vec<Node>,
        translator: &mut FunctionTranslator<'b, M>,
        builder: &mut FunctionBuilder
    ) -> Result<bool, Error> {
        match_nodes!(
            nodes(1): [Node::Call { name, .. }] if name == self.inner.name() => {
                if self.inner.should_inline() {
                    return self.inner.try_apply_inline(nodes, translator, builder);
                }

                translator.ins_call(name, self.inner.signature().arguments().len(), builder)?;
            }
        )
    }
}

pub struct NativeFun {
    name: String,

    sig: TypedSignature
}

impl NativeFun {
    pub fn new(name: &str, sig: &str) -> Result<Self, Error> {
        Ok(Self {
            name: name.to_string(),

            sig: sig.parse()?
        })
    }
}

impl<M: Module> EzFun<M> for NativeFun {
    fn init(&self, codegen: &mut CodeGenModule<M>) -> Result<(), Error> {
        let mut sig = codegen.build_cranelift_signature(&self.sig)?;
        sig.call_conv = codegen.module.target_config().default_call_conv;
    
        codegen.module
            .declare_function(&self.name, Linkage::Import, &sig)?;

        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn signature(&self) -> TypedSignature {
        self.sig.clone()
    }
}

pub struct UserFun {
    name: String,

    sig: TypedSignature,

    src: Vec<Node>,

    inline: bool
}

impl UserFun {
    pub fn new(name: &str, sig: &str, src: &str, tenv: &mut TypeEnv) -> Result<Self, Error> {
        UserFun::new_raw(name, sig, src, tenv, false)
    }

    pub fn new_inline(name: &str, sig: &str, src: &str, tenv: &mut TypeEnv) -> Result<Self, Error> {
        UserFun::new_raw(name, sig, src, tenv, true)
    }

    fn new_raw(name: &str, sig: &str, src: &str, tenv: &mut TypeEnv, inline: bool) -> Result<Self, Error> {
        let tokens = lex(src.to_string())?;
        let nodes = parse(tokens, tenv)?;

        Ok(Self {
            name: name.to_string(),
            inline,
            src: nodes,
            sig: sig.parse()?
        })
    }
}

impl<M: Module> EzFun<M> for UserFun {
    fn init(&self, codegen: &mut CodeGenModule<M>) -> Result<(), Error> {
        FunctionTranslator::new(codegen)
            .with_signature(self.sig.clone())
            .with_body(self.src.clone())?
            .finish_func(&self.name, FunctionOptions::internal())?;

        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
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
        
        translator.translate_nodes(self.src.clone(), builder);

        Ok(true)
    }
}