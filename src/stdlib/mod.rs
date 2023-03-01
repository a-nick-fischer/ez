mod functions;
mod library;
mod macros;

use cranelift_module::Module;

use crate::library;

use self::library::Library;

pub fn create_stdlib<M: Module>() -> Library<M> {
    let lib: Library<M> = library! {
        functions {
            native fn malloc("num -- pointer");
            native fn puts("str -- ");

            mezzaine fn toint("num -- int")|trans, builder|{
                Ok(())
            };

            #[inline]
            ez fn test("bla -- bla") "";
        }

        transformations {}
    };

    println!("{:?}", lib.bindings);

    lib
}
