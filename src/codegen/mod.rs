use std::collections::HashMap;

use cranelift::prelude::*;
use cranelift::{codegen, prelude::{FunctionBuilderContext, FunctionBuilder, InstBuilder}};
use cranelift_module::{Module, DataContext, Linkage, DataId, FuncId, FuncOrDataId};

use crate::{parser::{node::Node, types::{types::Type, *}}, lexer::token::Token};

pub mod compiler;
pub mod jit;

struct Translator {
    builder_context: FunctionBuilderContext,

    ctx: codegen::Context,

    data_ctx: DataContext,

    module: Box<dyn Module>,

    naming_idx: u32
}

impl Translator {
    fn translate(&mut self, nodes: Vec<Node>) -> Result<FuncId, ()> {
        let tran = FunctionTranslator::new(&mut self);
        
        for node in nodes {
            tran.translate_node(node);
        }

        let sig = self.module.make_signature();

        let id = self
            .module
            .declare_anonymous_function(&sig)
            .unwrap(); // TODO Error handling

        self.module
            .define_function(id, &mut self.ctx)
            .unwrap(); // TODO Error handling

        id
    }

    fn create_data(&mut self, name: String, content: Vec<u8>) -> DataId {
        self.data_ctx.define(content.into_boxed_slice());

        let id = self
            .module
            .declare_data(&name, Linkage::Export, true, false)
            .unwrap(); // TODO Error handling

        self.module
            .define_data(id, &self.data_ctx)
            .unwrap(); // TODO Error handling

        self.data_ctx.clear(); // TODO Needed?

        id
    }

    fn get_func_by_name(&self, name: &str) -> FuncId {
        let maybe_func = self
            .module
            .declarations()
            .get_name(name);

        match maybe_func {
            Some(FuncOrDataId::Func(id)) => id,

            _ => panic!("Replace me with an error"), // TODO Error handling
        }
    }

    fn pointer_type(&self) -> cranelift::prelude::Type {
        self.module.target_config().pointer_type()
    }

    fn gen_name(&mut self) -> String {
        self.naming_idx += 1;
        format!("n{}", self.naming_idx)
    }

    fn declare_libc_funcs(&mut self){
        let mut sig = self.module.make_signature();
        sig.params.push(AbiParam::new(self.pointer_type()));
        sig.returns.push(AbiParam::new(self.pointer_type()));

        let callee = self
            .module
            .declare_function("malloc", Linkage::Import, &sig)
            .expect("problem declaring function"); // TODO Error handling
    }
}

struct FunctionTranslator<'a> {
    builder: FunctionBuilder<'a>,

    parent: &'a mut Translator,

    variables: HashMap<String, Variable>
}

impl<'a> FunctionTranslator<'a> {
    fn new(parent: &'a mut Translator) -> Self {
        FunctionTranslator { 
            builder: FunctionBuilder::new(&mut parent.ctx.func, &mut parent.builder_context), 
            parent,
            variables: HashMap::new() 
        }
    }

    fn translate_node(&mut self, node: Node) {
        match node {
            Node::Assigment { name, typ, .. } => {
                let var = Variable::new(self.variables.len());
                self.variables.insert(name, var);

                self.builder.def_var(var, todo!()); // TODO 
            },
    
            Node::Variable { name, typ, .. } => {
                let var = self.variables.get(&name).unwrap(); // TODO Error handling
                self.builder.use_var(*var);
                // TODO Push on stack
            },
    
            Node::Call { name, arguments, returns, .. } => {
                let mut sig = self.parent.module.make_signature();

                for arg in arguments.vec() {
                    sig.params.push(AbiParam::new((*arg).into()))
                }

                for ret in returns.vec() {
                    sig.returns.push(AbiParam::new((*ret).into()))
                }

                let func = self.parent.get_func_by_name(name.as_str());

                let local_callee = self
                    .parent
                    .module
                    .declare_func_in_func(func, self.builder.func);

                self.builder.call(local_callee, todo!()); // TODO Error handling
            },
    
            Node::Literal { typ, token, .. } => {
                self.build_literal(typ, token);
                 // TODO Push on stack
            }
        }
    }

    fn build_literal(&mut self, typ: Type, token: Token) -> Value {
        if let Type::Kind(typ_name, _type_vars) = typ {
            match (typ_name.as_str(), token) {
                (QUOTE_TYPE_NAME, Token::Quote { value, .. }) => {
                    let id = self.parent.create_data(
                        self.parent.gen_name(), 
                        value.as_bytes().to_vec());

                    let local_id = self
                        .parent
                        .module
                        .declare_data_in_func(id, self.builder.func);

                    let pointer = self.parent.pointer_type();
                    self.builder.ins().symbol_value(pointer, local_id)
                },
    
                (NUMBER_TYPE_NAME, Token::Number { value, .. }) => {
                    self.builder.ins().f64const(value)
                },
    
                (LIST_TYPE_NAME, Token::List { value, .. }) => {
                    todo!()
                },
    
                (FUNC_TYPE_NAME, Token::Function { sig, body, .. }) => {
                    todo!()
                },
    
                _ => unreachable!()
            }
    
        }
        else { unreachable!() }
    }

    fn ins_malloc(&mut self, size: Value, builder: &mut FunctionBuilder) -> Value {
        let malloc = self.parent.get_func_by_name("malloc");
        
        let local_callee = self
            .parent
            .module
            .declare_func_in_func(malloc, &mut builder.func);

        let call = builder.ins().call(local_callee, &[size]);
        builder.inst_results(call)[0]
    }
}


impl Into<cranelift::prelude::Type> for Type {
    fn into(self) -> cranelift::prelude::Type {
        match self {
            Type::Kind(name, _) if name == "str" => cranelift::prelude::types::I64, // TODO Maybe assuming the pointer type isn't the best idea...

            Type::Kind(name, _) if name == "list" => cranelift::prelude::types::I64,

            Type::Kind(name, _) if name == "fun" => cranelift::prelude::types::I64,

            Type::Kind(name, _) if name == "num" => cranelift::prelude::types::F64,

            Type::Kind(_, _) => panic!("Unknown type"), // TODO Report error

            Type::Variable(_, _) => panic!("Variables not allowed"),
        }
    }
}