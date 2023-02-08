use std::collections::HashMap;

use cranelift::prelude::{FunctionBuilder, Value, Variable, EntityRef, InstBuilder};
use cranelift_module::Module;

use crate::{parser::{node::{Node, Literal}, types::types::Type}, error::{Error, error}};

use super::{codegen::CodeGen, pointer_type};


pub struct FunctionTranslator<'a> {
    builder: FunctionBuilder<'a>,

    variables: HashMap<String, Variable>,

    stack: Vec<Value>
}

impl<'a> FunctionTranslator<'a> {
    pub fn new<M: Module>(parent: &mut CodeGen<M>) -> Self {
        FunctionTranslator { 
            builder: FunctionBuilder::new(&mut parent.ctx.func, &mut parent.builder_context), 
            variables: HashMap::new(),
            stack: Vec::new()
        }
    }

    pub fn translate_node<M: Module>(&mut self, node: Node, parent: &mut CodeGen<M>) -> Result<(), Error> {
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
    
            Node::Call { name, arguments, .. } => {
                let return_vals = self.ins_call(name, arguments.len(), parent)?;
                self.stack.extend_from_slice(return_vals);
            },
    
            Node::Literal { typ, value, .. } => {
                let val = self.build_literal(typ, value, parent)?;
                self.stack.push(val);
            }
        }

        Ok(())
    }

    fn build_literal<M: Module>(&mut self, typ: Type, literal: Literal, parent: &mut CodeGen<M>) -> Result<Value, Error> {
        if let Type::Kind(typ_name, _type_vars) = typ {
            match (typ_name.as_str(), literal) {
                (QUOTE_TYPE_NAME, Literal::Quote(value)) => {
                    let id = parent.create_data(
                        parent.gen_name("s"),
                        value.as_bytes().to_vec()
                    )?;

                    let local_id = parent
                        .module
                        .declare_data_in_func(id, self.builder.func);

                    let pointer = pointer_type();
                    Ok(self.builder.ins().symbol_value(pointer, local_id))
                },
    
                (NUMBER_TYPE_NAME, Literal::Number(value)) => {
                    Ok(self.builder.ins().f64const(value))
                },

                (LIST_TYPE_NAME, Literal::List(ast)) => {
                    todo!() // TODO No fcking clue how to translate this one..
                },
    
                (FUNC_TYPE_NAME, Literal::Function(ast)) => {
                    let func = parent.translate(None, ast)?;

                    let local_callee = parent
                        .module
                        .declare_func_in_func(func, self.builder.func);

                    Ok(self.builder.ins().func_addr(pointer_type(), local_callee))
                },
    
                _ => unreachable!()
            }
    
        }
        else { unreachable!() }
    }

    fn ins_call<S: AsRef<str>, M: Module>(&mut self, name: S, args_len: usize, parent: &mut CodeGen<M>) -> Result<&[Value], Error> {
        let func_id = parent.get_func_by_name(name.as_ref())?;
        
        let local_callee = parent
            .module
            .declare_func_in_func(func_id, &mut self.builder.func);

        let slice = &self.stack[args_len..];
        let call = self.builder.ins().call(local_callee, slice);

        Ok(self.builder.inst_results(call))
    }
}