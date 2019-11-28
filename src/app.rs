//
// app.rs
// Copyright (C) 2019 gmg137 <gmg137@live.com>
// Distributed under terms of the GPLv3 license.
//

use crossbeam_channel::{unbounded, Receiver, Sender};
use gio::{self, prelude::*};
use glib;
use gtk::prelude::*;
use gtk::{ApplicationWindow, Builder, Overlay};

use crate::widgets::header::*;
use crate::{
    db::*,
    ssr::*,
    view::*,
    widgets::{mark_all_notif, notice::InAppNotification},
};
use std::cell::RefCell;
use std::env;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread::spawn;

#[derive(Debug, Clone)]
pub(crate) enum Action {
    ConnectSSR,
    Subscription(String),
    AddSSRUrl(String),
    AddConfig(SsrConfig),
    RefreshSsrListView(u8),
    UpdateHomeSsrListRow(Option<u8>),
    RefreshHomeSidebar,
    SpeedInt,
    Speed(Vec<(String, Option<String>, Vec<SsrConfig>)>),
    RefreshSubInt,
    RefreshSub(Vec<(String, Option<String>, Vec<SsrConfig>)>),
    RemoveGroup,
    RemoveSSR,
    Qrcode,
    ShowNotice(String),
}

#[derive(Clone)]
pub(crate) struct App {
    window: gtk::ApplicationWindow,
    view: Rc<View>,
    header: Rc<Header>,
    notice: RefCell<Option<InAppNotification>>,
    overlay: Overlay,
    sender: Sender<Action>,
    receiver: Receiver<Action>,
}

impl App {
    pub(crate) fn new(application: &gtk::Application) -> Rc<Self> {
        let (sender, receiver) = unbounded();
        // 初始化数据锁
        let data = Arc::new(Mutex::new(0u8));

        let glade_src = include_str!("../ui/window.glade");
        let builder = Builder::new_from_string(glade_src);

        let window: ApplicationWindow = builder
            .get_object("applicationwindow")
            .expect("Couldn't get window");
        window.set_application(Some(application));
        window.set_title("SSR-GTK");

        let view = View::new(&builder, &sender, data.clone());
        let header = Header::new(&builder, &sender, data.clone());

        window.show_all();

        let weak_app = application.downgrade();
        window.connect_delete_event(move |_, _| {
            let app = match weak_app.upgrade() {
                Some(a) => a,
                None => return Inhibit(false),
            };

            app.quit();
            return Inhibit(false);
        });

        let overlay: Overlay = builder.get_object("overlay").unwrap();

        let notice = RefCell::new(None);

        let app = App {
            window,
            header,
            view,
            notice,
            overlay,
            sender,
            receiver,
        };
        Rc::new(app)
    }

    fn init(app: &Rc<Self>) {
        // Setup the Action channel
        gtk::timeout_add(25, crate::clone!(app => move || app.setup_action_channel()));
    }

    fn setup_action_channel(&self) -> glib::Continue {
        use crossbeam_channel::TryRecvError;

        let action = match self.receiver.try_recv() {
            Ok(a) => a,
            Err(TryRecvError::Empty) => return glib::Continue(true),
            Err(TryRecvError::Disconnected) => {
                unreachable!("How the hell was the action channel dropped.")
            }
        };

        match action {
            Action::ConnectSSR => {
                let (gid, sid) = self.view.get_home_gr_id();
                if gid.is_some() && sid.is_some() {
                    let db = Data::new();
                    if let Some(configs) = db.get_all() {
                        if !run(&configs[gid.unwrap() as usize].2[sid.unwrap_or(0) as usize]) {
                            self.header.disc_button_active();
                            self.sender
                                .send(Action::ShowNotice("连接失败!".to_owned()))
                                .unwrap_or(());
                        }
                    } else {
                        self.header.disc_button_active();
                        self.sender
                            .send(Action::ShowNotice("连接失败!".to_owned()))
                            .unwrap_or(());
                    }
                } else {
                    self.header.disc_button_active();
                    self.sender
                        .send(Action::ShowNotice("请先选中要连接的 ssr 条目!".to_owned()))
                        .unwrap_or(());
                }
            }
            Action::Subscription(url) => {
                if let Some(configs) = add_sub(url) {
                    self.view
                        .update_home_sidebar((configs.len() - 1) as u8, &configs);
                } else {
                    self.sender
                        .send(Action::ShowNotice("添加订阅失败!".to_owned()))
                        .unwrap_or(());
                }
            }
            Action::AddSSRUrl(url) => {
                if let Some((group_id, configs)) = add_ssr_url(url) {
                    self.view.update_home_sidebar(group_id, &configs);
                    self.sender
                        .send(Action::ShowNotice("添加成功!".to_owned()))
                        .unwrap_or(());
                } else {
                    self.sender
                        .send(Action::ShowNotice("添加 SSR 链接失败!".to_owned()))
                        .unwrap_or(());
                }
            }
            Action::AddConfig(config) => println!("{:?}", config),
            Action::RefreshHomeSidebar => {
                let db = Data::new();
                if let Some(configs) = db.get_all() {
                    self.view.update_home_sidebar(0, &configs);
                }
            }
            Action::RefreshSsrListView(id) => {
                let db = Data::new();
                if let Some(configs) = db.get_all() {
                    self.view.update_home_ssr_list(id, &configs);
                }
            }
            Action::UpdateHomeSsrListRow(id) => self.view.update_home_ssr_list_row_id(id),
            Action::RefreshSubInt => {
                let (gid, _) = self.view.get_home_gr_id();
                if let Some(id) = gid {
                    let db = Data::new();
                    if let Some(mut configs) = db.get_all() {
                        let sender_clone = self.sender.clone();
                        spawn(move || {
                            if let Some(value) = configs.get(id as usize) {
                                if let Ok(config) =
                                    ssr_sub_url_parse(&value.1.as_ref().unwrap_or(&String::new()))
                                {
                                    configs[id as usize] =
                                        (value.0.to_owned(), value.1.to_owned(), config);
                                    sender_clone.send(Action::RefreshSub(configs)).unwrap_or(());
                                } else {
                                    sender_clone
                                        .send(Action::ShowNotice("更新订阅失败!".to_owned()))
                                        .unwrap_or(());
                                }
                            }
                        });
                    }
                }
            }
            Action::RefreshSub(configs) => {
                let (gid, _) = self.view.get_home_gr_id();
                if let Some(id) = gid {
                    let mut db = Data::new();
                    db.set_all(&configs).ok();
                    self.view.update_home_ssr_list(id, &configs);
                    self.sender
                        .send(Action::ShowNotice("更新订阅成功!".to_owned()))
                        .unwrap_or(());
                }
            }
            Action::SpeedInt => {
                let (gid, _) = self.view.get_home_gr_id();
                if let Some(id) = gid {
                    let db = Data::new();
                    if let Some(mut configs) = db.get_all() {
                        let sender_clone = self.sender.clone();
                        spawn(move || {
                            if let Some(value) = configs.get_mut(id as usize) {
                                value.2.iter_mut().for_each(|config| {
                                    if let Some(time) = timeout(config) {
                                        config.delay = format!("{} ms", time);
                                    } else {
                                        config.delay = String::from("超时");
                                    }
                                });
                                sender_clone.send(Action::Speed(configs)).unwrap_or(());
                            }
                        });
                    }
                }
            }
            Action::Speed(configs) => {
                let (gid, _) = self.view.get_home_gr_id();
                if let Some(id) = gid {
                    let mut db = Data::new();
                    db.set_all(&configs).ok();
                    self.view.update_home_ssr_list(id, &configs);
                    self.sender
                        .send(Action::ShowNotice("测速完成!".to_string()))
                        .unwrap_or(());
                }
            }
            Action::RemoveGroup => {
                let (gid, _) = self.view.get_home_gr_id();
                if let Some(id) = gid {
                    let mut db = Data::new();
                    if let Some(mut configs) = db.get_all() {
                        configs.remove(id as usize);
                        db.set_all(&configs).ok();
                        self.view.update_home_sidebar(0, &configs);
                    }
                }
            }
            Action::RemoveSSR => {
                let (gid, rid) = self.view.get_home_gr_id();
                if let Some(gid) = gid {
                    let mut db = Data::new();
                    if let Some(mut configs) = db.get_all() {
                        if let Some(rid) = rid {
                            configs[gid as usize].2.remove(rid as usize);
                            db.set_all(&configs).ok();
                            self.view.update_home_sidebar(gid, &configs);
                        } else {
                            self.sender
                                .send(Action::ShowNotice("请先选中要删除的条目:)".to_string()))
                                .unwrap_or(());
                        }
                    }
                }
            }
            Action::Qrcode => {
                if let Some((group_id, configs)) = add_qrcode() {
                    self.view.update_home_sidebar(group_id, &configs);
                    self.sender
                        .send(Action::ShowNotice("添加成功!".to_owned()))
                        .unwrap_or(());
                } else {
                    self.sender
                        .send(Action::ShowNotice("扫码添加失败!".to_owned()))
                        .unwrap_or(());
                }
            }
            Action::ShowNotice(text) => {
                let notif = mark_all_notif(text);
                let old = self.notice.replace(Some(notif));
                old.map(|i| i.destroy());
                self.notice.borrow().as_ref().map(|i| i.show(&self.overlay));
            }
        }

        glib::Continue(true)
    }

    pub(crate) fn run() {
        let application = gtk::Application::new(
            Some("com.github.gmg137.ssr-gtk"),
            gio::ApplicationFlags::empty(),
        )
        .expect("Application initialization failed...");

        let weak_app = application.downgrade();
        application.connect_startup(move |_| {
            if let Some(application) = weak_app.upgrade() {
                let app = Self::new(&application);
                Self::init(&app);

                let weak = Rc::downgrade(&app);
                application.connect_activate(move |_| {
                    if let Some(app) = weak.upgrade() {
                        // Ideally Gtk4/GtkBuilder make this irrelvent
                        app.window.show_all();
                        app.window.present();
                    } else {
                        debug_assert!(false, "I hate computers");
                    }
                });
            };
        });

        glib::set_application_name("ssr-gtk");
        glib::set_prgname(Some("ssr-gtk"));
        gtk::Window::set_default_icon_name("mail-send-symbolic");
        let args: Vec<String> = env::args().collect();
        ApplicationExtManual::run(&application, &args);
    }
}
