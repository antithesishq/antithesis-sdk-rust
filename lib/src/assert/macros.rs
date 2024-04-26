/// Assert that condition is true every time this function is called, AND that it is 
/// called at least once. This test property will be viewable in the "Antithesis SDK: Always" 
/// group of your triage report.
#[macro_export]
macro_rules! always {
    ($condition:expr, $message:literal, $details:expr) => {

{{
        fn f(){}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        static NAME: Lazy<&'static str> = Lazy::new(|| type_name_of(f));
        static FUN_NAME: Lazy<&'static str> = Lazy::new(|| &NAME[..NAME.len() - 3]);

        #[linkme::distributed_slice(ANTITHESIS_CATALOG)]
        static ALWAYS_CATALOG_ITEM: $crate::assert::CatalogInfo = $crate::assert::CatalogInfo{
            assert_type: concat!("always"),
            display_type: concat!("Always"),
            condition: false,
            message: $message,
            class: concat!(module_path!()),
            function: &FUN_NAME, /* function: &Lazy<&str> */
            file: concat!(file!()),
            begin_line: line!(),
            begin_column: column!(),
            must_hit: true,
            id: concat!($message)
        };

        let maybe_function = Lazy::get(&FUN_NAME);
        let function = *maybe_function.unwrap_or(&"anonymous");
        assert_impl(
            "always", /* assert_type */ 
            "Always", /* display_type */ 
            $condition, /* condition */
            $message, /* message */
            module_path!(), /* class */
            function, /* function */
            file!(), /* file */ 
            line!(), /* line */
            column!(), /* column */
            true,/* hit */ 
            true, /* must-hit */ 
            $message, /* id */ 
            $details /* details */
        )
}};

    } // arm ($condition:expr, $message:literal, $details:expr)
}

/// Assert that condition is true every time this function is called. Unlike the Always 
/// function, the test property spawned by AlwaysOrUnreachable will not be marked as 
/// failing if the function is never invoked. This test property will be viewable in 
/// the "Antithesis SDK: Always" group of your triage report.
#[macro_export]
macro_rules! always_or_unreachable {
    ($condition:expr, $message:literal, $details:expr) => {

{{
        fn f(){}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        static NAME: Lazy<&'static str> = Lazy::new(|| type_name_of(f));
        static FUN_NAME: Lazy<&'static str> = Lazy::new(|| &NAME[..NAME.len() - 3]);

        #[linkme::distributed_slice(ANTITHESIS_CATALOG)]
        static ALWAYS_CATALOG_ITEM: $crate::assert::CatalogInfo = $crate::assert::CatalogInfo{
            assert_type: concat!("always"),
            display_type: concat!("AlwaysOrUnreachable"),
            condition: false,
            message: $message,
            class: concat!(module_path!()),
            function: &FUN_NAME, /* function: &Lazy<&str> */
            file: concat!(file!()),
            begin_line: line!(),
            begin_column: column!(),
            must_hit: false,
            id: concat!($message)
        };

        let maybe_function = Lazy::get(&FUN_NAME);
        let function = *maybe_function.unwrap_or(&"anonymous");
        assert_impl(
            "always", /* assert_type */ 
            "AlwaysOrUnreachable", /* display_type */ 
            $condition, /* condition */
            $message, /* message */
            module_path!(), /* class */
            function, /* function */
            file!(), /* file */ 
            line!(), /* line */
            column!(), /* column */
            true,/* hit */ 
            false, /* must-hit */ 
            $message, /* id */ 
            $details /* details */
        )
}};

    } // arm ($condition:expr, $message:literal, $details:expr)
}

/// Assert that condition is true at least one time that this function was called. 
/// The test property spawned by Sometimes will be marked as failing if this function 
/// is never called, or if condition is false every time that it is called. This 
/// test property will be viewable in the "Antithesis SDK: Sometimes" group.
#[macro_export]
macro_rules! sometimes {
    ($condition:expr, $message:literal, $details:expr) => {

{{
        fn f(){}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        static NAME: Lazy<&'static str> = Lazy::new(|| type_name_of(f));
        static FUN_NAME: Lazy<&'static str> = Lazy::new(|| &NAME[..NAME.len() - 3]);

        #[linkme::distributed_slice(ANTITHESIS_CATALOG)]
        static SOMETIMES_CATALOG_ITEM: $crate::assert::CatalogInfo = $crate::assert::CatalogInfo{
            assert_type: concat!("sometimes"),
            display_type: concat!("Sometimes"),
            condition: false,
            message: $message,
            class: concat!(module_path!()),
            function: &FUN_NAME, /* function: &Lazy<&str> */
            file: concat!(file!()),
            begin_line: line!(),
            begin_column: column!(),
            must_hit: true,
            id: concat!($message)
        };

        let maybe_function = Lazy::get(&FUN_NAME);
        let function = *maybe_function.unwrap_or(&"anonymous");
        assert_impl(
            "sometimes", /* assert_type */ 
            "Sometimes", /* display_type */ 
            $condition, /* condition */
            $message, /* message */
            module_path!(), /* class */
            function, /* function */
            file!(), /* file */ 
            line!(), /* line */
            column!(), /* column */
            true,/* hit */ 
            true, /* must-hit */ 
            $message, /* id */ 
            $details /* details */
        )
}};

    } // arm ($condition:expr, $message:literal, $details:expr)
}

/// Assert that a line of code is reached at least once. The test property spawned by 
/// Reachable will be marked as failing if this function is never called. This test 
/// property will be viewable in the "Antithesis SDK: Reachablity assertions" group.
#[macro_export]
macro_rules! reachable {
    ($message:literal, $details:expr) => {

{{
        fn f(){}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        static NAME: Lazy<&'static str> = Lazy::new(|| type_name_of(f));
        static FUN_NAME: Lazy<&'static str> = Lazy::new(|| &NAME[..NAME.len() - 3]);

        #[linkme::distributed_slice(ANTITHESIS_CATALOG)]
        static REACHABILITY_CATALOG_ITEM: $crate::assert::CatalogInfo = $crate::assert::CatalogInfo{
            assert_type: concat!("reachability"),
            display_type: concat!("Reachable"),
            condition: false,
            message: $message,
            class: concat!(module_path!()),
            function: &FUN_NAME, /* function: &Lazy<&str> */
            file: concat!(file!()),
            begin_line: line!(),
            begin_column: column!(),
            must_hit: true,
            id: concat!($message)
        };

        let maybe_function = Lazy::get(&FUN_NAME);
        let function = *maybe_function.unwrap_or(&"anonymous");
        assert_impl(
            "reachability", /* assert_type */ 
            "Reachable", /* display_type */ 
            true, /* condition */
            $message, /* message */
            module_path!(), /* class */
            function, /* function */
            file!(), /* file */ 
            line!(), /* line */
            column!(), /* column */
            true,/* hit */ 
            true, /* must-hit */ 
            $message, /* id */ 
            $details /* details */
        )
}};

    } // arm ($message:literal, $details:expr)
}


/// Assert that a line of code is never reached. The test property spawned by Unreachable 
/// will be marked as failing if this function is ever called. This test property will 
/// be viewable in the "Antithesis SDK: Reachablity assertions" group.
#[macro_export]
macro_rules! unreachable {
    ($message:literal, $details:expr) => {

{{
        fn f(){}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        static NAME: Lazy<&'static str> = Lazy::new(|| type_name_of(f));
        static FUN_NAME: Lazy<&'static str> = Lazy::new(|| &NAME[..NAME.len() - 3]);

        #[linkme::distributed_slice(ANTITHESIS_CATALOG)]
        static REACHABILITY_CATALOG_ITEM: $crate::assert::CatalogInfo = $crate::assert::CatalogInfo{
            assert_type: concat!("reachability"),
            display_type: concat!("Unreachable"),
            condition: false,
            message: $message,
            class: concat!(module_path!()),
            function: &FUN_NAME, /* function: &Lazy<&str> */
            file: concat!(file!()),
            begin_line: line!(),
            begin_column: column!(),
            must_hit: false,
            id: concat!($message)
        };

        let maybe_function = Lazy::get(&FUN_NAME);
        let function = *maybe_function.unwrap_or(&"anonymous");
        assert_impl(
            "reachability", /* assert_type */ 
            "Unreachable", /* display_type */ 
            false, /* condition */
            $message, /* message */
            module_path!(), /* class */
            function, /* function */
            file!(), /* file */ 
            line!(), /* line */
            column!(), /* column */
            true,/* hit */ 
            false, /* must-hit */ 
            $message, /* id */ 
            $details /* details */
        )
}};

    } // arm ($message:literal, $details:expr)
}

