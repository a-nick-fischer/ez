use std::collections::HashMap;

use cranelift::prelude::*;
use cranelift::{codegen, prelude::{FunctionBuilderContext, FunctionBuilder, InstBuilder}};
use cranelift_module::{Module, DataContext, Linkage, DataId, FuncId, FuncOrDataId, ModuleError};

use crate::error::Error;
use crate::parser::node::Literal;
use crate::{parser::{node::Node, types::{types::Type, *}}};

pub struct Translator<M: Module> {
    pub builder_context: FunctionBuilderContext,

    pub ctx: codegen::Context,

    pub data_ctx: DataContext,

    pub module: M,

    pub naming_idx: u32
}

impl<M: Module> Translator<M> {
    pub fn translate(&mut self, name: Option<&str>, nodes: Vec<Node>) -> Result<FuncId, Vec<Error>> {
        let mut tran = FunctionTranslator::new(self);
        
        for node in nodes {
            tran.translate_node(node)?;
        }

        let sig = self.module.make_signature();
        let id = self.create_func(name, sig)?;

        Ok(id)
    }

    fn create_data(&mut self, name: String, content: Vec<u8>) -> Result<DataId, Vec<Error>> {
        self.data_ctx.define(content.into_boxed_slice());

        let id = self
            .module
            .declare_data(&name, Linkage::Export, true, false)
            .map_err(|err| vec![err.into()])?;

        self.module
            .define_data(id, &self.data_ctx)
            .map_err(|err| vec![err.into()])?;

        // self.data_ctx.clear(); // TODO Needed?

        Ok(id)
    }

    fn create_func(&mut self, maybe_name: Option<&str>, sig: Signature) -> Result<FuncId, Vec<Error>> {
        let id = 
            if let Some(name) = maybe_name {
                self
                    .module
                    .declare_function(name, Linkage::Export, &sig)
                    .map_err(|err| vec![err.into()])?
            }
            else {
                self
                    .module
                    .declare_anonymous_function(&sig)
                    .map_err(|err| vec![err.into()])?
            };
        
        self.module
            .define_function(id, &mut self.ctx)
            .map_err(|err| vec![err.into()])?;

        Ok(id)
    }

    fn get_func_by_name(&self, name: &str) -> Result<FuncId, Vec<Error>> {
        let maybe_func = self
            .module
            .declarations()
            .get_name(name);

        match maybe_func {
            Some(FuncOrDataId::Func(id)) => Ok(id),

            _ => Err(error(format!("{name} not a function - yes this is a compiler bug"))),
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
            .expect("problem declaring function"); // TODO Error handling or move to STDLIB
    }
}

struct FunctionTranslator<'a, M: Module> {
    builder: FunctionBuilder<'a>,

    parent: &'a mut Translator<M>,

    variables: HashMap<String, Variable>,

    stack: Vec<Value>
}

impl<'a, M: Module> FunctionTranslator<'a, M> {
    fn new(parent: &'a mut Translator<M>) -> Self {
        FunctionTranslator { 
            builder: FunctionBuilder::new(&mut parent.ctx.func, &mut parent.builder_context), 
            parent,
            variables: HashMap::new(),
            stack: Vec::new()
        }
    }

    fn translate_node(&mut self, node: Node) -> Result<(), Vec<Error>> {
        match node {
            Node::Assigment { name, .. } => {
                let var = Variable::new(self.variables.len());
                self.variables.insert(name, var);

                let content = self.stack.pop().unwrap();
                self.builder.def_var(var, content);
            },
    
            Node::Variable { name, .. } => {
                let var = self.variables.get(&name)
                    .ok_or(error(format!("Variable {name} not found - yes this is a compiler bug")))?;

                let val = self.builder.use_var(*var);
                self.stack.push(val);
            },
    
            Node::Call { name, arguments, returns, .. } => {
                let mut sig = self.parent.module.make_signature();

                for arg in arguments.vec() {
                    sig.params.push(AbiParam::new(arg.clone().into()))
                }

                for ret in returns.vec() {
                    sig.returns.push(AbiParam::new(ret.clone().into()))
                }

                let func = self.parent.get_func_by_name(name.as_str())?;

                let local_callee = self
                    .parent
                    .module
                    .declare_func_in_func(func, self.builder.func);

                let slice = &self.stack[arguments.len()..];
                let inst = self.builder.ins().call(local_callee, slice);

                let returns = self.builder.inst_results(inst);
                self.stack.extend_from_slice(returns);
            },
    
            Node::Literal { typ, value, .. } => {
                let val = self.build_literal(typ, value)?;
                self.stack.push(val);
            }
        }

        Ok(())
    }

    fn build_literal(&mut self, typ: Type, literal: Literal) -> Result<Value, Vec<Error>> {
        if let Type::Kind(typ_name, _type_vars) = typ {
            match (typ_name.as_str(), literal) {
                (QUOTE_TYPE_NAME, Literal::Quote(value)) => {
                    let name = self.parent.gen_name();
                    let id = self.parent.create_data(
                        name,
                        value.as_bytes().to_vec()
                    )?;

                    let local_id = self
                        .parent
                        .module
                        .declare_data_in_func(id, self.builder.func);

                    let pointer = self.parent.pointer_type();
                    Ok(self.builder.ins().symbol_value(pointer, local_id))
                },
    
                (NUMBER_TYPE_NAME, Literal::Number(value)) => {
                    Ok(self.builder.ins().f64const(value))
                },

                (LIST_TYPE_NAME, Literal::List(ast)) => {
                    todo!() // TODO No fcking clue how to translate this one..
                },
    
                (FUNC_TYPE_NAME, Literal::Function(ast)) => {
                    let func = self.parent.translate(None, ast)?;

                    let local_callee = self
                        .parent
                        .module
                        .declare_func_in_func(func, self.builder.func);

                    Ok(self.builder.ins().func_addr(self.parent.pointer_type(), local_callee))
                },
    
                _ => unreachable!()
            }
    
        }
        else { unreachable!() }
    }
    
    fn ins_malloc(&mut self, size: Value) -> Result<Value, Vec<Error>> {
        let malloc = self.parent.get_func_by_name("malloc")?;
        
        let local_callee = self
            .parent
            .module
            .declare_func_in_func(malloc, &mut self.builder.func);

        let call = self.builder.ins().call(local_callee, &[size]);
        Ok(self.builder.inst_results(call)[0])
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

fn error(message: String) -> Vec<Error> {
    vec![Error::GeneralError { message }]
}

impl From<ModuleError> for Error {
    fn from(value: ModuleError) -> Self {
        Error::GeneralError { message: value.to_string() }
    }
}