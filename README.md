# CFGenius

<!-- cargo-rdme start -->

A macro for defining `#[cfg]` if-else statements containing potentially cross-crate variables.

This macro is similar to [`cfg_if!`][cfg_if]—so similar, in fact, that we're going to plagiarize its
documentation for a bit:

> The macro provided by this crate, [`cond!`](https://docs.rs/cfgenius/latest/cfgenius/macro.cond.html), is similar to the `if/elif` C
> preprocessor macro by allowing definition of a cascade of `#[cfg]` cases, emitting the
> implementation which matches first.
>
> This allows you to conveniently provide a long list `#[cfg]`'d blocks of code without having
> to rewrite each clause multiple times.
>
> ### Example
>
> ```rust
> cfgenius::cond! {
>     if cfg(unix) {
>         fn foo() { /* unix specific functionality */ }
>     } else if cfg(target_pointer_width = "32") {
>         fn foo() { /* non-unix, 32-bit functionality */ }
>     } else {
>         fn foo() { /* fallback implementation */ }
>     }
> }
> ```

---

What's new, however, is the ability to [`define!`](https://docs.rs/cfgenius/latest/cfgenius/macro.define.html) custom conditional-compilation
variables and use those variables in your [`cond!`](https://docs.rs/cfgenius/latest/cfgenius/macro.cond.html) predicates:

```rust
// In `crate_1`...
cfgenius::define! {
    pub(super) is_32_bit_or_more = cfg(any(
        target_pointer_width = "32",
        target_pointer_width = "64",
    ));

    pub is_recommended = all(
        macro(is_32_bit_or_more),
        macro(is_supported),
        cfg(target_has_atomic),
    );
}

cfgenius::cond! {
    if all(cfg(windows), not(macro(is_32_bit_or_more))) {
        cfgenius::define!(pub is_supported = true());

        // windows-specific non-32-bit functionality
    } else if all(cfg(windows), macro(is_32_bit_or_more)) {
        cfgenius::define!(pub is_supported = true());

        // windows-specific non-32-bit functionality
    } else {
        cfgenius::define!(pub is_supported = false());
    }
}

pub const IS_SUPPORTED: bool = cfgenius::cond_expr!(macro(is_supported));

// In `crate_2`...
cfgenius::cond! {
    if any(
        macro(crate_1::is_recommended),
        all(cfg(feature = "force_crate_1_backend"), macro(crate_1::is_supported))
    ) {
        // (`crate_1` implementation)
    } else {
        // (fallback implementation)
    }
}
```

This is not possible in regular `#[cfg]` attributes:

```rust
macro_rules! truthy {
    () => { all() };
}

#[cfg(truthy!())]
//          ^ Syntax Error: expected one of `(`, `,`, `::`, or `=`, found `!`
mod this_is_compiled {}
```

### Predicates

In every place where we could expect a conditionally compiled predicate, the following predicates
are supported:

- `true()`: is always truthy

- `false()`: is always falsy

- `cfg(<cfg input>)`: resolves to the result of a regular [cfg attribute][cfg_attr] with the
  same input.

- `not(<predicate>)`: negates the resolution of the provided `cfgenius` predicate.

- `all(<predicate 1>, <predicate 2>, ...)`: resolves to truthy if none of the provided `cfgenius`
  predicates fail. `all()` with no provided predicates resolves to true.

- `any(<predicate 1>, <predicate 2>, ...)`: resolves to truthy if at least of the provided `cfgenius`
  predicates succeed. `any()` with no provided predicates resolves to false.

- `macro(<path to macro>)`: uses the macro to determine the truthiness of the predicate.

- `macro(<path to macro> => <macro arguments>)`: uses the macro with the provided arguments to
  determine the truthiness of the predicate.

### Custom Variables

Most variables can be succinctly defined using [`define!`](https://docs.rs/cfgenius/latest/cfgenius/macro.define.html). However, because
variables are just macros which are expanded to get their result, you can define your own
variables by following this protocol.

The predicate `macro(<path to macro>)` is evaluated by expanding:

```no_compile
path::to::macro! {
    yes { /* truthy tokens */ }
    no { /* falsy tokens */ }
}
```

...and the predicate `macro(<path to macro> => <macro arguments>)` is evaluated by expanding:

```no_compile
path::to::macro! {
    args { /* macro arguments */ }
    yes { /* truthy tokens */ }
    no { /* falsy tokens */ }
}
```

If the variable should be truthy, the macro should expand to `/* truthy tokens */` and nothing
more. If the variable should be falsy, the macro should expand to `/* falsy tokens */` and
nothing more.

These macros should be effectless and pure with respect to their environment. You should not
rely on this macro being evaluated once for every time it appears in a predicate, even though
this is the current behavior.

[cfg_if]: https://docs.rs/cfg-if/1.0.0/cfg_if/index.html
[cfg_attr]: https://doc.rust-lang.org/reference/conditional-compilation.html

<!-- cargo-rdme end -->
