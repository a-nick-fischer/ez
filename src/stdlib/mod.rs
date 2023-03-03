pub mod functions;
pub mod library;
pub mod macros;

use cranelift::prelude::InstBuilder;
use cranelift::prelude::*;
use cranelift_module::Module;

use crate::library;

use self::library::Library;

pub fn create_stdlib<M: Module + 'static>() -> Library<M> {
    library! {
        functions {
            native fn malloc("num -- pointer");
            native fn puts("cstr -- ");

            mezzaine fn cstr("str -- cstr")|trans, builder|{
                let top = trans.pop_node();
                
                // Skip the first 8 bytes 'cause they store the size
                let res = builder.ins().iadd_imm(top, Imm64::from(8));

                trans.push_node(res);
                
                Ok(())
            };

            #[inline]
            ez fn swap("num num -- num num") r#"
                b a b: a:
            "#;
        }

        transformations {
            // We implicitly assume the last elem is the jitstate
            // Yes bad things will happen if this is not the case...
            transform [Node::Call { name, .. }] if name == "save" => |trans, builder|{
                // Get the jitstate
                let (jitstate, stack) = trans.stack.split_last().unwrap();
                
                // We do not have struct so we have to break it down by ourselves.. See RawJitState
                let stack_ptr = builder.ins().load(types::I64, MemFlags::trusted(), *jitstate, 0);
                //let vars_ptr = builder.ins().load(types::I64, MemFlags::trusted(), *jitstate, 8);

                // Copy the stack from the Stack2SSA-Pass to the stack pointer
                for (offset, val) in stack.iter().enumerate() {
                    builder.ins().store(
                        MemFlags::trusted(), // The memory is aligned & does not trap (hopefully)
                        *val,                // The thing we want to store
                        stack_ptr,           // The base pointer
                        (offset * 8) as i32  // The offset (element index * element size) 
                    );
                }

                // TODO Save vars
            };
        }
    }
}
