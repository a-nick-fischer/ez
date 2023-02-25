#[macro_export]
macro_rules! match_nodes {
    ($nodes:ident ($arity:literal): $match:pat if $cond:expr => $blk:block) => {
        match &$nodes.clone()[..] {
            $match if $cond => {
                $nodes.truncate($arity);

                $blk

                Ok(true)
            },

            _ => Ok(false)
        }
    }
}

#[macro_export]
macro_rules! library {
    (functions { $($typ:ident fn $name:ident ($sig:literal))*; } transformations { $($transf:tt);* }) => {
        {
            use $crate::stdlib::library::Library;
            use $crate::__gen_func;

            let mut library = Library::new();

            $(
                __gen_func!(library, $typ fn $name ($sig));
            )*

            library
        }
    };
}

#[macro_export]
macro_rules! __gen_func {
    ($library:ident, native fn $name:ident ($sig:literal)) => {
        use $crate::stdlib::functions::NativeFun;
        use $crate::stdlib::functions::EzFun;
        use $crate::parser::types::typ::Type;

        let func = NativeFun::new(stringify!($name), $sig).unwrap();
        let name = <NativeFun<'_> as EzFun<M>>::name(&func).to_string();
        let sig: Type = <NativeFun<'_> as EzFun<M>>::signature(&func).into();

        $library.bindings.insert(name, sig);
        $library.transformations.push(Box::new(func));
    };

    ($library:ident, mezzaine fn $name:ident ($sig:literal) $blk:expr) => {
        use cranelift_module::Module;

        use $crate::codegen::function_translator::{FunctionTranslator, FunctionOptions};
        use $crate::stdlib::functions::EzFun;
        use $crate::error::Error;
        use $crate::parser::signature_parser::TypedSignature;

        #[allow(non_camel_case_types)]
        struct $name;

        impl<M: Module> EzFun<M> for $name {
            fn init(&self, codegen: &mut $crate::codegen::codegen_module::CodeGenModule<M>) -> Result<(), Error> {
                let sig = <$name as EzFun<M>>::signature(self);
                let name = <$name as EzFun<M>>::name(self);

                FunctionTranslator::new(codegen)
                    .with_signature(sig)
                    .with_body_generator($blk)?
                    .finish_func(name, FunctionOptions::internal())?;

                Ok(())
            }
        
            #[inline]
            fn name(&self) -> &str {
                stringify!($name)
            }
        
            #[inline]
            fn signature(&self) -> TypedSignature {
                $sig.parse().unwrap()
            }
        };

        $name {}
    };
}
