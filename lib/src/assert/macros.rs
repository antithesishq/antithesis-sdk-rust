#[cfg(feature = "full")]
#[doc(hidden)]
#[macro_export]
macro_rules! function {
    ($static:ident) => {
        // Define a do-nothing function `'_f()'` within the context of
        // the function invoking an assertion.  Then the ``type_name`` of
        // this do-nothing will be something like:
        //
        //     bincrate::binmod::do_stuff::_f
        //
        // After trimming off the last three chars ``::_f`` what remains is
        // the full path to the name of the function invoking the assertion
        //
        // The result will be stored as a lazily initialized statics in
        // `$static`, so that it can be available at
        // assertion catalog registration time.
        use $crate::once_cell::sync::Lazy;
        fn _f() {}
        static $static: $crate::once_cell::sync::Lazy<&'static str> =
            $crate::once_cell::sync::Lazy::new(|| {
                fn type_name_of<T>(_: T) -> &'static str {
                    ::std::any::type_name::<T>()
                }
                let name = type_name_of(_f);
                &name[..name.len() - 4]
            });
    };
}

/// Common handling used by all the assertion-related macros
#[cfg(feature = "full")]
#[doc(hidden)]
#[macro_export]
macro_rules! assert_helper {
    // The handling of this pattern-arm of assert_helper
    // is wrapped in a block {} to avoid name collisions
    (condition = $condition:expr, $message:literal, $(details = $details:expr)?, $assert_type:path, $display_type:literal, must_hit = $must_hit:literal) => {{
        // Force evaluation of expressions.
        let condition = $condition;
        let details = &$crate::serde_json::json!({});
        $(let details = $details;)?

        $crate::function!(FUN_NAME);

        use $crate::assert::AssertionCatalogInfo;
        #[$crate::linkme::distributed_slice($crate::assert::ANTITHESIS_CATALOG)]
        #[linkme(crate = $crate::linkme)] // Refer to our re-exported linkme.
        static ALWAYS_CATALOG_ITEM: AssertionCatalogInfo = AssertionCatalogInfo {
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
            id: $message,
        };

        let ptr_function = Lazy::force(&FUN_NAME);

        $crate::assert::assert_impl(
            $assert_type,                     /* assert_type */
            $display_type.to_owned(),         /* display_type */
            condition,                        /* condition */
            $message.to_owned(),              /* message */
            ::std::module_path!().to_owned(), /* class */
            String::from(*ptr_function),      /* function */
            ::std::file!().to_owned(),        /* file */
            ::std::line!(),                   /* line */
            ::std::column!(),                 /* column */
            true,                             /* hit */
            $must_hit,                        /* must-hit */
            $message.to_owned(),              /* id */
            details,                          /* details */
        )
    }}; // end pattern-arm block
}

#[cfg(not(feature = "full"))]
#[doc(hidden)]
#[macro_export]
macro_rules! assert_helper {
    (condition = $condition:expr, $message:literal, $(details = $details:expr)?, $assert_type:path, $display_type:literal, must_hit = $must_hit:literal) => {{
        // Force evaluation of expressions, ensuring that
        // any side effects of these expressions will always be
        // evaluated at runtime - even if the assertion itself
        // is supressed by the `no-antithesis-sdk` feature
        let condition = $condition;
        $(let details = $details;)?
    }};
}

/// Assert that ``condition`` is true every time this function is called, **and** that it is
/// called at least once. The corresponding test property will be viewable in the ``Antithesis SDK: Always`` group of your triage report.
///
/// # Example
///
/// ```
/// use serde_json::{json};
/// use antithesis_sdk::{assert_always, random};
///
/// const MAX_ALLOWED: u64 = 100;
/// let actual = random::get_random() % 100u64;
/// let details = json!({"max_allowed": MAX_ALLOWED, "actual": actual});
/// antithesis_sdk::assert_always!(actual < MAX_ALLOWED, "Value in range", &details);
/// ```
#[macro_export]
macro_rules! assert_always {
    ($condition:expr, $message:literal$(, $details:expr)?) => {
        $crate::assert_helper!(
            condition = $condition,
            $message,
            $(details = $details)?,
            $crate::assert::AssertType::Always,
            "Always",
            must_hit = true
        )
    };
    ($($rest:tt)*) => {
        ::std::compile_error!(
r#"Invalid syntax when calling macro `assert_always`.
Example usage:
    `assert_always!(condition_expr, "assertion message (static literal)", &details_json_value_expr)`
"#
        );
    };
}

/// Assert that ``condition`` is true every time this function is called. The corresponding test property will pass even if the assertion is never encountered.
/// This test property will be viewable in the ``Antithesis SDK: Always`` group of your triage report.
///
/// # Example
///
/// ```
/// use serde_json::{json};
/// use antithesis_sdk::{assert_always_or_unreachable, random};
///
/// const MAX_ALLOWED: u64 = 100;
/// let actual = random::get_random() % 100u64;
/// let details = json!({"max_allowed": MAX_ALLOWED, "actual": actual});
/// antithesis_sdk::assert_always_or_unreachable!(actual < MAX_ALLOWED, "Value in range", &details);
/// ```
#[macro_export]
macro_rules! assert_always_or_unreachable {
    ($condition:expr, $message:literal$(, $details:expr)?) => {
        $crate::assert_helper!(
            condition = $condition,
            $message,
            $(details = $details)?,
            $crate::assert::AssertType::Always,
            "AlwaysOrUnreachable",
            must_hit = false
        )
    };
    ($($rest:tt)*) => {
        ::std::compile_error!(
r#"Invalid syntax when calling macro `assert_always_or_unreachable`.
Example usage:
    `assert_always_or_unreachable!(condition_expr, "assertion message (static literal)", &details_json_value_expr)`
"#
        );
    };
}

/// Assert that ``condition`` is true at least one time that this function was called.
/// (If the assertion is never encountered, the test property will therefore fail.)
/// This test property will be viewable in the ``Antithesis SDK: Sometimes`` group.
///
/// # Example
///
/// ```
/// use serde_json::{json};
/// use antithesis_sdk::{assert_sometimes, random};
///
/// const MAX_ALLOWED: u64 = 100;
/// let actual = random::get_random() % 120u64;
/// let details = json!({"max_allowed": MAX_ALLOWED, "actual": actual});
/// antithesis_sdk::assert_sometimes!(actual > MAX_ALLOWED, "Value in range", &details);
/// ```
#[macro_export]
macro_rules! assert_sometimes {
    ($condition:expr, $message:literal$(, $details:expr)?) => {
        $crate::assert_helper!(
            condition = $condition,
            $message,
            $(details = $details)?,
            $crate::assert::AssertType::Sometimes,
            "Sometimes",
            must_hit = true
        )
    };
    ($($rest:tt)*) => {
        ::std::compile_error!(
r#"Invalid syntax when calling macro `assert_sometimes`.
Example usage:
    `assert_sometimes!(condition_expr, "assertion message (static literal)", &details_json_value_expr)`
"#
        );
    };
}

/// Assert that a line of code is reached at least once.
/// The corresponding test property will pass if this macro is ever called. (If it is never called the test property will therefore fail.)
/// This test property will be viewable in the ``Antithesis SDK: Reachablity assertions`` group.
///
/// # Example
///
/// ```
/// use serde_json::{json};
/// use antithesis_sdk::{assert_reachable, random};
///
/// const MAX_ALLOWED: u64 = 100;
/// let actual = random::get_random() % 120u64;
/// let details = json!({"max_allowed": MAX_ALLOWED, "actual": actual});
/// if (actual > MAX_ALLOWED) {
///     antithesis_sdk::assert_reachable!("Value in range", &details);
/// }
/// ```
#[macro_export]
macro_rules! assert_reachable {
    ($message:literal$(, $details:expr)?) => {
        $crate::assert_helper!(
            condition = true,
            $message,
            $(details = $details)?,
            $crate::assert::AssertType::Reachability,
            "Reachable",
            must_hit = true
        )
    };
    ($($rest:tt)*) => {
        ::std::compile_error!(
r#"Invalid syntax when calling macro `assert_reachable`.
Example usage:
    `assert_reachable!("assertion message (static literal)", &details_json_value_expr)`
"#
        );
    };
}

/// Assert that a line of code is never reached.
/// The corresponding test property will fail if this macro is ever called.
/// (If it is never called the test property will therefore pass.)
/// This test property will be viewable in the ``Antithesis SDK: Reachablity assertions`` group.
///
/// # Example
///
/// ```
/// use serde_json::{json};
/// use antithesis_sdk::{assert_unreachable, random};
///
/// const MAX_ALLOWED: u64 = 100;
/// let actual = random::get_random() % 120u64;
/// let details = json!({"max_allowed": MAX_ALLOWED, "actual": actual});
/// if (actual > 120u64) {
///     antithesis_sdk::assert_unreachable!("Value is above range", &details);
/// }
/// ```
#[macro_export]
macro_rules! assert_unreachable {
    ($message:literal$(, $details:expr)?) => {
        $crate::assert_helper!(
            condition = false,
            $message,
            $(details = $details)?,
            $crate::assert::AssertType::Reachability,
            "Unreachable",
            must_hit = false
        )
    };
    ($($rest:tt)*) => {
        ::std::compile_error!(
r#"Invalid syntax when calling macro `assert_unreachable`.
Example usage:
    `assert_unreachable!("assertion message (static literal)", &details_json_value_expr)`
"#
        );
    };
}

#[cfg(feature = "full")]
#[doc(hidden)]
#[macro_export]
macro_rules! guidance_helper {
    ($guidance_type:expr, $message:literal, $maximize:literal, $guidance_data:expr) => {
        $crate::function!(FUN_NAME);

        use $crate::assert::guidance::{GuidanceCatalogInfo, GuidanceType};
        #[$crate::linkme::distributed_slice($crate::assert::ANTITHESIS_GUIDANCE_CATALOG)]
        #[linkme(crate = $crate::linkme)] // Refer to our re-exported linkme.
        static GUIDANCE_CATALOG_ITEM: GuidanceCatalogInfo = GuidanceCatalogInfo {
            guidance_type: $guidance_type,
            message: $message,
            id: $message,
            class: ::std::module_path!(),
            function: &FUN_NAME,
            file: ::std::file!(),
            begin_line: ::std::line!(),
            begin_column: ::std::column!(),
            maximize: $maximize,
        };

        $crate::assert::guidance::guidance_impl(
            $guidance_type,
            $message.to_owned(),
            $message.to_owned(),
            ::std::module_path!().to_owned(),
            Lazy::force(&FUN_NAME).to_string(),
            ::std::file!().to_owned(),
            ::std::line!(),
            ::std::column!(),
            $maximize,
            $guidance_data,
            true,
        )
    };
}

#[cfg(feature = "full")]
#[doc(hidden)]
#[macro_export]
macro_rules! numeric_guidance_helper {
    ($assert:path, $op:tt, $maximize:literal, $left:expr, $right:expr, $message:literal$(, $details:expr)?) => {{
        let left = $left;
        let right = $right;
        let details = &$crate::serde_json::json!({});
        $(let details = $details;)?
        let mut details = details.clone();
        details["left"] = left.into();
        details["right"] = right.into();
        $assert!(left $op right, $message, &details);

        let guidance_data = $crate::serde_json::json!({
            "left": left,
            "right": right,
        });
        // TODO: Right now it seems to be impossible for this macro to use the returned
        // type of `diff` to instanciate the `T` in `Guard<T>`, which has to be
        // explicitly provided for the static variable `GUARD`.
        // Instead, we currently fix `T` to be `f64`, and ensure all implementations of `Diff` returns `f64`.
        // Here are some related language limitations:
        // - Although `typeof` is a reserved keyword in Rust, it is never implemented. See <https://stackoverflow.com/questions/64890774>.
        // - Rust does not, and explicitly would not (see https://doc.rust-lang.org/reference/items/static-items.html#statics--generics), support generic static variable.
        // - Type inference is not performed for static variable, i.e. `Guard<_>` is not allowed.
        // - Some form of existential type can help, but that's only available in nightly Rust under feature `type_alias_impl_trait`.
        //
        // Other approaches I can think of either requires dynamic type tagging that has
        // runtime overhead, or requires the user of the macro to explicitly provide the type,
        // which is really not ergonomic and deviate from the APIs from other SDKs.
        let diff = $crate::assert::guidance::Diff::diff(&left, right);
        type Guard<T> = $crate::assert::guidance::Guard<$maximize, T>;
        // TODO: Waiting for [type_alias_impl_trait](https://github.com/rust-lang/rust/issues/63063) to stabilize...
        // type Distance = impl Minimal;
        type Distance = f64;
        static GUARD: Guard<Distance> = Guard::init();
        if GUARD.should_emit(diff) {
            $crate::guidance_helper!($crate::assert::guidance::GuidanceType::Numeric, $message, $maximize, guidance_data);
        }
    }};
}

#[cfg(not(feature = "full"))]
#[doc(hidden)]
#[macro_export]
macro_rules! numeric_guidance_helper {
    ($assert:path, $op:tt, $maximize:literal, $left:expr, $right:expr, $message:literal$(, $details:expr)?) => {
        assert!($left $op $right, $message$(, $details)?);
    };
}

#[cfg(feature = "full")]
#[doc(hidden)]
#[macro_export]
macro_rules! boolean_guidance_helper {
    ($assert:path, $all:literal, {$($name:ident: $cond:expr),*}, $message:literal$(, $details:expr)?) => {{
        let details = &$crate::serde_json::json!({});
        $(let details = $details;)?
        let mut details = details.clone();
        let (cond, guidance_data) = {
            $(let $name = $cond;)*
            $(details[::std::stringify!($name)] = $name.into();)*
            (
                if $all { true $(&& $name)* } else { false $(|| $name)* },
                $crate::serde_json::json!({$(::std::stringify!($name): $name),*})
            )
        };
        $assert!(cond, $message, &details);
        $crate::guidance_helper!($crate::assert::guidance::GuidanceType::Boolean, $message, $all, guidance_data);
    }};
}

#[cfg(not(feature = "full"))]
#[doc(hidden)]
#[macro_export]
macro_rules! boolean_guidance_helper {
    ($assert:path, $all:literal, {$($name:ident: $cond:expr),*}, $message:literal$(, $details:expr)?) => {{
        let cond = {
            $(let $name = $cond;)*
            if $all { true $(&& $name)* } else { false $(|| $name)* }
        };
        $assert!(cond, $message$(, $details)?);
    }};
}

/// `assert_always_greater_than(x, y, ...)` is mostly equivalent to `assert_always!(x > y, ...)`, except Antithesis has more visibility to the value of `x` and `y`, and the assertion details would be merged with `{"left": x, "right": y}`.
#[macro_export]
macro_rules! assert_always_greater_than {
    ($left:expr, $right:expr, $message:literal$(, $details:expr)?) => {
        $crate::numeric_guidance_helper!($crate::assert_always, >, false, $left, $right, $message$(, $details)?)
    };
    ($($rest:tt)*) => {
        ::std::compile_error!(
r#"Invalid syntax when calling macro `assert_always_greater_than`.
Example usage:
    `assert_always_greater_than!(left_expr, right_expr, "assertion message (static literal)", &details_json_value_expr)`
"#
        );
    };
}

/// `assert_always_greater_than_or_equal_to(x, y, ...)` is mostly equivalent to `assert_always!(x >= y, ...)`, except Antithesis has more visibility to the value of `x` and `y`, and the assertion details would be merged with `{"left": x, "right": y}`.
#[macro_export]
macro_rules! assert_always_greater_than_or_equal_to {
    ($left:expr, $right:expr, $message:literal$(, $details:expr)?) => {
        $crate::numeric_guidance_helper!($crate::assert_always, >=, false, $left, $right, $message, $details)
    };
    ($($rest:tt)*) => {
        ::std::compile_error!(
r#"Invalid syntax when calling macro `assert_always_greater_than_or_equal_to`.
Example usage:
    `assert_always_greater_than_or_equal_to!(left_expr, right_expr, "assertion message (static literal)", &details_json_value_expr)`
"#
        );
    };
}

/// `assert_always_less_than(x, y, ...)` is mostly equivalent to `assert_always!(x < y, ...)`, except Antithesis has more visibility to the value of `x` and `y`, and the assertion details would be merged with `{"left": x, "right": y}`.
#[macro_export]
macro_rules! assert_always_less_than {
    ($left:expr, $right:expr, $message:literal$(, $details:expr)?) => {
        $crate::numeric_guidance_helper!($crate::assert_always, <, true, $left, $right, $message, $details)
    };
    ($($rest:tt)*) => {
        ::std::compile_error!(
r#"Invalid syntax when calling macro `assert_always_less_than`.
Example usage:
    `assert_always_less_than!(left_expr, right_expr, "assertion message (static literal)", &details_json_value_expr)`
"#
        );
    };
}

/// `assert_always_less_than_or_equal_to(x, y, ...)` is mostly equivalent to `assert_always!(x <= y, ...)`, except Antithesis has more visibility to the value of `x` and `y`, and the assertion details would be merged with `{"left": x, "right": y}`.
#[macro_export]
macro_rules! assert_always_less_than_or_equal_to {
    ($left:expr, $right:expr, $message:literal$(, $details:expr)?) => {
        $crate::numeric_guidance_helper!($crate::assert_always, <=, true, $left, $right, $message, $details)
    };
    ($($rest:tt)*) => {
        ::std::compile_error!(
r#"Invalid syntax when calling macro `assert_always_less_than_or_equal_to`.
Example usage:
    `assert_always_less_than_or_equal_to!(left_expr, right_expr, "assertion message (static literal)", &details_json_value_expr)`
"#
        );
    };
}

/// `assert_sometimes_greater_than(x, y, ...)` is mostly equivalent to `assert_sometimes!(x > y, ...)`, except Antithesis has more visibility to the value of `x` and `y`, and the assertion details would be merged with `{"left": x, "right": y}`.
#[macro_export]
macro_rules! assert_sometimes_greater_than {
    ($left:expr, $right:expr, $message:literal$(, $details:expr)?) => {
        $crate::numeric_guidance_helper!($crate::assert_sometimes, >, true, $left, $right, $message, $details)
    };
    ($($rest:tt)*) => {
        ::std::compile_error!(
r#"Invalid syntax when calling macro `assert_sometimes_greater_than`.
Example usage:
    `assert_sometimes_greater_than!(left_expr, right_expr, "assertion message (static literal)", &details_json_value_expr)`
"#
        );
    };
}

/// `assert_sometimes_greater_than_or_equal_to(x, y, ...)` is mostly equivalent to `assert_sometimes!(x >= y, ...)`, except Antithesis has more visibility to the value of `x` and `y`, and the assertion details would be merged with `{"left": x, "right": y}`.
#[macro_export]
macro_rules! assert_sometimes_greater_than_or_equal_to {
    ($left:expr, $right:expr, $message:literal$(, $details:expr)?) => {
        $crate::numeric_guidance_helper!($crate::assert_sometimes, >=, true, $left, $right, $message, $details)
    };
    ($($rest:tt)*) => {
        ::std::compile_error!(
r#"Invalid syntax when calling macro `assert_sometimes_greater_than_or_equal_to`.
Example usage:
    `assert_sometimes_greater_than_or_equal_to!(left_expr, right_expr, "assertion message (static literal)", &details_json_value_expr)`
"#
        );
    };
}

/// `assert_sometimes_less_than(x, y, ...)` is mostly equivalent to `assert_sometimes!(x < y, ...)`, except Antithesis has more visibility to the value of `x` and `y`, and the assertion details would be merged with `{"left": x, "right": y}`.
#[macro_export]
macro_rules! assert_sometimes_less_than {
    ($left:expr, $right:expr, $message:literal$(, $details:expr)?) => {
        $crate::numeric_guidance_helper!($crate::assert_sometimes, <, false, $left, $right, $message, $details)
    };
    ($($rest:tt)*) => {
        ::std::compile_error!(
r#"Invalid syntax when calling macro `assert_sometimes_less_than`.
Example usage:
    `assert_sometimes_less_than!(left_expr, right_expr, "assertion message (static literal)", &details_json_value_expr)`
"#
        );
    };
}

/// `assert_sometimes_less_than_or_equal_to(x, y, ...)` is mostly equivalent to `assert_sometimes!(x <= y, ...)`, except Antithesis has more visibility to the value of `x` and `y`, and the assertion details would be merged with `{"left": x, "right": y}`.
#[macro_export]
macro_rules! assert_sometimes_less_than_or_equal_to {
    ($left:expr, $right:expr, $message:literal$(, $details:expr)?) => {
        $crate::numeric_guidance_helper!($crate::assert_sometimes, <=, false, $left, $right, $message, $details)
    };
    ($($rest:tt)*) => {
        ::std::compile_error!(
r#"Invalid syntax when calling macro `assert_sometimes_less_than_or_equal_to`.
Example usage:
    `assert_sometimes_less_than_or_equal_to!(left_expr, right_expr, "assertion message (static literal)", &details_json_value_expr)`
"#
        );
    };
}

/// `assert_always_some({a: x, b: y, ...})` is similar to `assert_always(x || y || ...)`, except:
/// - Antithesis has more visibility to the individual propositions.
/// - There is no short-circuiting, so all of `x`, `y`, ... would be evaluated.
/// - The assertion details would be merged with `{"a": x, "b": y, ...}`.
#[macro_export]
macro_rules! assert_always_some {
    ({$($($name:ident: $cond:expr),+ $(,)?)?}, $message:literal$(, $details:expr)?) => {
        $crate::boolean_guidance_helper!($crate::assert_always, false, {$($($name: $cond),+)?}, $message$(, $details)?);
    };
    ($($rest:tt)*) => {
        ::std::compile_error!(
r#"Invalid syntax when calling macro `assert_always_some`.
Example usage:
    `assert_always_some!({field1: cond1, field2: cond2, ...}, "assertion message (static literal)", &details_json_value_expr)`
"#
        );
    };
}

/// `assert_sometimes_all({a: x, b: y, ...})` is similar to `assert_sometimes(x && y && ...)`, except:
/// - Antithesis has more visibility to the individual propositions.
/// - There is no short-circuiting, so all of `x`, `y`, ... would be evaluated.
/// - The assertion details would be merged with `{"a": x, "b": y, ...}`.
#[macro_export]
macro_rules! assert_sometimes_all {
    ({$($($name:ident: $cond:expr),+ $(,)?)?}, $message:literal$(, $details:expr)?) => {
        $crate::boolean_guidance_helper!($crate::assert_sometimes, true, {$($($name: $cond),+)?}, $message$(, $details)?);
    };
    ($($rest:tt)*) => {
        ::std::compile_error!(
r#"Invalid syntax when calling macro `assert_sometimes_all`.
Example usage:
    `assert_sometimes_all!({field1: cond1, field2: cond2, ...}, "assertion message (static literal)", &details_json_value_expr)`
"#
        );
    };
}
