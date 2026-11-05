//! Общие технические компоненты Benitoo; возможности включаются явно через features.
#[cfg(feature = "contracts")]
pub mod contracts;
#[cfg(feature = "postgres-wakeup")]
pub mod postgres_wakeup;
#[cfg(feature = "realtime")]
mod realtime;
#[cfg(feature = "test-support")]
pub mod test_support;
