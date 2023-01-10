use crate::log::Log;

use super::super::libs::*;
use super::Client as BasicClient;
use std::sync::{Arc, Mutex};

const CLIENT: &str = "Client";

pub struct Client {
    client: Arc<Mutex<BasicClient>>,
}
impl Client {
    pub fn new<T: AsAddr>(addr: T) -> Self {
        Self {
            client: BasicClient::connect(addr),
        }
    }

    pub fn sighup<T1: AsStr, T2: AsStr>(&mut self, name: T1, pwd: T2) {
        let mut client = self.client.lock().unwrap();
        client.sighup(name, pwd).log_warn("Client");
    }

    pub fn login<T1: AsStr, T2: AsStr>(&mut self, name: T1, uid: usize, pwd: T2) {
        let mut client = self.client.lock().unwrap();
        client.login(name, uid, pwd).log_warn(CLIENT);
    }

    pub fn send_message<T: AsStr>(&mut self, text: T, target: usize) {
        let mut client = self.client.lock().unwrap();
        client.send_text(text, target).log_warn(CLIENT)
    }

    pub fn user_id(&self) -> usize {
        self.client.lock().unwrap().profile.get_id()
    }

    pub fn is_login(&self) -> bool {
        self.client.lock().unwrap().login
    }
}

impl Clone for Client {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
        }
    }
}
