#[doc(hidden)]
#[macro_export]
macro_rules! assert_helper {
    ($condition:expr, $message:literal, $details:expr, $assert_type:literal, $display_type:literal, $must_hit:literal) => {{
        use $crate::once_cell::sync::Lazy;
        fn f(){}
        fn type_name_of<T>(_: T) -> &'static str {
            ::std::any::type_name::<T>()
        }
        static NAME: Lazy<&'static str> = Lazy::new(|| type_name_of(f));
        static FUN_NAME: Lazy<&'static str> = Lazy::new(|| &NAME[..NAME.len() - 3]);

        #[$crate::linkme::distributed_slice(ANTITHESIS_CATALOG)]
        #[linkme(crate = $crate::linkme)] // Refer to our re-exported linkme.
        static ALWAYS_CATALOG_ITEM: $crate::assert::CatalogInfo = $crate::assert::CatalogInfo{
            assert_type: $assert_type,
            display_type: $display_type,
            condition: false,
            message: $message,
            class: ::std::module_path!(),
            function: &FUN_NAME, /* function: &Lazy<&str> */
            file: ::std::file!(),
            begin_line: ::std::line!(),
            begin_column: ::std::column!(),
            must_hit: $must_hit,
            id: $message
        };

        let maybe_function = Lazy::get(&FUN_NAME);
        let function = *maybe_function.unwrap_or(&"anonymous");
        // Force evaluation of expressions.
        let condition = $condition;
        let details = $details;
        assert_impl(
            $assert_type, /* assert_type */ 
            $display_type, /* display_type */ 
            condition, /* condition */
            $message, /* message */
            ::std::module_path!(), /* class */
            function, /* function */
            ::std::file!(), /* file */ 
            ::std::line!(), /* line */
            ::std::column!(), /* column */
            true,/* hit */ 
            $must_hit, /* must-hit */ 
            $message, /* id */ 
            details /* details */
        )
    }}
}

/// Assert that condition is true every time this function is called, AND that it is 
/// called at least once. This test property will be viewable in the "Antithesis SDK: Always" 
/// group of your triage report.
#[macro_export]
macro_rules! always {
    ($condition:expr, $message:literal, $details:expr) => {
        $crate::assert_helper!($condition, $message, $details, "always", "Always", true)
    }
}

/// Assert that condition is true every time this function is called. Unlike the Always 
/// function, the test property spawned by AlwaysOrUnreachable will not be marked as 
/// failing if the function is never invoked. This test property will be viewable in 
/// the "Antithesis SDK: Always" group of your triage report.
#[macro_export]
macro_rules! always_or_unreachable {
    ($condition:expr, $message:literal, $details:expr) => {
        $crate::assert_helper!($condition, $message, $details, "always", "AlwaysOrUnreachable", false)
    }
}

/// Assert that condition is true at least one time that this function was called. 
/// The test property spawned by Sometimes will be marked as failing if this function 
/// is never called, or if condition is false every time that it is called. This 
/// test property will be viewable in the "Antithesis SDK: Sometimes" group.
#[macro_export]
macro_rules! sometimes {
    ($condition:expr, $message:literal, $details:expr) => {
        $crate::assert_helper!($condition, $message, $details, "sometimes", "Sometimes", true)
    }
}

/// Assert that a line of code is reached at least once. The test property spawned by 
/// Reachable will be marked as failing if this function is never called. This test 
/// property will be viewable in the "Antithesis SDK: Reachablity assertions" group.
#[macro_export]
macro_rules! reachable {
    ($message:literal, $details:expr) => {
        $crate::assert_helper!(true, $message, $details, "reachability", "Reachable", true)
    }
}

/// Assert that a line of code is never reached. The test property spawned by Unreachable 
/// will be marked as failing if this function is ever called. This test property will 
/// be viewable in the "Antithesis SDK: Reachablity assertions" group.
#[macro_export]
macro_rules! unreachable {
    ($message:literal, $details:expr) => {
        $crate::assert_helper!(false, $message, $details, "reachability", "Unreachable", false)
    }
}

