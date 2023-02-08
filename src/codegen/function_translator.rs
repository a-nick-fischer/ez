use std::collections::HashMap;

use cranelift::prelude::{FunctionBuilder, Value, Variable};
use cranelift_module::Module;

use super::translator::Translator;


pub struct FunctionTranslator<'a> {
    builder: FunctionBuilder<'a>,

    variables: HashMap<String, Variable>,

    stack: Vec<Value>
}

impl<'a> FunctionTranslator<'a> {
    pub fn new<M: Module>(parent: &mut Translator<M>) -> Self {
        FunctionTranslator { 
            builder: FunctionBuilder::new(&mut parent.ctx.func, &mut parent.builder_context), 
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
    
            Node::Call { name, arguments, returns, .. } => {
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

    fn build_literal(&mut self, typ: Type, literal: Literal) -> Result<Value, Error> {
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
    
    fn ins_malloc(&mut self, size: Value) -> Result<Value, Error> {
        let malloc = self.parent.get_func_by_name("malloc")?;
        
        let local_callee = self
            .parent
            .module
            .declare_func_in_func(malloc, &mut self.builder.func);

        let call = self.builder.ins().call(local_callee, &[size]);
        Ok(self.builder.inst_results(call)[0])
    }
}