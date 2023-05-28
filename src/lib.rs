//! A macro for defining `#[cfg]` if-else statements containing potentially cross-crate variables.
//!
//! This macro is similar to [`cfg_if!`][cfg_if]—so similar, in fact, that we're going to plagiarize its
//! documentation for a bit:
//!
//! > The macro provided by this crate, [`cond!`], is similar to the `if/elif` C preprocessor macro
//! > by allowing definition of a cascade of `#[cfg]` cases, emitting the implementation which
//! > matches first.
//! >
//! > This allows you to conveniently provide a long list `#[cfg]`'d blocks of code without having
//! > to rewrite each clause multiple times.
//! >
//! > ## Example
//! >
//! > ```
//! > cfgenius::cond! {
//! >     if cfg(unix) {
//! >         fn foo() { /* unix specific functionality */ }
//! >     } else if cfg(target_pointer_width = "32") {
//! >         fn foo() { /* non-unix, 32-bit functionality */ }
//! >     } else {
//! >         fn foo() { /* fallback implementation */ }
//! >     }
//! > }
//! > ```
//!
//! ---
//!
//! What's new, however, is the ability to [`define!`] custom conditional-compilation variables and
//! use those variables in your [`cond!`] predicates:
//!
//! ```
//! // In `crate_1`...
//! # mod crate_1 {
//! cfgenius::define! {
//!     pub(super) is_32_bit_or_more = cfg(any(
//!         target_pointer_width = "32",
//!         target_pointer_width = "64",
//!     ));
//!
//!     pub is_recommended = all(
//!         macro(is_32_bit_or_more),
//!         macro(is_supported),
//!         cfg(target_has_atomic),
//!     );
//! }
//!
//! cfgenius::cond! {
//!     if all(cfg(windows), not(macro(is_32_bit_or_more))) {
//!         cfgenius::define!(pub is_supported = true());
//!
//!         // windows-specific non-32-bit functionality
//!     } else if all(cfg(windows), macro(is_32_bit_or_more)) {
//!         cfgenius::define!(pub is_supported = true());
//!
//!         // windows-specific non-32-bit functionality
//!     } else {
//!         cfgenius::define!(pub is_supported = false());
//!     }
//! }
//!
//! pub const IS_SUPPORTED: bool = cfgenius::cond_expr!(macro(is_supported));
//! # }
//!
//! // In `crate_2`...
//! cfgenius::cond! {
//!     if any(
//!         macro(crate_1::is_recommended),
//!         all(cfg(feature = "force_crate_1_backend"), macro(crate_1::is_supported))
//!     ) {
//!         // (`crate_1` implementation)
//!     } else {
//!         // (fallback implementation)
//!     }
//! }
//! ```
//!
//! This is not possible in regular `#[cfg]` attributes:
//!
//! ```compile_fail
//! macro_rules! truthy {
//!     () => { all() };
//! }
//!
//! #[cfg(truthy!())]
//! //          ^ Syntax Error: expected one of `(`, `,`, `::`, or `=`, found `!`
//! mod this_is_compiled {}
//! ```
//!
//! ## Predicates
//!
//! In every place where we could expect a conditionally compiled predicate, the following predicates
//! are supported:
//!
//! - `true()`: is always truthy
//!
//! - `false()`: is always falsy
//!
//! - `cfg(<cfg input>)`: resolves to the result of a regular [cfg attribute][cfg_attr] with the
//!   same input.
//!
//! - `not(<predicate>)`: negates the resolution of the provided `cfgenius` predicate.
//!
//! - `all(<predicate 1>, <predicate 2>, ...)`: resolves to truthy if none of the provided `cfgenius`
//!   predicates fail. `all()` with no provided predicates resolves to true.
//!
//! - `any(<predicate 1>, <predicate 2>, ...)`: resolves to truthy if at least of the provided `cfgenius`
//!   predicates succeed. `any()` with no provided predicates resolves to false.
//!
//! - `macro(<path to macro>)`: uses the macro to determine the truthiness of the predicate.
//!
//! - `macro(<path to macro> => <macro arguments>)`: uses the macro with the provided arguments to
//!   determine the truthiness of the predicate.
//!
//! ## Custom Variables
//!
//! Most variables can be succinctly defined using [`define!`]. However, because variables are just
//! macros which are expanded to get their result, you can define your own variables by following the
//! protocol.
//!
//! The predicate `macro(<path to macro>)` is evaluated by expanding:
//!
//! ```no_compile
//! path::to::macro! {
//!     yes { /* truthy tokens */ }
//!     no { /* falsy tokens */ }
//! }
//! ```
//!
//! ...and the predicate `macro(<path to macro> => <macro arguments>)` is evaluated by expanding:
//!
//! ```no_compile
//! path::to::macro! {
//!     args { /* macro arguments */ }
//!     yes { /* truthy tokens */ }
//!     no { /* falsy tokens */ }
//! }
//! ```
//!
//! If the variable should be truthy, the macro should expand to `/* truthy tokens */` and nothing
//! more. If the variable should be falsy, the macro should expand to `/* falsy tokens */` and
//! nothing more.
//!
//! These macros should be effectless and pure with respect to their environment. You should not
//! rely on this macro being evaluated once for every time it appears in a predicate, even though
//! this is the current behavior.
//!
//! [cfg_if]: https://docs.rs/cfg-if/1.0.0/cfg_if/index.html
//! [cfg_attr]: https://doc.rust-lang.org/reference/conditional-compilation.html

// #![no_std]

/// A conditionally-compiled statement or item. See crate documentation for more information on the
/// predicate syntax.
///
/// ## Syntax
///
/// ```plain_text
/// cond! {
///     if <if predicate> {
///         // arbitrary tokens
///     } else if <else-if predicate> {  // There can be zero or more of these.
///         // arbitrary tokens
///     } else {                         // This is optional.
///         // arbitrary tokens
///     }
/// }
/// ```
///
/// See the [predicates](index.html#predicates) section of the crate documentation for more
/// information about the predicate grammar.
#[cfg(doc)]
#[macro_export]
macro_rules! cond {
    (
        $(if $pred:ident ($($pred_args:tt)*) {
            $($yes:tt)*
        }) else + $(else {
            $($no:tt)*
        })?
    ) => {};
}

#[cfg(not(doc))]
#[macro_export]
macro_rules! cond {
    // We begin by implementing `cond!` for one level of `if ... { ... } else { ... }`.

	// true
	(
		@__internal_single_munch
		if true() {
			$($yes:tt)*
		} else {
			$($no:tt)*
		}
	) => {
		$($yes)*
	};

	// false
	(
		@__internal_single_munch
		if false() {
			$($yes:tt)*
		} else {
			$($no:tt)*
		}
	) => {
		$($no)*
	};

    // cfg
    (@__internal_id $($id:tt)*) => { $($id)* };
    (
        @__internal_single_munch
        if cfg($($args:tt)*) {
            $($yes:tt)*
        } else {
            $($no:tt)*
        }
    ) => {
        #[cfg($($args)*)] $crate::cond! { @__internal_id $($yes)* }
        #[cfg(not($($args)*))] $crate::cond! { @__internal_id $($no)* }
    };

    // not
    (
        @__internal_single_munch
        if not($pred:ident ($($pred_args:tt)*)) {
            $($yes:tt)*
        } else {
            $($no:tt)*
        }
    ) => {
        $crate::cond! {
            if $pred($($pred_args)*) {
                $($no)*
            } else {
                $($yes)*
            }
        }
    };

    // all
    (
        @__internal_single_munch
        if all(
            $first_pred:ident($($first_args:tt)*)
            $(, $($rest:tt)*)?
        ) {
            $($yes:tt)*
        } else {
            $($no:tt)*
        }
    ) => {
        $crate::cond! {
            @__internal_single_munch
            if $first_pred($($first_args)*) {
                $crate::cond! {
                    @__internal_single_munch
                    if all($($($rest)*)?) {
                        $($yes)*
                    } else {
                        $($no)*
                    }
                }
            } else {
                $($no)*
            }
        }
    };
    (
        @__internal_single_munch
        if all() {
            $($yes:tt)*
        } else {
            $($no:tt)*
        }
    ) => { $($yes)* };

    // any
    (
        @__internal_single_munch
        if any($first_pred:ident($($first_args:tt)*) $(, $($rest:tt)*)?) {
            $($yes:tt)*
        } else {
            $($no:tt)*
        }
    ) => {
        $crate::cond! {
            @__internal_single_munch
            if $first_pred($($first_args)*) {
                $($yes)*
            } else {
                $crate::cond! {
                    @__internal_single_munch
                    if any($($($rest)*)?) {
                        $($yes)*
                    } else {
                        $($no)*
                    }
                }
            }
        }
    };
    (
        @__internal_single_munch
        if any() {
            $($yes:tt)*
        } else {
            $($no:tt)*
        }
    ) => { $($no)* };

    // macro
    (
        @__internal_single_munch
        if macro($path:path $( => $($args:tt)*)?) {
            $($yes:tt)*
        } else {
            $($no:tt)*
        }
    ) => {
        $path!($(args { $($args)* })? yes { $($yes)* } no { $($no)* } );
    };

    // Now, we can implement support for an arbitrary chaining of these.
    // TODO: Validate `cond!` grammar in its entirety, even if the faulty branches are never taken.

    // Because falsy paths are never expanded into the final output, bad macro calls to `cond!` are
    // ignored in the falsy paths, which is a bit janky. We avoid this scenario by validating the
    // syntax before munching through it.
    (
        $(if $pred:ident ($($pred_args:tt)*) {
            $($yes:tt)*
        }) else + $(else {
            $($no:tt)*
        })?
    ) => {
        $crate::cond! {
            @__internal_chained_munch
            $(
                if $pred($($pred_args)*) {
                    $($yes)*
                }
            ) else + $(else {
                $($no)*
            })?
        }
    };

    (
        @__internal_chained_munch
        if $pred:ident ($($pred_args:tt)*) {
            $($yes:tt)*
        } $(else $($rest:tt)*)?
    ) => {
        $crate::cond! {
            @__internal_single_munch
            if $pred($($pred_args)*) {
                $($yes)*
            } else {
                $($crate::cond! {
                    @__internal_chained_munch
                    $($rest)*
                })?
            }
        }
    };
    (
        @__internal_chained_munch
        { $($rest:tt)* }
    ) => {
        $($rest)*
    };
}

/// A conditionally-compiled expression. See crate documentation for more information on the predicate
/// syntax.
///
/// ## Syntax
///
/// ```plain_text
/// cond_expr! {
///     if <if predicate> {
///         // arbitrary tokens forming a `BlockExpression`.
///     } else if <else-if predicate> {  // There can be zero or more of these.
///         // arbitrary tokens forming a `BlockExpression`.
///     } else {                         // This is optional.
///         // arbitrary tokens forming a `BlockExpression`.
///     }
/// }
/// ```
///
/// or, if you just want to evaluate a boolean literal for the predicate, the following alias can
/// be used instead:
///
/// ```plain_text
/// cond_expr!(<predicate>)
/// ```
///
/// See the [predicates](index.html#predicates) section of the crate documentation for more
/// information about the predicate grammar.
#[macro_export]
macro_rules! cond_expr {
    (
        $(if $pred:ident ($($pred_args:tt)*) {
            $($yes:tt)*
        }) else + $(else {
            $($no:tt)*
        })?
    ) => {'__cond_expr_out: {
        $crate::cond! {
            $(if $pred ($($pred_args)*) {
                break '__cond_expr_out ({ $($yes)* });
            }) else + $(else {
                break '__cond_expr_out ({ $($no)* });
            })?
        }
    }};
    ($pred:ident ($($pred_args:tt)*)) => {
        $crate::cond_expr! {
            if $pred($($pred_args)*) {
                true
            } else {
                false
            }
        }
    }
}

/// A conditional-compilation variable that always resolves to `true`.
///
/// Note that you can equivalently use the `true()` predicate inside `cfgenius` predicates.
#[macro_export]
macro_rules! truthy {
    (yes { $($yes:tt)* } no { $($no:tt)* }) => { $($yes)* };
}

/// A conditional-compilation variable that always resolves to `false`.
///
/// Note that you can equivalently use the `false()` predicate inside `cfgenius` predicates.
#[macro_export]
macro_rules! falsy {
    (yes { $($yes:tt)* } no { $($no:tt)* }) => { $($no)* };
}

/// Defines a zero or more conditional-compilation variables which evaluate to the provided `cfgenius`
/// predicate.
///
/// These merely desugar to `use` items of [`truthy!`] and [`falsy!`].
///
/// ## Syntax
///
/// ```plain_text
/// define! {
///     <visibility> <name> = <predicate>
/// }
/// ```
///
/// ...or, if you want to define more than one predicate:
///
/// ```plain_text
/// define! {
///     <visibility 1> <name 1> = <predicate 1>;
///     <visibility 2> <name 2> = <predicate 2>;
///     // ...
///     <visibility N> <name N> = <predicate N> // <-- the semicolon is optional.
/// }
/// ```
///
/// See the [predicates](index.html#predicates) section of the crate documentation for more
/// information about the predicate grammar.
///
/// See also the [custom variable](index.html#custom-variables) section of the crate documentation
/// for information how to define more complex variables, potentially with arguments.
#[macro_export]
macro_rules! define {
    (
		$( $vis:vis $name:ident = $pred:ident ($($pred_args:tt)*) );* $(;)?
	) => {
		$(
			$crate::cond! {
				if $pred($($pred_args)*) {
					$vis use $crate::truthy as $name;
				} else {
					$vis use $crate::falsy as $name;
				}
			}
		)*
	};
}
