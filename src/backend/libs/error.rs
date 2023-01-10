use std::fmt::Display;

#[derive(Debug)]
pub enum ChatErrorKind {
    UnMatchPasswd,
    EmptyUser,
    SingUp,
}
impl Display for ChatErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
