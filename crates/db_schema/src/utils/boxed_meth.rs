/// Traits that are like those in [`diesel::query_dsl::methods`] but with `Output` automatically
/// set to `Self`, which is useful for boxed queries
use diesel::query_dsl::methods;

macro_rules! traits {
    ($(
        $name:ident
        <
        $($param:ident),*
        >,
    )*) => {$(
        pub trait $name<$($param),*>: methods::$name<$($param,)* Output = Self> {}
        impl<$($param,)* T: methods::$name<$($param,)* Output = Self>> $name<$($param),*> for T {}
    )*};
}

traits! {
    FilterDsl<Predicate>,
    LimitDsl<>,
    ThenOrderDsl<Expr>,
}
