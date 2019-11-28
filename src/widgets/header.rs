//
// header.rs
// Copyright (C) 2019 gmg137 <gmg137@live.com>
// Distributed under terms of the GPLv3 license.
//

use crate::app::Action;
use crate::ssr::{is_run, stop, SsrConfig};
use crate::APP_VERSION;
use crate::{clone, upgrade_weak};
use crossbeam_channel::Sender;
use gtk::prelude::*;
use gtk::{AboutDialog, Builder, Button, ComboBoxText, Dialog, Entry, RadioButton};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub(crate) struct Header {
    conn_button: RadioButton,
    disc_button: RadioButton,
    subscription_button: Button,
    subscription_dialog: SubDialog,
    add_url_button: Button,
    addurl_dialog: AddUrlDialog,
    manual_setting_button: Button,
    manual_setting_dialog: ManualSettingDialog,
    qrcode_button: Button,
    about_button: Button,
    about_dialog: AboutDialog,
    sender: Sender<Action>,
    data: Arc<Mutex<u8>>,
}

#[derive(Clone)]
pub(crate) struct SubDialog {
    dialog: Dialog,
    url_entry: Entry,
    sub_button: Button,
}

#[derive(Clone)]
pub(crate) struct AddUrlDialog {
    dialog: Dialog,
    ssr_entry: Entry,
    add_button: Button,
}

#[derive(Clone)]
pub(crate) struct ManualSettingDialog {
    dialog: Dialog,
    cancel_button: Button,
    finished_button: Button,
    group_entry: Entry,
    configname_entry: Entry,
    server_entry: Entry,
    remote_port_entry: Entry,
    local_port_entry: Entry,
    password_entry: Entry,
    method_combo: ComboBoxText,
    protocol_combo: ComboBoxText,
    protoparam_entry: Entry,
    obfs_combo: ComboBoxText,
    obfsparam_entry: Entry,
}

impl Header {
    pub(crate) fn new(
        builder: &Builder,
        sender: &Sender<Action>,
        data: Arc<Mutex<u8>>,
    ) -> Rc<Self> {
        let conn_button: RadioButton = builder
            .get_object("conn-button")
            .expect("Couldn't get conn button");
        let disc_button: RadioButton = builder
            .get_object("disc-button")
            .expect("Couldn't get disc button");
        let subscription_button: Button = builder
            .get_object("subscription-button")
            .expect("Couldn't get subscription button");
        let dialog: Dialog = builder
            .get_object("subscription-dialog")
            .expect("Couldn't get sub dialog");
        let url_entry: Entry = builder
            .get_object("url-entry")
            .expect("Couldn't get url entry");
        let sub_button: Button = builder
            .get_object("sub-button")
            .expect("Couldn't get sub button");
        let subscription_dialog = SubDialog {
            dialog,
            url_entry,
            sub_button,
        };
        let add_url_button: Button = builder
            .get_object("add-url-button")
            .expect("Couldn't get add-url button");
        let dialog: Dialog = builder
            .get_object("addurl-dialog")
            .expect("Couldn't get addurl dialog");
        let ssr_entry: Entry = builder
            .get_object("ssr-entry")
            .expect("Couldn't get ssr entry");
        let add_button: Button = builder
            .get_object("add-button")
            .expect("Couldn't get add button");
        let addurl_dialog = AddUrlDialog {
            dialog,
            ssr_entry,
            add_button,
        };
        let manual_setting_button: Button = builder
            .get_object("manual-setting-button")
            .expect("Couldn't get manual_setting_button");
        let dialog: Dialog = builder
            .get_object("ssr-setting-dialog")
            .expect("Couldn't get ssr-setting-dialog");
        let cancel_button: Button = builder
            .get_object("cancel-button")
            .expect("Couldn't get cancel_button");
        let finished_button: Button = builder
            .get_object("finished-button")
            .expect("Couldn't get finished_button");
        let group_entry: Entry = builder
            .get_object("group-entry")
            .expect("Couldn't get group_entry");
        let configname_entry: Entry = builder
            .get_object("configname-entry")
            .expect("Couldn't get configname_entry");
        let server_entry: Entry = builder
            .get_object("server-entry")
            .expect("Couldn't get server_entry");
        let remote_port_entry: Entry = builder
            .get_object("remote-port-entry")
            .expect("Couldn't get remote_port_entry");
        let local_port_entry: Entry = builder
            .get_object("local-port-entry")
            .expect("Couldn't get local_port_entry");
        let password_entry: Entry = builder
            .get_object("password-entry")
            .expect("Couldn't get password_entry");
        let method_combo: ComboBoxText = builder
            .get_object("method-combo")
            .expect("Couldn't get method_combo");
        let protocol_combo: ComboBoxText = builder
            .get_object("protocol-combo")
            .expect("Couldn't get protocol_combo");
        let protoparam_entry: Entry = builder
            .get_object("protoparam-entry")
            .expect("Couldn't get protoparam_entry");
        let obfs_combo: ComboBoxText = builder
            .get_object("obfs-combo")
            .expect("Couldn't get obfs_combo");
        let obfsparam_entry: Entry = builder
            .get_object("obfsparam-entry")
            .expect("Couldn't get obfsparam_entry");
        let manual_setting_dialog = ManualSettingDialog {
            dialog,
            configname_entry,
            cancel_button,
            finished_button,
            group_entry,
            server_entry,
            remote_port_entry,
            local_port_entry,
            password_entry,
            method_combo,
            protocol_combo,
            protoparam_entry,
            obfs_combo,
            obfsparam_entry,
        };
        let qrcode_button: Button = builder
            .get_object("qrcode-button")
            .expect("Couldn't get qrcode_button");
        let about_button: Button = builder
            .get_object("about-button")
            .expect("Couldn't get about_button");
        let about_dialog: AboutDialog = builder
            .get_object("about-dialog")
            .expect("Couldn't get about_dialog");
        let header = Header {
            conn_button,
            disc_button,
            subscription_button,
            subscription_dialog,
            add_url_button,
            addurl_dialog,
            manual_setting_button,
            manual_setting_dialog,
            qrcode_button,
            about_button,
            about_dialog,
            sender: sender.clone(),
            data: data.clone(),
        };
        let h = Rc::new(header);
        Self::init(&h, &sender);
        h
    }

    fn init(s: &Rc<Self>, sender: &Sender<Action>) {
        // 修改连接按钮样式
        let button_weak = s.conn_button.downgrade();
        s.conn_button.connect_toggled(clone!(button_weak=>move|_| {
            let button = upgrade_weak!(button_weak);
            let context = button.get_style_context();
            if button.get_active(){
                context.remove_class("suggested-action");
            } else {
                context.add_class("suggested-action");
            }
        }));

        // 修改断开按钮样式
        let button_weak = s.disc_button.downgrade();
        s.disc_button.connect_toggled(clone!(button_weak=>move|_| {
            let button = upgrade_weak!(button_weak);
            let context = button.get_style_context();
            if button.get_active(){
                context.remove_class("destructive-action");
            } else {
                context.add_class("destructive-action");
            }
        }));

        // 初始化连接按钮
        if is_run() {
            s.conn_button.set_active(true);
        }

        // 连接断开按钮行为
        let button_weak = s.conn_button.downgrade();
        let sender_clone = sender.clone();
        s.conn_button.connect_clicked(clone!(button_weak=>move|_| {
            let button = upgrade_weak!(button_weak);
            if button.get_active(){
                sender_clone.send(Action::ConnectSSR).unwrap();
            } else {
                stop();
            }
        }));

        // 订阅
        let dialog_weak = s.subscription_dialog.dialog.downgrade();
        s.subscription_button
            .connect_clicked(clone!(dialog_weak=>move|_| {
                let dialog = upgrade_weak!(dialog_weak);
                dialog.run();
                dialog.hide();
            }));

        // 订阅按钮
        let sender_clone = sender.clone();
        let dialog_weak = s.subscription_dialog.dialog.downgrade();
        let entry_weak = s.subscription_dialog.url_entry.downgrade();
        s.subscription_dialog
            .sub_button
            .connect_clicked(clone!(dialog_weak,entry_weak=>move|_| {
                let dialog = upgrade_weak!(dialog_weak);
                if let Some(url) = upgrade_weak!(entry_weak).get_text(){
                    if !url.is_empty(){
                        sender_clone.send(Action::Subscription(url.to_owned())).unwrap_or(());
                    }
                }
                dialog.hide();
            }));

        // 通过URL添加
        let dialog_weak = s.addurl_dialog.dialog.downgrade();
        s.add_url_button
            .connect_clicked(clone!(dialog_weak=>move|_| {
                let dialog = upgrade_weak!(dialog_weak);
                dialog.run();
                dialog.hide();
            }));

        // 添加按钮
        let sender_clone = sender.clone();
        let dialog_weak = s.addurl_dialog.dialog.downgrade();
        let entry_weak = s.addurl_dialog.ssr_entry.downgrade();
        s.addurl_dialog
            .add_button
            .connect_clicked(clone!(dialog_weak,entry_weak=>move|_| {
                let dialog = upgrade_weak!(dialog_weak);
                if let Some(url) = upgrade_weak!(entry_weak).get_text(){
                    if !url.is_empty(){
                        sender_clone.send(Action::AddSSRUrl(url.to_owned())).unwrap_or(());
                    }
                }
                dialog.hide();
            }));

        // 手动添加
        let dialog_weak = s.manual_setting_dialog.dialog.downgrade();
        s.manual_setting_button
            .connect_clicked(clone!(dialog_weak=>move|_| {
                let dialog = upgrade_weak!(dialog_weak);
                dialog.run();
                dialog.hide();
            }));

        // 取消添加
        let dialog_weak = s.manual_setting_dialog.dialog.downgrade();
        s.manual_setting_dialog
            .cancel_button
            .connect_clicked(clone!(dialog_weak=>move|_| {
                let dialog = upgrade_weak!(dialog_weak);
                dialog.hide();
            }));

        // 确定添加
        let dialog_weak = s.manual_setting_dialog.dialog.downgrade();
        let group_weak = s.manual_setting_dialog.group_entry.downgrade();
        let configname_weak = s.manual_setting_dialog.configname_entry.downgrade();
        let server_weak = s.manual_setting_dialog.server_entry.downgrade();
        let remote_port_weak = s.manual_setting_dialog.remote_port_entry.downgrade();
        let local_port_weak = s.manual_setting_dialog.local_port_entry.downgrade();
        let password_weak = s.manual_setting_dialog.password_entry.downgrade();
        let method_weak = s.manual_setting_dialog.method_combo.downgrade();
        let protocol_weak = s.manual_setting_dialog.protocol_combo.downgrade();
        let protoparam_weak = s.manual_setting_dialog.protoparam_entry.downgrade();
        let obfs_weak = s.manual_setting_dialog.obfs_combo.downgrade();
        let obfsparam_weak = s.manual_setting_dialog.obfsparam_entry.downgrade();
        let sender_clone = sender.clone();
        s.manual_setting_dialog.finished_button.connect_clicked(
            clone!(dialog_weak,group_weak,configname_weak,
                    server_weak,remote_port_weak,local_port_weak,password_weak,
                    method_weak,method_weak,protocol_weak,protoparam_weak,
                    obfs_weak,obfsparam_weak=>move|_| {
                let dialog = upgrade_weak!(dialog_weak);
                let remote_addr = match upgrade_weak!(server_weak).get_text() {
                    Some(text) => text.to_owned(),
                    None => String::new()
                };
                let remote_port = match upgrade_weak!(remote_port_weak).get_text() {
                    Some(text) => text.to_owned(),
                    None => String::new()
                };
                let local_port = match upgrade_weak!(local_port_weak).get_text() {
                    Some(text) => text.to_owned(),
                    None => String::new()
                };
                let method = match upgrade_weak!(method_weak).get_active_text() {
                    Some(text) => text.to_owned(),
                    None => String::new()
                };
                let password = match upgrade_weak!(password_weak).get_text() {
                    Some(text) => text.to_owned(),
                    None => String::new()
                };
                let protocol = match upgrade_weak!(protocol_weak).get_active_text() {
                    Some(text) => text.to_owned(),
                    None => String::new()
                };
                let protoparam = match upgrade_weak!(protoparam_weak).get_text() {
                    Some(text) => text.to_owned(),
                    None => String::new()
                };
                let obfs = match upgrade_weak!(obfs_weak).get_active_text() {
                    Some(text) => text.to_owned(),
                    None => String::new()
                };
                let obfsparam = match upgrade_weak!(obfsparam_weak).get_text() {
                    Some(text) => text.to_owned(),
                    None => String::new()
                };
                let remarks = match upgrade_weak!(configname_weak).get_text() {
                    Some(text) => text.to_owned(),
                    None => String::from("未命名")
                };
                let group = match upgrade_weak!(group_weak).get_text() {
                    Some(text) => text.to_owned(),
                    None => String::from("默认")
                };
                sender_clone.send(Action::AddConfig(SsrConfig{
                    local_addr: "127.0.0.1".to_owned(),
                    timeout: "300".to_owned(),
                    group,
                    remarks,
                    obfsparam,
                    obfs,
                    protoparam,
                    protocol,
                    password,
                    method,
                    local_port,
                    remote_port,
                    remote_addr,
                    delay: String::from("0 ms")
                })).unwrap_or(());

                dialog.hide();
            }),
        );

        // 扫码按钮
        let sender = s.sender.clone();
        s.qrcode_button.connect_clicked(move |_| {
            sender.send(Action::Qrcode).unwrap_or(());
        });

        // 设置关于窗口版本号
        s.about_dialog.set_version(Some(APP_VERSION));

        // 关于按钮
        let about_weak = s.about_dialog.downgrade();
        s.about_button
            .connect_clicked(clone!(about_weak => move |_| {
            let about = upgrade_weak!(about_weak);
            about.run();
            about.hide();
            }));
    }

    pub fn disc_button_active(&self) {
        self.disc_button.set_active(true);
    }
}
