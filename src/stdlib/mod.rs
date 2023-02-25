mod functions;
mod library;
mod macros;

use crate::library;

fn test(){
    library! {
        native fn malloc("num -- pointer");

        mezzaine fn toint("num -- int")|trans, builder|{
            Ok(())
        }
    }
}
