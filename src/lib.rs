#![no_std]

#[macro_export]
macro_rules! cond {
    // We begin by implementing `cond!` for one level of `if ... { ... } else { ... }`.

	// true
	(@__internal_id $($id:tt)*) => { $($id)* };
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
	(@__internal_id $($id:tt)*) => { $($id)* };
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
    ) => { $no };

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

#[macro_export]
macro_rules! truthy {
    (yes { $($yes:tt)* } no { $($no:tt)* }) => { $($yes)* };
}

#[macro_export]
macro_rules! falsy {
    (yes { $($yes:tt)* } no { $($no:tt)* }) => { $($no)* };
}

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
