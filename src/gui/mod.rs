use crate::backend::client::api::Client;

mod login;
pub fn run() {
    let client = Client::new("127.0.0.1:3888");
    login::login(client.clone());
}
