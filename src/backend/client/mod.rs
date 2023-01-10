pub mod api;
use super::libs::{message::Message, AsStr};
use super::*;
use crate::log::*;
use crate::{backend::libs::usergroup::Target, string};

use std::{
    fmt::Display,
    io::{ErrorKind, Read, Write},
    net::{TcpStream, ToSocketAddrs},
    sync::{
        mpsc::{self, Sender, TryRecvError},
        Arc, Mutex,
    },
    thread::{self},
    time::Duration,
};

struct Client {
    profile: Target,
    sender: Sender<Message>,
    pub login: bool,
}
impl Client {
    pub fn connect<T: ToSocketAddrs + Clone + Display + Send + 'static>(
        server_ip: T,
    ) -> Arc<Mutex<Client>> {
        let mut stream = connect_server(server_ip.clone());
        let (sender, recevier) = mpsc::channel::<Message>();
        let s = Arc::new(Mutex::new(Self {
            profile: Default::default(),
            sender,
            login: false,
        }));
        let sc = s.clone();
        thread::spawn(move || loop {
            let mut buffer = vec![0; MEG_SIZE];
            let read_result = stream.read_exact(&mut buffer);
            match read_result {
                Ok(_) => {
                    let msg = Message::try_from(
                        json::parse(
                            String::from_utf8(buffer.into_iter().take_while(|&x| x != 0).collect())
                                .unwrap()
                                .as_str(),
                        )
                        .log_expect("未能转化成json"),
                    )
                    .log_expect("未能转化成Message");
                    // DEBUG
                    msg.log_with("Server");

                    // 处理message
                    let mut s = sc.lock().log_expect("解锁失败");
                    s.resolve_message(msg);
                }
                Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                Err(_) => {
                    string!("连接错误,正在尝试恢复").log_warn("Client");
                    stream = connect_server(server_ip.clone());
                }
            }
            match recevier.try_recv() {
                Ok(msg) => {
                    let mut buffer = json::stringify(msg.to_map()).to_string().into_bytes();
                    if buffer.len() <= 1024 {
                        buffer.resize(MEG_SIZE, 0);
                        stream.write_all(&buffer).log_expect("写入失败");
                    } else {
                        string!("消息过长,发送失败").log_warn("Client")
                    }
                }
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => break,
            }
            thread::sleep(Duration::from_millis(10));
        });
        s
    }
    pub fn login<T1: AsStr, T2: AsStr>(
        &mut self,
        name: T1,
        uid: usize,
        pwd: T2,
    ) -> Result<(), String> {
        if !self.login {
            // 尝试登录
            self.profile = Target::User {
                name: name.to_string(),
                id: uid,
            };
            // 发送登录请求
            match self.send_message(Message {
                head: crate::backend::libs::message::MessageHead::Login,
                from: self.profile.clone(),
                target: Default::default(),
                msg: pwd.to_string(),
            }) {
                Ok(_) => Ok(()),
                Err(_) => Err(string!("未能发送登录请求")),
            }
        } else {
            Err(string!("不允许重复登录!"))
        }
    }
    pub fn sighup<T1: AsStr, T2: AsStr>(&mut self, name: T1, pwd: T2) -> Result<(), String> {
        if !self.login {
            self.profile = Target::User {
                name: name.to_string(),
                id: 0,
            };
            match self.send_message(Message {
                head: crate::backend::libs::message::MessageHead::SighUp,
                from: self.profile.clone(),
                target: Default::default(),
                msg: pwd.to_string(),
            }) {
                Ok(_) => Ok(()),
                Err(_) => Err(string!("未能发送注册请求")),
            }
        } else {
            Err(string!("不能重复登录!"))
        }
    }
    pub fn send_text<T: AsStr>(&mut self, text: T, target_id: usize) -> Result<(), String> {
        if self.login {
            let target = Target::User {
                name: string!(""),
                id: target_id,
            };
            match self.send_message(Message {
                head: crate::backend::libs::message::MessageHead::Message,
                from: self.profile.clone(),
                target: target,
                msg: text.to_string(),
            }) {
                Ok(_) => Ok(()),
                Err(e) => Err(e.to_string()),
            }
        } else {
            Err(string!("尚未登录"))
        }
    }
    fn send_message(&mut self, message: Message) -> Result<(), mpsc::SendError<Message>> {
        self.sender.send(message)
    }
    fn resolve_message(&mut self, message: Message) {
        match message.head {
            libs::message::MessageHead::SighUp => {
                self.login = true;
                self.profile
                    .set_id(message.msg.parse().log_expect("服务器传回了无效的ID"))
            }
            libs::message::MessageHead::Login => self.login = true,
            libs::message::MessageHead::Message => {
                let from = message.from.get_name();
                format!("<{}> {}", from, message.msg).log_with("Msg")
            }
            libs::message::MessageHead::Display => message.msg.log_warn("Server"),
        }
    }
}
fn connect_server<T: ToSocketAddrs + Display + Send + Clone>(server_ip: T) -> TcpStream {
    let client: TcpStream;
    loop {
        match TcpStream::connect(server_ip.clone()) {
            Ok(stream) => {
                client = stream;
                break;
            }
            Err(reason) => {
                format!(
                    "未能连接至服务器[{}] 原因:[{}] 一秒后重试",
                    server_ip, reason
                )
                .log_warn("Client");
                thread::sleep(Duration::from_secs(1));
            }
        }
    }
    client
        .set_nonblocking(true)
        .log_expect("未能设置为非阻塞模式");
    client
}
