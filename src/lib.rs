pub mod backend;
pub mod gui;
pub mod log;

#[macro_export]
macro_rules! string {
    ($str:expr) => {
        String::from($str)
    };
}
