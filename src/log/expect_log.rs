use super::*;
pub trait ExpectLog<T> {
    fn log_expect<S: AsStr>(self, text: S) -> T;
    fn log_expect_with<S1: AsStr, S2: AsStr>(self, text: S1, with: S2) -> T;
}
impl<T, E> ExpectLog<T> for Result<T, E> {
    fn log_expect<S: AsStr>(self, text: S) -> T {
        match self {
            Ok(ret) => ret,
            Err(_) => {
                log(format!("[ERROR]: {}", text.to_string()));
                exit(-1);
            }
        }
    }

    fn log_expect_with<S1: AsStr, S2: AsStr>(self, text: S1, with: S2) -> T {
        match self {
            Ok(ret) => ret,
            Err(_) => {
                log(format!(
                    "[ERROR][{}]: {}",
                    with.to_string(),
                    text.to_string()
                ));
                exit(-1);
            }
        }
    }
}
