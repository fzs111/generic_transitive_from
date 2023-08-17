#![forbid(unsafe_code)]
#![cfg_attr(not(test), no_std)]

//! [![github]](https://github.com/fzs111/generic_transitive_from)
//! [![MIT / Apache 2.0 licensed]](#License)
//! [![unsafe forbidden]](https://github.com/rust-secure-code/safety-dance/)
//! 
//! [github]: https://img.shields.io/badge/github-fzs111/generic__transitive__from-red.svg
//! [MIT / Apache 2.0 licensed]: https://img.shields.io/badge/license-MIT_or_Apache--2.0-blue.svg
//! [unsafe forbidden]: https://img.shields.io/badge/unsafe-forbidden-success.svg
//! 
//! This repository is a fork of [steffahn/transitive_from](https://github.com/steffahn/transitive_from), extended to support lifetimes/generics.

/**
Helper macro to create transitive [`From`] implementations.

This macro can work on tree-shaped hierarchies of
implementations `impl From<Child> for Parent` along the tree edges.
It will produce new `From` implementations along paths towards
the root by chaining [`From::from()`] calls along the edges.
There must not be any pre-existing `impl From`s for anything but
the immediate edges.

For further details, study the example below.

The first item of the macro invocation must be a list of generics and 
lifetimes enclosed by square brackets, which will be applied to all impls
the macro generates. You can also specify bounds on the types.

That is followed by the definition of the tree structure. Inside of each 
`{` `}` block, a trailing comma is optional.

Note that the same generic arguments will be provided to *all* implementations.
It is *not* an error to specify superfluous lifetime parameters, 
so it is possible to only use a subset of the in some impls. The same is not true 
for generic types, each type must be used in each implementation.

# Examples

```
# use core::fmt::Debug;

enum A<'a, T>{
    B(B<T>),
    E(E<'a, T>)
}
struct B<T>(T);

struct C<T>(T);
struct D<T>(T);

struct E<'a, T>(F<'a, T>);
struct F<'a, T>(&'a T);

//It is required to manually provide From implementations for each step in the hierarchy: 

impl<'a, T> From<C<T>> for B<T> {
    fn from(C(t): C<T>) -> Self {
        B(t)
    }
}
impl<'a, T> From<D<T>> for B<T> {
    fn from(D(t): D<T>) -> Self {
        B(t)
    }
}
impl<'a, T> From<T> for D<T> {
    fn from(t: T) -> Self {
        D(t)
    }
}

impl<'a, T> From<B<T>> for A<'a, T> {
    fn from(b: B<T>) -> Self {
        A::B(b)
    }
}

impl<'a, T> From<E<'a, T>> for A<'a, T> {
    fn from(e: E<'a, T>) -> Self {
        A::E(e)
    }
}


impl<'a, T> From<F<'a, T>> for E<'a, T> {
    fn from(f: F<'a, T>) -> Self {
        E(f)
    }
}

impl<'a, T> From<&'a T> for F<'a, T> {
    fn from(r: &'a T) -> Self {
        F(r)
    }
}

// Now, to produce all the remaining (transitive) implementations
// and complete the hierarchy, call the macro like this:

generic_transitive_from::impl_from! {
    ['a, T: Debug]
    A<'a, T> {
        B<T> {
            C<T>,
            D<T> {
                T
            }
        },
        E<'a, T>{
            F<'a, T> {
                &'a T
            }
        }
    }
}

/*
This macro call will create the following impls:

impl<'a, T: Debug> From<C<T>>     for A<'a, T> { ... }
impl<'a, T: Debug> From<D<T>>     for A<'a, T> { ... }
impl<'a, T: Debug> From<T>        for A<'a, T> { ... }
impl<'a, T: Debug> From<T>        for B<'a, T> { ... }
impl<'a, T: Debug> From<F<'a, T>> for A<'a, T> { ... }
impl<'a, T: Debug> From<&'a T>    for A<'a, T> { ... }
impl<'a, T: Debug> From<&'a T>    for E<'a, T> { ... }
*/

// Finally, a few demonstration/test cases:
```
*/
#[macro_export]
macro_rules! impl_from {

    (
        $generics:tt
        $($root:ty $({
            $($child:ty $({
                $($grandchildren_parsed_recursively:tt)*
            })?),* $(,)?
        })?),* $(,)?
    ) => {
        $($(
            $crate::impl_from!{
                $generics
                $($child $({
                    $($grandchildren_parsed_recursively)*
                })?),*
            }
            $($(
                $crate::__impl_from_internals_recursive!{
                    $generics[$root][$child][
                        $($grandchildren_parsed_recursively)*
                    ]
                }
            )?)*
        )?)*

    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __impl_from_internals_recursive {
    ($generics:tt[$root:ty][$child:ty][
        $($grandchild:ty $({
            $($further:tt)*
        })?),* $(,)?
    ]) => {
        $(
            $($crate::__impl_from_internals_recursive!{
                $generics[$root][$child][
                    $($further)*
                ]
            })?
            $crate::__impl_from_internals_make_impl!{
                $generics[$root][$child][$grandchild]
            }
        )*
    };
}


#[doc(hidden)]
#[macro_export]
macro_rules! __impl_from_internals_make_impl {
    ([$($generic:tt)*][$root:ty][$child:ty][$grandchild:ty]) => {
        impl<$($generic)*> ::core::convert::From<$grandchild> for $root {
            fn from(g: $grandchild) -> Self {
                <$root>::from(<$child>::from(g))
            }
        }
    }
}

#[cfg(test)]
mod test {
    #![allow(unused)]
    pub enum GlobalError<'a> {
        Shape(ShapeError),
        Color(ColorError<'a>),
    }
    impl From<ShapeError> for GlobalError<'_> {
        fn from(e: ShapeError) -> Self {
            Self::Shape(e)
        }
    }
    impl<'a> From<ColorError<'a>> for GlobalError<'a> {
        fn from(e: ColorError<'a>) -> Self {
            Self::Color(e)
        }
    }

    pub enum ShapeError {
        Circle(CircleError),
        Rectangle(RectangleError),
    }
    impl From<CircleError> for ShapeError {
        fn from(e: CircleError) -> Self {
            Self::Circle(e)
        }
    }
    impl From<RectangleError> for ShapeError {
        fn from(e: RectangleError) -> Self {
            Self::Rectangle(e)
        }
    }

    pub struct CircleError {
        msg: &'static str,
        radius: f64,
    }

    pub enum RectangleError {
        Square(SquareError),
        ArbitraryRectangleError { msg: &'static str, a: f64, b: f64 },
    }
    impl From<SquareError> for RectangleError {
        fn from(e: SquareError) -> Self {
            Self::Square(e)
        }
    }

    pub struct SquareError {
        msg: &'static str,
        a: f64,
    }

    pub enum ColorError<'a> {
        Red(RedError<'a>),
        Blue(BlueError<'a>),
    }
    impl<'a> From<RedError<'a>> for ColorError<'a> {
        fn from(e: RedError<'a>) -> Self {
            Self::Red(e)
        }
    }
    impl<'a> From<BlueError<'a>> for ColorError<'a> {
        fn from(e: BlueError<'a>) -> Self {
            Self::Blue(e)
        }
    }

    pub struct RedError<'a> {
        msg: &'a str,
    }

    pub struct BlueError<'a> {
        msg: &'a str,
    }

    crate::impl_from! {
        ['a]
        GlobalError<'a> {
            ShapeError {
                CircleError,
                RectangleError { SquareError },
            },
            ColorError<'a> { RedError<'a>, BlueError<'a> }
        }
    }

    fn foo() -> Result<(), SquareError> {
        Err(SquareError {
            msg: "hello world",
            a: 42.0,
        })
    }

    fn bar() -> Result<(), GlobalError<'static>> {
        foo()?;
        Ok(())
    }

    #[test]
    fn conversion_test() {
        bar().err().unwrap();
    }

    #[test]
    fn baz() {
        let s = std::string::String::from("error");
        let e = RedError{
            msg: &s
        };
        let a: GlobalError = e.into();
    }
}
