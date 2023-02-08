use std::collections::HashMap;

use cranelift::prelude::{FunctionBuilder, Value, Variable, EntityRef, InstBuilder};
use cranelift_module::Module;

use crate::{parser::{node::{Node, Literal}, types::{types::Type, self}}, error::{Error, error}};

use super::{codegen::CodeGen, pointer_type};


pub struct FunctionTranslator<'a, M: Module> {
    builder: FunctionBuilder<'a>,

    codegen: &'a mut CodeGen<M>,

    variables: HashMap<String, Variable>,

    stack: Vec<Value>
}

impl<'a, M: Module> FunctionTranslator<'a, M> {
    pub fn new(codegen: &'a mut CodeGen<M>) -> Self {
        FunctionTranslator { 
            builder: FunctionBuilder::new(&mut codegen.ctx.func, &mut codegen.builder_context), 
            codegen,
            variables: HashMap::new(),
            stack: Vec::new()
        }
    }

    pub fn translate_node(&mut self, node: Node) -> Result<(), Error> {
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
    
            Node::Call { name, arguments, .. } => 
                self.ins_call(name, arguments.len())?,
    
            Node::Literal { typ, value, .. } => {
                let val = self.build_literal(typ, value)?;
                self.stack.push(val);
            }
        }

        Ok(())
    }

    fn build_literal(&mut self, typ: Type, literal: Literal) -> Result<Value, Error> {
        if let Type::Kind(typ_name, _type_vars) = typ {
            match (typ_name.as_str(), literal) {
                (types::QUOTE_TYPE_NAME, Literal::Quote(value)) => {
                    let name = self.codegen.gen_name("s");
                    
                    let id = self.codegen.create_data(
                        name,
                        value.as_bytes().to_vec()
                    )?;

                    let local_id = self
                        .codegen
                        .module
                        .declare_data_in_func(id, self.builder.func);

                    let pointer = pointer_type();
                    Ok(self.builder.ins().symbol_value(pointer, local_id))
                },
    
                (types::NUMBER_TYPE_NAME, Literal::Number(value)) => {
                    Ok(self.builder.ins().f64const(value))
                },

                (types::LIST_TYPE_NAME, Literal::List(ast)) => {
                    todo!() // TODO No fcking clue how to translate this one..
                },
    
                (types::FUNC_TYPE_NAME, Literal::Function(ast)) => {
                    let func = self.codegen.translate(None, ast)?;

                    let local_callee = self
                        .codegen
                        .module
                        .declare_func_in_func(func, self.builder.func);

                    Ok(self.builder.ins().func_addr(pointer_type(), local_callee))
                },
    
                _ => unreachable!()
            }
    
        }
        else { unreachable!() }
    }

    fn ins_call<S: AsRef<str>>(&mut self, name: S, args_len: usize) -> Result<(), Error> {
        let func_id = self.codegen.get_func_by_name(name.as_ref())?;
        
        let local_callee = self
            .codegen
            .module
            .declare_func_in_func(func_id, &mut self.builder.func);

        let slice = &self.stack[args_len..];
        let call = self.builder.ins().call(local_callee, slice);

        let results = self.builder.inst_results(call);
        self.stack.extend_from_slice(results);

        Ok(())
    }
}