use std::any::{Any, TypeId};

pub trait InputOrOutput {
    type T;
    fn convert(items: Vec<&dyn Any>) -> Self::T;
    fn convert_output(item: &Self::T) -> Vec<Box<dyn Any>>;
    fn needed_types() -> Vec<TypeId>;
}

macro_rules! tuple_to_vec {
    ($($elem:expr),*) => {
        {
            let mut vec: Vec<Box<dyn Any>> = Vec::new();
            $(
                vec.push(Box::new($elem) as Box<dyn Any>);
            )*
            vec
        }
    };
}

// make a macro that generates the impl Input to turn Vec<dyn Any> into (A, B, C, D)
// make sure T1, T2, T3, T4 are Clone and 'static
macro_rules! impl_input {
    ($($t:ident),*) => {
        #[allow(non_snake_case)]
        impl<$($t: Clone + 'static),*> InputOrOutput for ($($t,)*) {
            type T = ($($t,)*);
            fn convert(items: Vec<&dyn Any>) -> Self::T {
                let mut items = items;
                let new_item = Box::new(()) as Box<dyn Any>;
                if TypeId::of::<Self::T>() == TypeId::of::<((),)>() {
                    items.push(&*new_item as &dyn Any);
                }

                let mut items = items.into_iter();

                ($($t::clone(items.next().expect("Not enough elements!").downcast_ref::<$t>().expect("Wrong type!")),)*)
            }
            fn convert_output(($($t,)*): &Self::T) -> Vec<Box<dyn Any>> {
                tuple_to_vec!($($t.clone()),*)
            }
            fn needed_types() -> Vec<TypeId> {
                // if T is (), then we don't need any types
                // this is not a good solution, but rust doesn't allow negative trait bounds
                // which is needed for a specific implementation of () only
                if TypeId::of::<Self::T>() == TypeId::of::<((),)>() {
                    return vec![];
                }

                vec![$(TypeId::of::<$t>()),*]
            }
        }
    };
}

impl_input!(T1);
impl_input!(T1, T2);
impl_input!(T1, T2, T3);
impl_input!(T1, T2, T3, T4);
impl_input!(T1, T2, T3, T4, T5);
impl_input!(T1, T2, T3, T4, T5, T6);
impl_input!(T1, T2, T3, T4, T5, T6, T7);
impl_input!(T1, T2, T3, T4, T5, T6, T7, T8);
impl_input!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_input!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_input!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_input!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
impl_input!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
impl_input!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
impl_input!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);
impl_input!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16);
impl_input!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17);
impl_input!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19);
impl_input!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20
);
