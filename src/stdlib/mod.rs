pub mod functions;
pub mod library;
pub mod macros;

use cranelift::prelude::InstBuilder;
use cranelift_module::Module;
use cranelift::prelude::*;

use crate::library;

use self::library::Library;

pub fn create_stdlib<M: Module>() -> Library<M> {
    library! {
        functions {
            native fn malloc("num -- pointer");
            native fn puts("cstr -- ");

            mezzaine fn toint("num -- cint")|trans, builder|{
                let top = trans.stack.pop().unwrap();
                
                let res = builder.ins().sextend(types::I64, top);

                trans.stack.push(res);
                
                Ok(())
            };

            #[inline]
            ez fn swap("a b -- b a") r#"
                b a b: a:
            "#;
        }

        transformations {}
    }
}
