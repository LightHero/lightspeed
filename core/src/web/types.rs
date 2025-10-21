pub use traits::*;

mod traits {
    pub trait MaybeWeb: Send + Sync {}
    impl<T: Send + Sync> MaybeWeb for T where T: ?Sized {}
}
