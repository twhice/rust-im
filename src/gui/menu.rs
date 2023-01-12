use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::Display,
    rc::Rc,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use fltk::{
    app::{event_dy, event_key, App, MouseWheel, Sender},
    button::*,
    enums::{Align, Color, Event, Key},
    frame::Frame,
    group::Flex,
    input::Input,
    prelude::{GroupExt, InputExt, WidgetBase, WidgetExt, WindowExt},
    window::Window,
};

use crate::{
    backend::{client::api::Client, libs::usergroup::Target},
    log::Log,
    string,
};

enum Birdge {
    HashMap(HashMap<usize, Target>),
    Id(usize),
    Text(String),
}

impl Display for Birdge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Birdge::HashMap(a) => write!(f, "{a:?}"),
            Birdge::Id(a) => write!(f, "{a}"),
            Birdge::Text(a) => write!(f, "{a}"),
        }
    }
}

pub fn menu(mut client: Client) {
    let app = App::default().load_system_fonts();
    let win_sizex = Rc::new(RefCell::new(800));
    let win_sizey = Rc::new(RefCell::new(450));

    let mut menu = Window::default()
        .with_size(*win_sizex.borrow(), *win_sizey.borrow())
        .center_screen();

    // 奇奇怪怪的问题
    let (punlic_sender, public_recvier) = fltk::app::channel();

    // 所有可选的聊天对象
    let mut targets_hm: HashMap<usize, Target> = HashMap::new();
    // 现在选取的聊天对象
    let taeget_id_now = Rc::new(RefCell::new(0));
    // 更新在线列表
    let tsw_update_senser = punlic_sender.clone();
    // 获取一次在线列表
    client.update_userlist();
    loop {
        targets_hm = client.get_userlist();
        if targets_hm.len() != 0 {
            break;
        }
        thread::sleep(Duration::from_micros(10));
    }
    tsw_update_senser.send(Birdge::HashMap(targets_hm.clone()));

    // 总消息表
    let messages_table: Arc<Mutex<HashMap<usize, Vec<(bool, String)>>>> = Default::default();
    // 消息接收器 线程接受信息
    let msg_recvier = client.get_msg_recvier();
    let messages_clone = messages_table.clone();
    let msw_update_sender = punlic_sender.clone();
    thread::spawn(move || loop {
        if let Ok(msg) = msg_recvier.lock().unwrap().recv() {
            let mut lock = messages_clone.lock().unwrap();
            let from_id = msg.from.get_id();
            match &mut lock.get_mut(&from_id) {
                Some(vec) => vec.push((false, msg.msg.clone())),
                None => {
                    lock.insert(from_id, vec![(false, msg.msg.clone())]);
                }
            };
            msw_update_sender.send(Birdge::Id(from_id))
        }
    });
    let msw_update_sender = punlic_sender.clone();
    let text_sender = punlic_sender.clone();

    // 两(三)个子窗口
    let (tsw, tsf) = targets_wf();
    let (msw, msf) = messages_wf();
    let (tiw, _tib) = input_wi(text_sender);

    // tsw.show();
    // msw.show();
    // tiw.show();

    menu.add(&tsw);
    menu.add(&msw);
    menu.add(&tiw);

    menu.show();
    menu.end();

    // 1 app监听
    while app.wait() {
        if let Some(result) = public_recvier.recv() {
            match result {
                Birdge::HashMap(targets_hm) => {
                    let tsf = tsf.clone();
                    // 清空并隐藏等待更新
                    string!("Updated").log_debug("TUR");
                    (*tsf).borrow_mut().clear();
                    (*tsf).borrow_mut().hide();
                    for (uid, target) in targets_hm {
                        format!("UID: {uid}").log_debug("TUR");
                        let mut button = taget_button(target.get_name());
                        let tin = taeget_id_now.clone();
                        (*tsf).borrow_mut().add(&button);
                        (*tsf).borrow_mut().set_size(&button, 40);
                        let msus = msw_update_sender.clone();
                        button.set_callback(move |_| {
                            format!("target_user reset to {uid} now").log_debug("GUI");
                            *(*tin).borrow_mut() = uid;
                            msus.send(Birdge::Id(uid))
                        })
                    }
                    (*tsf).borrow_mut().show();
                }
                Birdge::Id(nid) => {
                    string!("Updated").log_debug("MUR");
                    let msf = msf.clone();
                    let mst = messages_table.clone();
                    if nid == *taeget_id_now.borrow() {
                        (*msf).borrow_mut().clear();
                        (*msf).borrow_mut().hide();

                        if let Some(vec) = mst.lock().unwrap().get(&nid) {
                            for (is_send_byself, text) in vec {
                                let mut row = Flex::default().row();
                                let mut text_frame = if *is_send_byself {
                                    let f = Frame::default();
                                    row.set_size(&f, 640);
                                    Frame::default().with_align(Align::Left)
                                } else {
                                    let f = Frame::default().with_align(Align::Right);
                                    let p = Frame::default();
                                    row.set_size(&p, 640);
                                    f
                                };
                                text_frame.set_label(&text);
                                row.end();
                                (*msf).borrow_mut().add(&row);
                            }
                        }

                        (*msf).borrow_mut().show()
                    }
                }
                Birdge::Text(text) => {
                    string!("Updated").log_debug("TSR");
                    let mst = messages_table.clone();
                    let tai = *(*taeget_id_now.clone()).borrow();
                    let msus = msw_update_sender.clone();
                    let mut lock = mst.lock().unwrap();
                    format!("send {} to {}", text, tai).log_debug("GUI");
                    if let Some(vec) = lock.get_mut(&tai) {
                        vec.push((true, text.clone()));
                        client.send_message(text, tai);
                    } else {
                        lock.insert(tai, vec![(true, text.clone())]);
                        client.send_message(text, tai);
                    }
                    msus.send(Birdge::Id(tai))
                }
            }
        }
    }

    // 2 线程监听
}

fn taget_button(display: String) -> RadioButton {
    let mut btn = RadioButton::default();
    btn.set_label(&display);
    btn.clear_visible_focus();
    btn.set_frame(fltk::enums::FrameType::FlatBox);
    btn.set_color(Color::from_rgba_tuple((0, 0, 0, 0)));
    btn.set_selection_color(Color::from_rgb(255, 255, 255));
    if let Some(mut p) = btn.parent() {
        btn.set_callback(move |_| {
            p.hide();
            p.show();
        })
    }

    btn
}
fn targets_wf() -> (Window, Rc<RefCell<Flex>>) {
    let mut targets_window = Window::new(0, 0, 160, 450, None);
    // 元素大小设置为40
    let targets_flex;
    {
        targets_flex = Rc::new(RefCell::new(Flex::default_fill().column()));
        {}
        (*targets_flex).borrow_mut().end()
    }
    targets_window.end();

    targets_window.handle({
        let targets_flex = targets_flex.clone();
        move |_, event| match event {
            Event::MouseWheel | Event::KeyDown => {
                (*targets_flex).borrow_mut().hide();
                let ev_key = event_key();
                let ev_dy = event_dy();
                let flex_y = targets_flex.borrow().y() + 20;
                let mut new_y = 0;
                if ev_key == Key::PageUp || ev_key == Key::Up || ev_dy == MouseWheel::Up {
                    if flex_y <= 0 {
                        new_y = flex_y
                    }
                }
                if ev_key == Key::PageDown || ev_key == Key::Down || ev_dy == MouseWheel::Down {
                    let flex_ally = targets_flex.borrow().children() * 40;
                    if flex_ally < 450 {
                    } else if flex_ally + flex_y < 450 {
                        new_y = 450 - flex_ally
                    } else {
                        new_y = flex_y
                    }
                }
                (*targets_flex).borrow_mut().set_pos(0, new_y);
                (*targets_flex).borrow_mut().hide();
                true
            }
            _ => false,
        }
    });
    (targets_window, targets_flex)
}
fn messages_wf() -> (Window, Rc<RefCell<Flex>>) {
    let mut messages_window = Window::new(160, 0, 640, 360, None);
    let messages_flex;
    {
        messages_flex = Rc::new(RefCell::new(Flex::default_fill().row()));
        {}
        (*messages_flex).borrow_mut().end();
    }
    messages_window.end();

    messages_window.handle({
        let messages_flex = messages_flex.clone();
        move |_, event| match event {
            Event::MouseWheel | Event::KeyDown => {
                (*messages_flex).borrow_mut().hide();
                let ev_key = event_key();
                let ev_dy = event_dy();
                let flex_y = messages_flex.borrow().y() + 20;
                let mut new_y = 0;
                if ev_key == Key::PageUp || ev_key == Key::Up || ev_dy == MouseWheel::Up {
                    if flex_y <= 0 {
                        new_y = flex_y
                    }
                }
                if ev_key == Key::PageDown || ev_key == Key::Down || ev_dy == MouseWheel::Down {
                    let flex_ally = messages_flex.borrow().children() * 40;
                    if flex_ally < 450 {
                    } else if flex_ally + flex_y < 450 {
                        new_y = 450 - flex_ally
                    } else {
                        new_y = flex_y
                    }
                }
                (*messages_flex).borrow_mut().set_pos(0, new_y);
                (*messages_flex).borrow_mut().show();
                true
            }
            _ => false,
        }
    });
    (messages_window, messages_flex)
}
fn input_wi(text_sender: Sender<Birdge>) -> (Window, Rc<RefCell<Input>>) {
    let mut win = Window::new(160, 360, 640, 90, None);
    let input = Rc::new(RefCell::new(Input::default_fill()));
    (*input)
        .borrow_mut()
        .set_frame(fltk::enums::FrameType::FlatBox);
    (*input)
        .borrow_mut()
        .set_color(Color::from_rgba_tuple((0, 0, 0, 0)));
    win.end();
    win.handle({
        let input = input.clone();
        move |_, event| match event {
            Event::KeyDown => {
                if event_key() == Key::Enter {
                    string!("Enter").log_debug("Gui_input");
                    let input_text = (*input).borrow_mut().value();
                    (*input).borrow_mut().set_value("");
                    text_sender.send(Birdge::Text(input_text));
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    });
    (win, input)
}
