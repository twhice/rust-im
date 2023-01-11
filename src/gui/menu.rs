use std::{cell::RefCell, collections::HashMap, rc::Rc, thread, time::Duration};

use fltk::{
    app::App,
    button::*,
    enums::{Color, Event},
    group::Flex,
    prelude::{GroupExt, WidgetBase, WidgetExt, WidgetId, WindowExt},
    window::Window,
};

use crate::backend::client::api::Client;

pub fn menu(client: Client) {
    let app = App::default().load_system_fonts();
    let win_sizex = Rc::new(RefCell::new(800));
    let win_sizey = Rc::new(RefCell::new(450));

    // 获取在线列表
    let mut targets_hm = HashMap::new();
    loop {
        targets_hm = client.get_userlist();
        if targets_hm.len() != 0 {
            break;
        }
        thread::sleep(Duration::from_micros(10));
    }

    // 聊天对象
    let taeget_id = Rc::new(RefCell::new(0));

    // 消息表
    let messages: Rc<RefCell<HashMap<usize, Vec<(bool, String)>>>> = Default::default();

    let mut menu = Window::default()
        .with_size(*win_sizex.borrow(), *win_sizey.borrow())
        .center_screen();
    {
        let mut targets_flex =
            Flex::new(0, 0, *win_sizex.borrow() / 5, *win_sizey.borrow(), None).column();
        {
            for (_, user) in &targets_hm {
                let mut btn = taget_button(user.get_name());
                targets_flex.set_size(&btn, 40);
                let id = user.get_id();
                btn.set_callback({
                    let tid = taeget_id.clone();
                    move |_| {
                        *tid.borrow_mut() = id;
                    }
                });
            }
        }
        targets_flex.end();
    }
    menu.show();
    menu.end();

    menu.handle({
        // let mut nx = 800;
        // let mut ny = 450;
        move |w, event| match event {
            Event::Resize => true,
            _ => false,
        }
    });

    app.run().unwrap()
}

fn taget_button(display: String) -> RadioButton {
    let mut btn = RadioButton::default();
    btn.set_label(&display);
    btn.clear_visible_focus();
    btn.set_frame(fltk::enums::FrameType::FlatBox);
    btn.set_color(Color::from_rgba_tuple((0, 0, 0, 0)));
    btn.set_selection_color(Color::from_rgb(255, 255, 255));
    btn
}
