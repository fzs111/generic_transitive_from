#![forbid(unsafe_code)]
#![cfg_attr(not(test), no_std)]

//! [![crates.io]](https://crates.io/crates/transitive_from)
//! [![github]](https://github.com/steffahn/transitive_from)
//! [![MIT / Apache 2.0 licensed]](https://github.com/steffahn/transitive_from#License)
//! [![unsafe forbidden]](https://github.com/rust-secure-code/safety-dance/)
//!
//! Helper macros for creating hierarchies of transitive [`From`] implementations.
//!
//! Currently, this crate only consists of the [`transitive_from::hierarchy`](hierarchy) macro.  
//! Please use the link to go to its page for further documentation.
//!
//! [github]: https://img.shields.io/badge/github-steffahn/transitive__from-yellowgreen.svg
//! [crates.io]: https://img.shields.io/crates/v/transitive_from.svg
//! [MIT / Apache 2.0 licensed]: https://img.shields.io/crates/l/replace_with.svg
//! [docs.rs]: https://docs.rs/transitive_from/badge.svg
//! [unsafe forbidden]: https://img.shields.io/badge/unsafe-forbidden-success.svg

/**
Helper macro to create transitive [`From`] implementations.

This macro can work on tree-shaped hierarchies of
implementations `impl From<Child> for Parent` along the tree edges.
It will produce new `From` implementations along paths towards
the root by chaining [`From::from()`] calls along the edges.
There must not be any pre-existing `impl From`s for anything but
the immediate edges.

For further details, study the example below.

The syntax supports arbitrary type expressions where the example just
uses simple names like `A`, `B`, `C`, etc; the macro does
however not produce any generic implementations. Inside of each `{` `}`
block, a trailing comma is optional.

# Examples
```
// Here’s a drawing of an example hierarchy.
//
//            ┌─ E
//      ┌─ B ─┤     ┌─ J
//      │     └─ F ─┤
//      │           └─ K
//   A ─┼─ C ─── G
//      │
//      │     ┌─ H
//      └─ D ─┤
//            └─ I ─── L
//
// For example, all these types could be error types and we
// would like to fully support upcasting with the `?` operator
// from anywhere to anywhere in this hierarchy.
struct A;
struct B;
struct C;
struct D;
struct E;
struct F;
struct G;
struct H;
struct I;
struct J;
struct K;
struct L;

// We need to provide implementation for all the tree edges
// (all the immediate "child -> parent" steps) manually,
// or by some other means. In this example we use a small macro
// to save some boilerplate.
macro_rules! impl_From {
    (<$B:ident> for $A:ident) => {
        impl From<$B> for $A {
            fn from(_: $B) -> $A {
                $A
            }
        }
    }
}
impl_From!(<B> for A);
impl_From!(<C> for A);
impl_From!(<D> for A);
impl_From!(<E> for B);
impl_From!(<F> for B);
impl_From!(<G> for C);
impl_From!(<H> for D);
impl_From!(<I> for D);
impl_From!(<J> for F);
impl_From!(<K> for F);
impl_From!(<L> for I);

// Now, to produce all the remaining (transitive) implementations
// and complete the hierarchy, call the macro like this:
transitive_from::hierarchy! {
    []
    A {
        B {
            E,
            F { J, K },
        },
        C { G },
        D {
            H,
            I { L },
        },
    }
}
// Note how the syntax resembles the tree drawn at the top of this example.

// Finally, a few demonstration/test cases:
A::from(K);
A::from(E);
B::from(K);
D::from(L);
A::from(L);
```
*/
#[macro_export]
macro_rules! hierarchy {

    (
        $generics:tt
        $($root:ty $({
            $($child:ty $({
                $($grandchildren_parsed_recursively:tt)*
            })?),* $(,)?
        })?),* $(,)?
    ) => {
        $($(
            $crate::hierarchy!{
                $generics
                $($child $({
                    $($grandchildren_parsed_recursively)*
                })?),*
            }
            $($(
                $crate::__hierarchy_internals!{
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
macro_rules! __hierarchy_internals {
    ($generics:tt[$root:ty][$child:ty][
        $($grandchild:ty $({
            $($further:tt)*
        })?),* $(,)?
    ]) => {
        $(
            $($crate::__hierarchy_internals!{
                $generics[$root][$child][
                    $($further)*
                ]
            })?
            $crate::__hierarchy_internals_impl!{
                $generics[$root][$child][$grandchild]
            }
        )*
    };
}


#[doc(hidden)]
#[macro_export]
macro_rules! __hierarchy_internals_impl {
    ([$($generic:tt),*][$root:ty][$child:ty][$grandchild:ty]) => {
        impl<$($generic),*> ::core::convert::From<$grandchild> for $root {
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

    crate::hierarchy! {
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
