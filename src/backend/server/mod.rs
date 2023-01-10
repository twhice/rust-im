pub mod api;
use crate::{
    backend::libs::{error::ChatErrorKind, message::Message},
    log::*,
    string,
};

use super::*;
use std::{
    collections::HashMap,
    io::{self, ErrorKind, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs},
    sync::{
        mpsc::{self, Receiver},
        Arc, Mutex,
    },
    thread::{self},
    time::Duration,
};

#[derive(Debug)]
struct Server {
    // ID与密码
    user_lib: HashMap<usize, String>,
    // 用户总量
    users: usize,
    // 客户端
}
impl Server {
    pub fn start_server<T: ToSocketAddrs + Send + 'static>(ip: T) -> Arc<Mutex<Self>> {
        let s = Self {
            user_lib: HashMap::new(),
            users: 1,
        };
        let s = Arc::new(Mutex::new(s));
        let arc_self = s.clone();
        let mut clients: HashMap<usize, TcpStream> = HashMap::new();

        thread::spawn(move || {
            let listener = TcpListener::bind(ip).log_expect("未能创建TcpListenier");
            listener
                .set_nonblocking(true)
                .log_expect("未能设置为非阻塞模式");

            // 传递信息
            let (msg_sc, msg_rc) = mpsc::channel();

            // 客户端

            loop {
                // 接受连接
                if let Ok((mut socket, from_addr)) = listener.accept() {
                    // 传入子线程的变量
                    let msg_sd = msg_sc.clone();
                    // let thread_self = arc_self.clone();

                    // 子线程监听连接
                    thread::spawn(move || loop {
                        // 缓冲区
                        let mut buffer = [0; MEG_SIZE];

                        match socket.read_exact(&mut buffer) {
                            // 成功
                            Ok(_) => {
                                let message = buffer
                                    .into_iter()
                                    .take_while(|&x| x != 0)
                                    .collect::<Vec<_>>();
                                let strin =
                                    String::from_utf8(message).log_expect("未能从utf8生成字符串");

                                let msg = Message::try_from(
                                    json::parse(&strin).log_expect("损坏的字节流"),
                                )
                                .log_expect("未能实现从Json到Message的转化");

                                msg_sd
                                    .send((msg, socket.try_clone().unwrap(), from_addr))
                                    .log_expect("子线程未能发送Json");
                            }
                            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                            Err(_) => {
                                break;
                            }
                        };
                        thread::sleep(Duration::from_millis(10));
                    });
                }
                // DEBUG
                // println!("{:?}", self.user_lib);

                // 处理
                let mut s = arc_self.lock().unwrap();
                clients = s.try_solve_recv(&msg_rc, clients);
                drop(s);

                // 转发消息
                thread::sleep(Duration::from_millis(100));
            }
        });
        s
    }

    fn try_solve_recv(
        &mut self,
        msg_rc: &Receiver<(Message, TcpStream, SocketAddr)>,
        mut clients: HashMap<usize, TcpStream>,
    ) -> HashMap<usize, TcpStream> {
        if let Ok((message, stream, from_addr)) = msg_rc.try_recv() {
            // 两条调试信息
            format!("调用 clients: {clients:?} stream: {stream:?} from_addr: {from_addr}")
                .log_debug("TSR");
            message.log_with(format!("FR {}", from_addr));

            // 加入Clients
            clients.insert(0, stream);
            let mut new_clients = HashMap::new();

            // 迭代处理,同时清理无效连接
            let mut links_removed = 0;
            clients.into_iter().for_each(|(id, client)| {
                match client.peer_addr() {
                    Ok(client_addr) => {
                        // 调试信息
                        format!("匹配分支client: {client:?} id: {id}").log_debug("TSR");

                        // 判断迭代到的client是否为发送信息的client
                        if client_addr == from_addr {
                            if let Some((id, client)) = self.sender_active(&message, (id, client)) {
                                new_clients.insert(id, client);
                            }
                        // 不是发起者的情况
                        } else {
                            if let Some((id, client)) = self.getter_active(&message, (id, client)) {
                                new_clients.insert(id, client);
                            }
                        }
                    }

                    // 放弃无效连接
                    Err(_) => links_removed += 1,
                }
            });

            // 再两条调试信息
            format!("清理掉{}条无效连接", links_removed).log_debug("TSR");
            format!("结束 clients: {new_clients:?} ").log_debug("TSR");

            new_clients
        } else {
            clients
        }
    }
    fn sender_active<'a114514>(
        &'a114514 mut self,
        msg: &Message,
        mut hash: (usize, TcpStream),
    ) -> Option<(usize, TcpStream)> {
        // format!("调用函数SA msg: {msg:?} hash: ({},{:?})", hash.0, hash.1).log_debug("Server");
        match msg.head {
            // 登录
            libs::message::MessageHead::Login => {
                // 查看ID是否存在
                let from_id = msg.from.get_id();
                match self.user_lib.get(&from_id) {
                    Some(pwd) => {
                        // 查看密码是否匹配
                        if *pwd == msg.msg {
                            // 生成返回信息
                            send_message(
                                &mut hash.1,
                                Message {
                                    head: libs::message::MessageHead::Login,
                                    from: Default::default(),
                                    target: Default::default(),
                                    msg: string!("ok"),
                                },
                            )
                            .unwrap();

                            // 生成LOG
                            format!("用户[{}]登录 id: {}", msg.from.get_name(), from_id,).log();
                            Some(hash)
                        } else {
                            send_err(&mut hash.1, ChatErrorKind::UnMatchPasswd).unwrap();
                            None
                        }
                    }
                    // 不存在用户
                    None => {
                        send_err(&mut hash.1, ChatErrorKind::UnMatchPasswd).unwrap();
                        None
                    }
                }
            }

            // 注册
            libs::message::MessageHead::SighUp => {
                // 注册新用户
                let new_id = self.users;
                self.user_lib.insert(new_id, msg.msg.clone());

                // 反馈id
                send_message(
                    &mut hash.1,
                    Message {
                        head: libs::message::MessageHead::SighUp,
                        from: Default::default(),
                        target: Default::default(),
                        msg: new_id.to_string(),
                    },
                )
                .unwrap();

                // 处理id 和返回值
                self.users += 1;
                let hash = (new_id, hash.1);

                // 生成LOG
                format!("用户[{}]注册 id: {}", msg.from.get_name(), new_id,).log();
                Some(hash)
            }

            _ => Some(hash),
        }
    }
    fn getter_active<'a, 'b, 'c>(
        &'a mut self,
        msg: &'b Message,
        mut hash: (usize, TcpStream),
    ) -> Option<(usize, TcpStream)> {
        // 只有转发信息
        if let libs::message::MessageHead::Message = msg.head {
            let from_id = msg.target.get_id();
            let taget_id = hash.0.clone();

            // 调试信息
            format!("fid: {} tid: {}", from_id, taget_id).log_debug("RE");

            // 仅在目标id为0(全体)或者与传入的id相匹配时发送信息
            if from_id == 0 || from_id == taget_id {
                if let Ok(..) = send_message(&mut hash.1, msg.clone()) {
                    Some(hash)
                } else {
                    None
                }
            } else {
                Some(hash)
            }
        } else {
            Some(hash)
        }
    }
}

fn send_message(stream: &mut TcpStream, msg: Message) -> io::Result<()> {
    // DEBUG
    msg.log_with(format!("TO {}", msg.target.to_string()));

    let mut bytes = json::stringify(msg.to_map()).into_bytes();
    bytes.resize(MEG_SIZE, 0);
    stream.write_all(&bytes)
}
fn send_err(stream: &mut TcpStream, err: ChatErrorKind) -> io::Result<()> {
    send_message(
        stream,
        Message::new(
            libs::message::MessageHead::Display,
            Default::default(),
            Default::default(),
            err.to_string(),
        ),
    )
}
