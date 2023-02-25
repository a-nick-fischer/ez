mod functions;
mod library;
mod macros;

use cranelift_module::Module;

use crate::library;

use self::library::Library;

fn test<M: Module>(){
    let lib: Library<M> = library! {
        functions {
            native fn malloc("num -- pointer");
            native fn puts("str -- ");

            /*mezzaine fn toint("num -- int")|trans, builder|{
                Ok(())
            }*/
        }

        transformations {}
    };
}
