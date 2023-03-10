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
            native fn malloc("cint -- pointer");
            native fn puts("cstr -- ");
            native fn exit("cint -- ");

            mezzaine fn add("num num -- num")|trans, builder|{
                let a = trans.pop_value();
                let b = trans.pop_value();
                
                let res = builder.ins().fadd(a, b);

                trans.push_value(res);
                
                Ok(())
            };

            mezzaine fn sub("num num -- num")|trans, builder|{
                let a = trans.pop_value();
                let b = trans.pop_value();
                
                let res = builder.ins().fsub(a, b);

                trans.push_value(res);
                
                Ok(())
            };

            mezzaine fn mul("num num -- num")|trans, builder|{
                let a = trans.pop_value();
                let b = trans.pop_value();
                
                let res = builder.ins().fmul(a, b);

                trans.push_value(res);
                
                Ok(())
            };

            mezzaine fn div("num num -- num")|trans, builder|{
                let a = trans.pop_value();
                let b = trans.pop_value();
                
                let res = builder.ins().fdiv(a, b);

                trans.push_value(res);
                
                Ok(())
            };

            mezzaine fn cstr("str -- cstr")|trans, builder|{
                let top = trans.pop_value();
                
                // Skip the first 8 bytes 'cause they store the size
                let res = builder.ins().iadd_imm(top, Imm64::from(8));

                trans.push_value(res);
                
                Ok(())
            };

            #[inline]
            ez fn swap("num num -- num num") r#"
                b a b: a:
            "#;

            #[inline]
            ez fn print("str -- ") r#"
                puts cstr
            "#;
        }

        transformations {
            // We implicitly assume the last elem is the jitstate
            // Yes bad things will happen if this is not the case...
            transform __save: [Node::Call { name, .. }, ..] if name == "__save" => |nodes, trans, builder|{
                // Pop the __save call
                nodes.remove(0);

                // Get the jitstate
                let (jitstate, stack) = trans.stack.split_first().unwrap();

                // Copy the stack from the Stack2SSA-Pass to the stack pointer
                for (offset, val) in stack.iter().enumerate() {
                    builder.ins().store(
                        MemFlags::new(),     // The memory is aligned & does not trap (hopefully)
                        *val,                // The thing we want to store
                        *jitstate,           // The base pointer
                        (offset * 8) as i32  // The offset (element index * element size) 
                    );
                }

                // TODO Save vars
            };

            transform __exit: [Node::Call { name, .. }, ..] if name == "__exit" => |nodes, trans, builder|{
                // Pop the __exit call
                nodes.remove(0);

                // Call exit(0)
                let status = builder.ins().iconst(types::I64, 0);
                trans.push_value(status);
                trans.ins_call("exit", 1, builder).unwrap(); // TODO Handle
            };
        }
    }
}
