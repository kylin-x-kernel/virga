//! 传输协议层

#[cfg(feature = "use-xtransport")]
mod xtransport_impl;
#[cfg(feature = "use-xtransport")]
pub use xtransport_impl::XTransportHandler;

#[cfg(feature = "use-yamux")]
mod yamux_impl;
#[cfg(feature = "use-yamux")]
pub use yamux_impl::YamuxTransportHandler;
#[cfg(feature = "use-yamux")]
pub use yamux_impl::get_runtime;
