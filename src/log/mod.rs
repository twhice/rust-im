pub mod expect_log;
pub use expect_log::*;

use std::{
    fmt::Display,
    io::{stdout, Write},
    process::exit,
};

use crate::backend::libs::{message::Message, AsStr};

static mut OUT_PUT: Option<Box<dyn Write>> = None;

pub fn init() {
    unsafe { OUT_PUT = Some(Box::new(stdout())) }
}

fn log<T: AsStr>(text: T) {
    unsafe {
        if let Some(output) = &mut OUT_PUT {
            output
                .write(format!("{}\n", text.to_string()).as_str().as_bytes())
                .unwrap();
        }
    }
}
pub trait Log {
    fn log_message(&self) -> (String, bool);

    fn log(&self) {
        let (msg, iflog) = self.log_message();
        if iflog {
            log(format!("[LOG]: {}", msg));
        }
    }

    fn log_with<T: AsStr>(&self, text: T) {
        let (msg, iflog) = self.log_message();
        if iflog {
            log(format!("[LOG][{}]: {}", text.to_string(), msg));
        }
    }
    fn log_error<T: AsStr>(&self, text: T) {
        let (msg, iflog) = self.log_message();
        if iflog {
            log(format!("[ERROR][{}] :{}", text.to_string(), msg))
        }
    }
    fn log_warn<T: AsStr>(&self, text: T) {
        let (msg, iflog) = self.log_message();
        if iflog {
            log(format!("[WARN][{}] :{}", text.to_string(), msg))
        }
    }
    fn log_debug<T: AsStr>(&self, text: T) {
        let (msg, iflog) = self.log_message();
        if iflog {
            log(format!("[DEBUD][{}] :{}", text.to_string(), msg))
        }
    }
    fn log_exiterror<T: AsStr>(&self, text: T) {
        println!("程序终止,请查看日志");
        self.log_error(text);
        exit(-1)
    }
}

impl<T, E> Log for Result<T, E>
where
    T: Sized,
    E: Display,
{
    fn log_message(&self) -> (String, bool) {
        match self {
            Ok(_) => (String::new(), false),
            Err(e) => (e.to_string(), true),
        }
    }
}

impl Log for Message {
    fn log_message(&self) -> (String, bool) {
        (format!("{self:?}"), true)
    }
}

impl Log for String {
    fn log_message(&self) -> (String, bool) {
        (self.clone(), self.len() != 0)
    }
}
