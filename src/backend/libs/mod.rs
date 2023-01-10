pub mod error;
pub mod event;
pub mod message;
pub mod usergroup;
use std::{fmt::Display, net::ToSocketAddrs};
pub trait AsAddr
where
    Self: ToSocketAddrs + Clone + Display + Send + 'static,
{
}
pub trait AsStr
where
    Self: ToString + Clone,
{
}
impl<T> AsAddr for T where T: ToSocketAddrs + Clone + Display + Send + 'static {}
impl<T> AsStr for T where T: ToString + Clone {}
