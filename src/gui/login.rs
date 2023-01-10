use std::{cell::RefCell, rc::Rc, sync::Arc, thread, time::Duration};

use fltk::{
    app::App,
    button::Button,
    enums::{Color, FrameType},
    frame::Frame,
    group::Flex,
    image::JpegImage,
    input::{Input, IntInput},
    prelude::{ButtonExt, GroupExt, ImageExt, InputExt, WidgetBase, WidgetExt},
    window::Window,
};

use crate::backend::client::api::Client;

pub fn login(client: Client) {
    let app = App::default().load_system_fonts();
    let mut main_win = Window::default().with_size(600, 300).with_label("载入");
    {
        let wayis_login = Rc::new(RefCell::new(false));

        let img = Flex::new(0, 0, 200, 300, None).column();
        {
            let mut head =
                JpegImage::load("/home/twhicer/code/rust/learn/fl/src/head1.png").unwrap();
            head.scale(180, 180, false, false);

            let mut frame = Frame::default().with_size(180, 180).center_of(&img);
            frame.set_image(Some(head));
            frame.set_frame(FrameType::OFlatFrame);
        }
        img.end();

        let mut login = login_flex(client.clone());
        let mut sighup = sighup_flex(client.clone());

        let mut switch = Button::new(204, 280, 98, 20, "注册");
        switch.clear_visible_focus();
        switch.set_down_frame(FrameType::NoBox);
        input_style(&mut switch);
        switch.set_callback({
            let way = wayis_login.clone();
            move |switch| {
                let not = !*way.borrow();
                *(*way).borrow_mut() = not;
                match *wayis_login.borrow() {
                    true => {
                        switch.set_label("注册");
                        login.show();
                        sighup.hide()
                    }
                    false => {
                        switch.set_label("登录");
                        login.hide();
                        sighup.show()
                    }
                }
            }
        });
        switch.do_callback();

        let mut bar = Frame::new(200, 0, 3, 300, None);
        bar.set_frame(FrameType::EmbossedBox);
    }

    let (s, r) = fltk::app::channel();
    let clone_client = client.clone();
    thread::spawn(move || loop {
        if clone_client.is_login() {
            s.send(true);
            break;
        }
        thread::sleep(Duration::from_millis(10));
    });

    main_win.end();
    main_win.show();
    while app.wait() {
        if let Some(_) = r.recv() {
            app.quit();
        }
    }
}

fn input_style(widget: &mut dyn WidgetExt) {
    widget.set_frame(FrameType::FlatBox);
    widget.set_color(Color::from_rgba_tuple((0, 0, 0, 0)))
}

fn login_flex(client: Client) -> Flex {
    let mut name: Input;
    let mut id: IntInput;
    let mut pwd: Input;
    let mut login = Flex::new(240, 50, 320, 180, None).column();
    {
        let mut description = Frame::default().with_label("登录");
        description.set_label_size(30);

        let mut name_line = Flex::default().row();
        name_line.set_frame(FrameType::NoBox);
        {
            let mut t = Frame::default().with_label("昵称");
            t.set_frame(FrameType::NoBox);
            name_line.set_size(&t, 80);

            name = Input::default();
            input_style(&mut name);
        }
        name_line.end();

        let mut id_line = Flex::default().row();
        id_line.set_frame(FrameType::NoBox);
        {
            let mut t = Frame::default().with_label("ID");
            t.set_frame(FrameType::NoBox);
            id_line.set_size(&t, 80);

            id = IntInput::default();
            input_style(&mut id)
        }
        id_line.end();

        let mut pwd_line = Flex::default().row();
        pwd_line.set_frame(FrameType::NoBox);
        {
            let mut t = Frame::default().with_label("密码");
            t.set_frame(FrameType::NoBox);
            pwd_line.set_size(&t, 80);

            pwd = Input::default();
            input_style(&mut pwd)
        }
        pwd_line.end();

        let mut ctl_line = Flex::default().row();
        ctl_line.set_frame(FrameType::NoBox);
        {
            let mut warn = Frame::default();
            ctl_line.set_size(&warn, 200);

            let mut active = Button::default();
            login_button_style(&mut active);

            active.set_callback({
                let mut client = client.clone();

                move |_| {
                    let name = name.value();
                    let pwd = pwd.value();
                    if name.len() == 0 || pwd.len() == 0 {
                        warn.set_label("请正确填写!");
                        return;
                    }
                    client.login(name, id.value().parse::<usize>().unwrap(), pwd);
                }
            })
        }
        ctl_line.end();
    }
    login.hide();
    login.end();
    login
}

fn sighup_flex(client: Client) -> Flex {
    let mut name: Input;
    let mut pwd: Input;
    let mut sighup = Flex::new(240, 50, 320, 180, None).column();
    {
        let mut description = Frame::default().with_label("注册");
        description.set_label_size(30);

        let mut name_line = Flex::default().row();
        name_line.set_frame(FrameType::NoBox);
        {
            let mut t = Frame::default().with_label("昵称");
            t.set_frame(FrameType::NoBox);
            name_line.set_size(&t, 80);

            name = Input::default();
            input_style(&mut name);
        }
        name_line.end();

        let mut pwd_line = Flex::default().row();
        pwd_line.set_frame(FrameType::NoBox);
        {
            let mut t = Frame::default().with_label("密码");
            t.set_frame(FrameType::NoBox);
            pwd_line.set_size(&t, 80);

            pwd = Input::default();
            input_style(&mut pwd)
        }
        pwd_line.end();

        let mut ctl_line = Flex::default().row();
        ctl_line.set_frame(FrameType::NoBox);
        {
            let mut warn = Frame::default();
            warn.set_frame(FrameType::NoBox);
            ctl_line.set_size(&warn, 200);

            let mut active = Button::default();
            login_button_style(&mut active);
            active.set_callback({
                let mut client = client.clone();
                let name = Rc::new(RefCell::new(name));
                let pwd = Rc::new(RefCell::new(pwd));
                move |_| {
                    let name = name.borrow().value();
                    let pwd = pwd.borrow().value();
                    if name.len() == 0 || pwd.len() == 0 {
                        warn.set_label("请正确填写!");
                        return;
                    }
                    client.sighup(name, pwd);
                }
            })
        }
        ctl_line.end();
    }
    sighup.hide();
    sighup.end();
    sighup
}

fn login_button_style(btn: &mut Button) {
    btn.set_label(">");
    btn.set_label_size(30);
    btn.clear_visible_focus();
    btn.set_selection_color(Color::White);
    btn.set_frame(FrameType::FlatBox);
}
