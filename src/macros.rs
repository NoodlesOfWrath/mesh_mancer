use std::any::TypeId;

pub trait Input {
    type T;
    fn convert(&self) -> Self::T;
    fn needed_types() -> Vec<TypeId>;
}

// make a macro that generates the impl Input to turn Vec<dyn Any> into (A, B, C, D)
// make sure T1, T2, T3, T4 are Clone and 'static
macro_rules! impl_input {
    ($($t:ident),*) => {
        impl<$($t: Clone + 'static),*> Input for ($($t,)*) {
            type T = ($($t,)*);
            fn convert(&self) -> Self::T {
                self.clone()
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
