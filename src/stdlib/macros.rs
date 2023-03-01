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
    (functions { $($func:tt)* } transformations { $($transf:tt);* }) => {
        {
            use cranelift_module::Module;

            use std::collections::HashMap;

            use $crate::stdlib::functions::*;
            use $crate::parser::types::typ::Type;
            use $crate::codegen::function_translator::{FunctionTranslator, FunctionOptions};
            use $crate::error::Error;
            use $crate::parser::signature_parser::TypedSignature;
            use $crate::parser::types::type_env::TypeEnv;

            use $crate::stdlib::library::Library;
            use $crate::__gen_funcs;

            let mut library = Library::new();
            let mut tenv = TypeEnv::new(&HashMap::new());

             __gen_funcs!(library, tenv, $($func)*);

            library
        }
    };
}

#[macro_export]
macro_rules! __gen_funcs {
    ($library:ident, $tenv:ident, native fn $name:ident ($sig:literal); $($tail:tt)*) => {
        let func = NativeFun::new(stringify!($name), format!("({})", $sig).as_str()).unwrap();
        let name = <NativeFun<'_> as EzFun<M>>::name(&func).to_string();
        let sig: Type = <NativeFun<'_> as EzFun<M>>::signature(&func).into();

        $tenv.bindings.insert(name.clone(), sig.clone());
        $library.bindings.insert(name, sig);
        $library.transformations.push(Box::new(func));

        __gen_funcs!($library, $tenv, $($tail)*)
    };

    ($library:ident, $tenv:ident, mezzaine fn $name:ident ($sig:literal) $blk:expr; $($tail:tt)*) => {
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
                format!("({})", $sig).parse().unwrap()
            }
        }

        let func = $name {};
        let name = <$name as EzFun<M>>::name(&func).to_string();
        let sig: Type = <$name as EzFun<M>>::signature(&func).into();

        $tenv.bindings.insert(name.clone(), sig.clone());
        $library.bindings.insert(name, sig);
        $library.transformations.push(Box::new(func));

        __gen_funcs!($library, $tenv, $($tail)*)
    };

    ($library:ident, $tenv:ident, #[inline] ez $($content:tt)*) => {
        __gen_funcs!($library, $tenv, true, ez_int $($content)*)
    };

    ($library:ident, $tenv:ident, ez $($content:tt)*) => {
        __gen_funcs!($library, $tenv, false, ez_int $($content)*)
    };

    ($library:ident, $tenv:ident, $inline:literal, ez_int fn $name:ident ($sig:literal) $src:expr; $($tail:tt)*) => {
        let sig = format!("({})", $sig);
        let name = stringify!($name);
        let src = stringify!($src);

        println!("{}", src);

        let func = if $inline {
            UserFun::new_inline(name, sig.clone().as_str(), src, &mut $tenv).unwrap()
        }
        else {
            UserFun::new(name, sig.clone().as_str(), src, &mut $tenv).unwrap()
        };
        
        
        let name = <UserFun<'_> as EzFun<M>>::name(&func).to_string();
        let sig: Type = <UserFun<'_> as EzFun<M>>::signature(&func).into();

        $tenv.bindings.insert(name.clone(), sig.clone());
        $library.bindings.insert(name, sig.clone());
        $library.transformations.push(Box::new(func));

        __gen_funcs!($library, $tenv, $($tail)*)
    };

    ($library:ident, ) => {};

    ($library:ident, $tenv:ident, ) => {};

    () => {};
}