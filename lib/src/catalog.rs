use std::io;

#[cfg(feature = "full")]
use crate::assert::{self, guidance};
#[cfg(feature = "full")]
use once_cell::sync::Lazy;
#[cfg(feature = "full")]
use serde_json::Value;

/// Which assertion macro a declaration came from.
///
/// This is the typed form of the `display_type` field in the serialized
/// event; [`display_type()`](Self::display_type) gives that string back. It
/// is distinct from [`AssertType`](crate::assert::AssertType), the
/// three-way wire-level enum, which cannot tell [`assert_always!`] from
/// [`assert_always_or_unreachable!`] or [`assert_reachable!`] from
/// [`assert_unreachable!`] without also consulting `must_hit`.
///
/// The numeric and boolean macros (`assert_always_greater_than!`,
/// `assert_sometimes_all!`, and so on) are ordinary `Always` or `Sometimes`
/// assertions with guidance attached, and report as such.
///
/// [`assert_always!`]: crate::assert_always
/// [`assert_always_or_unreachable!`]: crate::assert_always_or_unreachable
/// [`assert_reachable!`]: crate::assert_reachable
/// [`assert_unreachable!`]: crate::assert_unreachable
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum AssertionKind {
    /// [`assert_always!`](crate::assert_always): the condition must hold on
    /// every evaluation, and the assertion must be reached.
    Always,
    /// [`assert_always_or_unreachable!`](crate::assert_always_or_unreachable):
    /// the condition must hold on every evaluation, but the assertion need
    /// never be reached.
    AlwaysOrUnreachable,
    /// [`assert_sometimes!`](crate::assert_sometimes): the condition must
    /// hold on at least one evaluation.
    Sometimes,
    /// [`assert_reachable!`](crate::assert_reachable): the assertion must be
    /// reached at least once.
    Reachable,
    /// [`assert_unreachable!`](crate::assert_unreachable): the assertion must
    /// never be reached.
    Unreachable,
}

impl AssertionKind {
    /// The `display_type` string that serialized events carry for this kind,
    /// which is also how the triage report labels it. Use it to match catalog
    /// entries against events read back from a run.
    pub const fn display_type(self) -> &'static str {
        match self {
            AssertionKind::Always => "Always",
            AssertionKind::AlwaysOrUnreachable => "AlwaysOrUnreachable",
            AssertionKind::Sometimes => "Sometimes",
            AssertionKind::Reachable => "Reachable",
            AssertionKind::Unreachable => "Unreachable",
        }
    }
}

/// Where an assertion was declared, as captured by the macro at compile time.
///
/// `file` is whatever `file!()` produced when the crate was compiled, so its
/// form (relative or absolute, remapped or not) depends on how the build was
/// invoked. `class` is the enclosing module path and `function` the path of
/// the enclosing function.
///
/// Locations order by file, line, and column, then module and function.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub struct SourceLocation {
    pub file: &'static str,
    pub begin_line: u32,
    pub begin_column: u32,
    pub class: &'static str,
    pub function: &'static str,
}

/// One assertion declaration compiled into this binary.
///
/// There is one per assertion macro call site linked into the program,
/// whether or not that code ever runs: the catalog is a static property of
/// the binary, not of any particular execution. The numeric and boolean
/// macros each contribute one assertion. The guidance they additionally
/// register is not an assertion: [`antithesis_init()`](crate::antithesis_init)
/// registers it with the platform, but this module does not list it.
///
/// Catalog entries registered at runtime through
/// [`assert_raw`](crate::assert::assert_raw) are not part of the compiled-in
/// catalog and do not appear.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct Assertion {
    /// The key under which Antithesis aggregates this assertion's evaluations
    /// into one test property. Currently equal to `message`; treat it as
    /// opaque.
    pub id: &'static str,
    /// The name of the test property, as shown in the triage report.
    pub message: &'static str,
    pub kind: AssertionKind,
    pub location: SourceLocation,
}

/// Every assertion declaration compiled into this binary.
///
/// These are the assertions that [`antithesis_init()`](crate::antithesis_init)
/// registers, read without emitting anything and without init having run.
/// The order is the linker's, the same order init registers them in: fixed
/// for a given binary, but free to change between builds. Sort by
/// [`Assertion::location`] if you need an order that survives rebuilds.
///
/// Without the `full` feature the catalog is always empty.
///
/// ```
/// use antithesis_sdk::{assert_sometimes, catalog};
///
/// // Never called, yet its assertion is still in the catalog.
/// #[allow(dead_code)]
/// fn never_called(x: u64) {
///     assert_sometimes!(x > 3, "x exceeds three");
/// }
///
/// for assertion in catalog::assertions() {
///     println!(
///         "{:?} {:?} at {}:{}",
///         assertion.kind, assertion.message,
///         assertion.location.file, assertion.location.begin_line
///     );
/// }
/// ```
pub fn assertions() -> impl Iterator<Item = Assertion> {
    #[cfg(feature = "full")]
    {
        assert::ANTITHESIS_CATALOG.iter().map(assertion_of)
    }
    #[cfg(not(feature = "full"))]
    {
        std::iter::empty()
    }
}

/// Writes the assertions of [`assertions()`] to `out` as JSON lines.
///
/// Each line is the catalog-registration event that
/// [`antithesis_init()`](crate::antithesis_init) emits for that assertion,
/// produced by the same serialization path and in the same order. Init
/// additionally writes the `antithesis_sdk` version header first and the
/// guidance registrations of the numeric and boolean macros last; neither
/// describes an assertion, so they are left out here.
///
/// Nothing is sent to the Antithesis environment or to the local output file,
/// and init need not have run. Without the `full` feature nothing is written.
///
/// For a checked-in golden file, sort the lines first: their order is the
/// linker's and can change between builds. Two further caveats: `file` is
/// whatever `file!()` produced at compile time, which depends on how the
/// build was invoked (for example `--remap-path-prefix`), and `function`
/// comes from `std::any::type_name`, whose exact form is not guaranteed
/// stable across compiler versions.
pub fn write<W: io::Write>(out: W) -> io::Result<()> {
    #[cfg(feature = "full")]
    {
        let mut out = out;
        for event in assertion_events() {
            serde_json::to_writer(&mut out, &event)?;
            out.write_all(b"\n")?;
        }
        Ok(())
    }
    #[cfg(not(feature = "full"))]
    {
        let _ = out;
        Ok(())
    }
}

/// Everything `antithesis_init()` registers, in order: the assertion events
/// [`write()`] emits, then the guidance the numeric and boolean macros also
/// declare. Init and `write()` share the assertion sequence, so they cannot
/// disagree about it.
#[cfg(feature = "full")]
pub(crate) fn registrations() -> impl Iterator<Item = Value> {
    let guidance = assert::ANTITHESIS_GUIDANCE_CATALOG.iter().map(guidance::catalog_event);
    assertion_events().chain(guidance)
}

#[cfg(feature = "full")]
fn assertion_events() -> impl Iterator<Item = Value> {
    assert::ANTITHESIS_CATALOG.iter().map(assert::catalog_event)
}

// The explicit deref is needed on older compilers, which otherwise infer
// `T = str` from the field type and reject the call.
#[cfg(feature = "full")]
#[allow(clippy::explicit_auto_deref)]
fn assertion_of(info: &assert::AssertionCatalogInfo) -> Assertion {
    Assertion {
        id: info.id,
        message: info.message,
        kind: kind_of(info.assert_type, info.must_hit),
        location: SourceLocation {
            file: info.file,
            begin_line: info.begin_line,
            begin_column: info.begin_column,
            class: info.class,
            function: *Lazy::force(info.function),
        },
    }
}

#[cfg(feature = "full")]
fn kind_of(assert_type: assert::AssertType, must_hit: bool) -> AssertionKind {
    match (assert_type, must_hit) {
        (assert::AssertType::Always, true) => AssertionKind::Always,
        (assert::AssertType::Always, false) => AssertionKind::AlwaysOrUnreachable,
        (assert::AssertType::Sometimes, _) => AssertionKind::Sometimes,
        (assert::AssertType::Reachability, true) => AssertionKind::Reachable,
        (assert::AssertType::Reachability, false) => AssertionKind::Unreachable,
    }
}
