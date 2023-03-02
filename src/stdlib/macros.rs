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
    (functions { $($func:tt)* } transformations { $($transf:tt)* }) => {
        {
            use cranelift_module::Module;
            use std::rc::Rc;

            use $crate::stdlib::functions::*;
            use $crate::parser::types::typ::Type;
            use $crate::codegen::function_translator::{FunctionTranslator, FunctionOptions};
            use $crate::error::Error;
            use $crate::parser::node::Node;
            use $crate::parser::signature_parser::TypedSignature;

            use $crate::stdlib::library::Library;
            use $crate::__gen_transforms;
            use $crate::__gen_funcs;
            use $crate::match_nodes;

            let mut library = Library::new();

            __gen_transforms!(library, $($transf)*);
            __gen_funcs!(library, $($func)*);

            library
        }
    };
}

#[macro_export]
macro_rules! __gen_transforms {
    ($library:ident, transform $match:pat if $cond:expr => $blk:block; $($tail:tt)*) => {
        struct Temp;

        impl<M: Module> CodeTransformation<M> for Temp {
            fn try_apply<'b>(
                &self,
                nodes: &mut Vec<Node>,
                translator: &mut FunctionTranslator<'b, M>,
                builder: &mut FunctionBuilder
            ) -> Result<bool, Error> {
                match_nodes!(nodes(0): $match if $cond => $blk)
            }
        }

        $library.transformations.push(Temp {});

        __gen_transforms!($library, $($tail)*)
    };

    ($library:ident, ) => {}
}

#[macro_export]
macro_rules! __gen_funcs {
    ($library:ident, native fn $name:ident ($sig:literal); $($tail:tt)*) => {
        let func = NativeFun::new(stringify!($name), format!("({})", $sig).as_str()).unwrap();
        let name = <NativeFun<'_> as EzFun<M>>::name(&func).to_string();
        let sig: Type = <NativeFun<'_> as EzFun<M>>::signature(&func).into();

        let funcrc = Rc::new(func);
        $library.bindings.insert(name, sig);
        $library.functions.push(funcrc.clone());
        $library.transformations.push(funcrc);

        __gen_funcs!($library, $($tail)*)
    };

    ($library:ident, mezzaine fn $name:ident ($sig:literal) $blk:expr; $($tail:tt)*) => {
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

        let funcrc = Rc::new(func);
        $library.bindings.insert(name, sig);
        $library.functions.push(funcrc.clone());
        $library.transformations.push(funcrc);

        __gen_funcs!($library, $($tail)*)
    };

    ($library:ident, #[inline] ez $($content:tt)*) => {
        __gen_funcs!($library, true, ez_int $($content)*)
    };

    ($library:ident, ez $($content:tt)*) => {
        __gen_funcs!($library, false, ez_int $($content)*)
    };

    ($library:ident, $inline:literal, ez_int fn $name:ident ($sig:literal) $src:expr; $($tail:tt)*) => {
        let sig = format!("({})", $sig);
        let name = stringify!($name);
        let src = $src;

        let mut tenv = $library.type_env();
        tenv.stack = sig.parse::<TypedSignature>()
            .unwrap()
            .arguments()
            .clone();

        let func = if $inline {
            UserFun::new_inline(name, sig.clone().as_str(), src, &mut tenv).unwrap()
        }
        else {
            UserFun::new(name, sig.clone().as_str(), src, &mut tenv).unwrap()
        };
        
        
        let name = <UserFun<'_> as EzFun<M>>::name(&func).to_string();
        let sig: Type = <UserFun<'_> as EzFun<M>>::signature(&func).into();

        let funcrc = Rc::new(func);
        $library.bindings.insert(name, sig);
        $library.functions.push(funcrc.clone());
        $library.transformations.push(funcrc);

        __gen_funcs!($library, $($tail)*)
    };

    ($library:ident, ) => {};

    () => {};
}