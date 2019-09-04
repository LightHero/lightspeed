use c3p0::{C3p0Pool, Connection};
use lightspeed_core::error::LightSpeedError;

pub mod pg;

pub trait CmsRepositoryManager: Clone {
    type CONN: Connection;
    type C3P0: C3p0Pool<CONN = Self::CONN>;

    fn c3p0(&self) -> &Self::C3P0;
    fn start(&self) -> Result<(), LightSpeedError>;
}
