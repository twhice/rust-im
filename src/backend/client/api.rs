use crate::{backend::libs::usergroup::Target, log::Log, string};

use super::super::libs::*;
use super::Client as BasicClient;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

const CLIENT: &str = "Client";

#[derive(Clone)]
pub struct Client {
    client: Arc<Mutex<BasicClient>>,
    user_list: Arc<Mutex<HashMap<usize, Target>>>,
    // wait:Arc<Mutex<HashMap<usize,Target>>>
}
impl Client {
    pub fn new<T: AsAddr>(addr: T) -> Self {
        let (s, r) = BasicClient::connect(addr);
        let s = Self {
            client: s,
            user_list: Arc::new(Mutex::new(HashMap::new())),
        };
        let s_clone = s.clone();
        thread::spawn(move || loop {
            if let Ok((i, t)) = r.recv() {
                s_clone.user_list.lock().unwrap().insert(i, t);
            }
        });
        s
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

    pub fn update_userlist(&mut self) {
        if self.is_login() {
            let mut lock = self.client.lock().unwrap();
            // let profile = lock.profile.clone();
            lock.send_message(message::Message {
                head: message::MessageHead::UserList,
                from: Target::default(),
                target: Target::default(),
                msg: string!(""),
            })
            .unwrap();
        } else {
            string!("请先登录").log_warn("Client");
        }
    }

    pub fn get_userlist(&self) -> HashMap<usize, Target> {
        loop {
            let lock = self.client.lock().unwrap();
            if !lock.wait {
                return self.user_list.lock().unwrap().clone();
            }
            drop(lock);
            thread::sleep(Duration::from_millis(10));
        }
    }
}
