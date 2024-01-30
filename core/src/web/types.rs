#[cfg(feature = "poem_openapi")]
pub mod types {

    pub use poem_openapi::types::*;
    pub trait MaybeWeb: Send + Sync + Type + ParseFromJSON + ToJSON {}
    impl<T: Send + Sync + Type + ParseFromJSON + ToJSON> MaybeWeb for T {}
}

#[cfg(not(feature = "poem_openapi"))]
pub mod types {
    pub trait MaybeWeb: Send + Sync {}
    impl<T: Send + Sync> MaybeWeb for T where T: ?Sized {}
}
