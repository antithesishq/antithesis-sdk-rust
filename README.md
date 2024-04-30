### **This SDK is still under development & not ready for adoption yet.**

# Antithesis Rust SDK

This library provides methods for Rust programs to configure the [Antithesis](https://antithesis.com) platform. It contains three kinds of functionality:
* Assertion macros that allow you to define test properties about your software or workload.
* Randomness functions for requesting both structured and unstructured randomness from the Antithesis platform.
* Lifecycle functions that inform the Antithesis environment that particular test phases or milestones have been reached.

For general usage guidance see the [Antithesis Rust SDK Documentation](https://antithesis.com/docs/using_antithesis/sdk/rust_sdk.html)

### Notes

When using llvm's link/loader from clang prior to version 16, it may be necessary to explicitly request 
that unused sections are NOT removed from the resulting binary.  

This can be done from cargo using environment variables:

     RUSTFLAGS="-C link-dead-code" 

Using cargo version 1.73 or later will ensure that this request is always issued to the llvm
link/loader, but older versions of cargo, like 1.69, will not ensure this.


To disable assertions (eg for a production build) use this feature flag for cargo builds:

    -F no-antithesis-sdk

When assertions are disabled, the `condition` and `detail` arguments specified
for assertions will be evaluated, but no assertions will be emitted, or otherwise processed.  

In this case (using feature flag `no-antithesis-sdk`), the assert macros will expand to 
nothing (other than the evaluation of `condition` and `details`)
