# Antithesis Rust SDK

This library provides methods for Rust programs to configure the [Antithesis](https://antithesis.com) platform. It contains three kinds of functionality:
* Assertion macros that allow you to define test properties about your software or workload.
* Randomness functions for requesting both structured and unstructured randomness from the Antithesis platform.
* Lifecycle functions that inform the Antithesis environment that particular test phases or milestones have been reached.

For general usage guidance see the [Antithesis Rust SDK Documentation](https://antithesis.com/docs/using_antithesis/sdk/rust/)

### Notes

To disable assertions disable `default-features` for this crate.

When assertions are disabled, the `condition` and `detail` arguments specified
for assertions will be evaluated, but no assertions will be emitted, or otherwise processed.

In this case, the assert macros will expand to
nothing (other than the evaluation of `condition` and `details`).
