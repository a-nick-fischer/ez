use cranelift::prelude::*;
use cranelift::{codegen, prelude::{FunctionBuilderContext, FunctionBuilder, InstBuilder}};
use cranelift_module::{Module, DataContext, Linkage, DataId};
use uuid::Uuid;

use crate::{parser::{node::Node, types::{types::Type, *}}, lexer::token::Token};

pub mod compiler;
pub mod jit;

struct Translator {
    builder_context: FunctionBuilderContext,

    ctx: codegen::Context,

    data_ctx: DataContext,

    module: Box<dyn Module>
}

impl Translator {
    fn translate(&mut self, nodes: Vec<Node>) {
        for node in nodes {
            self.translate_node(node, builder);
        }
    }

    fn translate_node(&mut self, node: Node, builder: &mut FunctionBuilder) {
        match node {
            Node::Assigment { name, token, typ } => {
                todo!()
            },
    
            Node::Variable { name, typ, .. } => {
                todo!()
            },
    
            Node::Call { name, .. } => { 
                todo!() 
            },
    
            Node::Literal { typ, token } => {
                self.build_literal(typ, token, builder);
            }
        }
    }
    
    fn build_literal(&mut self, typ: Type, token: Token, builder: &mut FunctionBuilder) -> Value {
        if let Type::Kind(typ_name, _type_vars) = typ {
            match (typ_name.as_str(), token) {
                (QUOTE_TYPE_NAME, Token::Quote { value, .. }) => {
                    let name = Uuid::new_v4().to_string();
                    let id = self.create_data(name, value.as_bytes().to_vec());

                    let local_id = self
                        .module
                        .declare_data_in_func(id, builder.func);

                    let pointer = self.module.target_config().pointer_type();
                    builder.ins().symbol_value(pointer, local_id)
                },
    
                (NUMBER_TYPE_NAME, Token::Number { value, .. }) => {
                    builder.ins().f32const(value)
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
}
