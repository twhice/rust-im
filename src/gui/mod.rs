use std::{thread, time::Duration};

use crate::backend::client::api::Client;

mod login;
pub mod menu;
pub fn run() {
    let mut client = Client::new("127.0.0.1:3888");
    login::login(client.clone());
    println!("Logined");
    client.update_userlist();
    menu::menu(client);
}
