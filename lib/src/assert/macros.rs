// use linkme::distributed_slice;

        // #[distributed_slice(ANTITHESIS_CATALOG)]
#[macro_export]
macro_rules! always {
    ($condition:expr, $message:literal, $details:expr, $x:ident) => {

        // assert_catalog!({
        //     assert_type: "always",
        //     display_type: "Always",
        //     message: $message,
        // });

        static ALWAYS_241: antithesis_sdk_rust::assert::CatalogInfo = antithesis_sdk_rust::assert::CatalogInfo{
            assert_type: concat!("always"),
            display_type: concat!("Always"),
            condition: false,
            message: $message,
            class: concat!(module_path!()),
            function: concat!("yes"),
            file: concat!(file!()),
            begin_line: line!(), /* line */
            begin_column: column!(), /* column */
            must_hit: true, /* must-hit */ 
            id: concat!($message) /* id */ 
        };

        assert_impl(
            "always", /* assert_type */ 
            "Always", /* display_type */ 
            $condition, /* condition */
            $message, /* message */
            module_path!(), /* class */

            { // taken from function!() in https://crates.io/crates/stdext
                fn f(){}
                fn type_name_of<T>(_: T) -> &'static str {
                    std::any::type_name::<T>()
                }
                let name = type_name_of(f);
                &name[..name.len() - 3]
            }, /* function */

            file!(), /* file */ 
            line!(), /* line */
            column!(), /* column */
            true,/* hit */ 
            true, /* must-hit */ 
            $message, /* id */ 
            $details /* details */
        )
    }
}

#[macro_export]
macro_rules! always_or_unreachable {
    ($condition:expr, $message:literal, $details:expr) => {

        assert_impl(
            "always", /* assert_type */ 
            "AlwaysOrUnreachable", /* display_type */ 
            $condition, /* condition */
            $message, /* message */
            module_path!(), /* class */

            { // taken from function!() in https://crates.io/crates/stdext
                fn f(){}
                fn type_name_of<T>(_: T) -> &'static str {
                    std::any::type_name::<T>()
                }
                let name = type_name_of(f);
                &name[..name.len() - 3]
            }, /* function */


            file!(), /* file */ 
            line!(), /* line */
            column!(), /* column */
            true,/* hit */ 
            false, /* must-hit */ 
            $message, /* id */ 
            $details /* details */
        )
    }
}

#[macro_export]
macro_rules! sometimes {
    ($condition:expr, $message:literal, $details:expr) => {
        assert_impl(
            "sometimes", /* assert_type */ 
            "Sometimes", /* display_type */ 
            $condition, /* condition */
            $message, /* message */
            module_path!(), /* class */

            { // taken from function!() in https://crates.io/crates/stdext
                fn f(){}
                fn type_name_of<T>(_: T) -> &'static str {
                    std::any::type_name::<T>()
                }
                let name = type_name_of(f);
                &name[..name.len() - 3]
            }, /* function */


            file!(), /* file */ 
            line!(), /* line */
            column!(), /* column */
            true,/* hit */ 
            true, /* must-hit */ 
            $message, /* id */ 
            $details /* details */
        )
    }
}

#[macro_export]
macro_rules! reachable {
    ($message:literal, $details:expr) => {
        assert_impl(
            "reachability", /* assert_type */ 
            "Reachable", /* display_type */ 
            true, /* condition */
            $message, /* message */
            module_path!(), /* class */

            { // taken from function!() in https://crates.io/crates/stdext
                fn f(){}
                fn type_name_of<T>(_: T) -> &'static str {
                    std::any::type_name::<T>()
                }
                let name = type_name_of(f);
                &name[..name.len() - 3]
            }, /* function */


            file!(), /* file */ 
            line!(), /* line */
            column!(), /* column */
            true,/* hit */ 
            true, /* must-hit */ 
            $message, /* id */ 
            $details /* details */
        )
    }
}

#[macro_export]
macro_rules! unreachable {
    ($message:literal, $details:expr) => {
        assert_impl(
            "reachability", /* assert_type */ 
            "Unreachable", /* display_type */ 
            true, /* condition */
            $message, /* message */
            module_path!(), /* class */

            { // taken from function!() in https://crates.io/crates/stdext
                fn f(){}
                fn type_name_of<T>(_: T) -> &'static str {
                    std::any::type_name::<T>()
                }
                let name = type_name_of(f);
                &name[..name.len() - 3]
            }, /* function */


            file!(), /* file */ 
            line!(), /* line */
            column!(), /* column */
            true,/* hit */ 
            false, /* must-hit */ 
            $message, /* id */ 
            $details /* details */
        )
    }
}

