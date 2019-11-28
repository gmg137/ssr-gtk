//
// home.rs
// Copyright (C) 2019 gmg137 <gmg137@live.com>
// Distributed under terms of the GPLv3 license.
//

use crate::{app::Action, ssr::SsrConfig};
use crossbeam_channel::Sender;
use gtk::prelude::*;
use gtk::{Builder, Button, Label, ListBox, ListBoxRow};

#[derive(Clone)]
pub(crate) struct Home {
    sidebar: ListBox,
    group: Label,
    refresh: Button,
    speed: Button,
    remove: Button,
    ssr_listbox: ListBox,
    group_index: Option<u8>,
    ssr_index: Option<u8>,
    sender: Sender<Action>,
}

impl Home {
    pub(crate) fn new(builder: &Builder, sender: Sender<Action>) -> Self {
        let sidebar: ListBox = builder
            .get_object("group-listbox")
            .expect("无法获取 group-listbox .");
        let group: Label = builder
            .get_object("group-name-label")
            .expect("无法获取 group-name-label .");
        let refresh: Button = builder
            .get_object("refresh-button")
            .expect("无法获取 refresh-button .");
        let speed: Button = builder
            .get_object("speed-button")
            .expect("无法获取 speed-button .");
        let remove: Button = builder
            .get_object("remove-button")
            .expect("无法获取 remove-button .");
        let ssr_listbox: ListBox = builder
            .get_object("ssr-listbox")
            .expect("无法获取 ssr-listbox .");

        let s = Home {
            sidebar,
            group,
            refresh,
            speed,
            remove,
            ssr_listbox,
            group_index: None,
            ssr_index: None,
            sender: sender.clone(),
        };
        Self::init(&s);
        s
    }

    fn init(s: &Self) {
        let sender = s.sender.clone();
        s.sidebar.connect_row_selected(move |_, row| {
            if let Some(row) = row.as_ref() {
                sender
                    .send(Action::RefreshSsrListView(row.get_index() as u8))
                    .unwrap_or(());
                sender
                    .send(Action::UpdateHomeSsrListRow(None))
                    .unwrap_or(());
            }
        });

        let sender = s.sender.clone();
        s.ssr_listbox.connect_row_selected(move |_, row| {
            if let Some(row) = row.as_ref() {
                sender
                    .send(Action::UpdateHomeSsrListRow(Some(row.get_index() as u8)))
                    .unwrap_or(());
            }
        });

        let sender = s.sender.clone();
        s.refresh.connect_clicked(move |_| {
            sender.send(Action::RefreshSubInt).unwrap_or(());
        });

        let sender = s.sender.clone();
        s.speed.connect_clicked(move |_| {
            sender.send(Action::SpeedInt).unwrap_or(());
        });

        let sender = s.sender.clone();
        s.remove.connect_clicked(move |_| {
            sender.send(Action::RemoveGroup).unwrap_or(());
        });

        s.sender.send(Action::RefreshHomeSidebar).unwrap_or(());
    }

    pub(crate) fn select_group(&mut self, index: u8) {
        self.group_index = Some(index);
    }

    pub(crate) fn select_ssr(&mut self, index: Option<u8>) {
        self.ssr_index = index;
    }

    pub(crate) fn update_sidebar(
        &self,
        group_id: u8,
        group_list: &[(String, Option<String>, Vec<SsrConfig>)],
    ) {
        self.sidebar.foreach(|w| {
            self.sidebar.remove(w);
        });

        group_list.iter().for_each(|(sl, _, _)| {
            let label = Label::new(Some(sl));
            label.set_halign(gtk::Align::Start);
            label.set_valign(gtk::Align::Fill);
            label.set_margin_start(18);
            label.set_ellipsize(pango::EllipsizeMode::End);
            label.set_max_width_chars(16);
            let row = ListBoxRow::new();
            row.set_property_height_request(53);
            row.add(&label);
            self.sidebar.insert(&row, -1);
        });
        if let Some(one_row) = self.sidebar.get_row_at_index(group_id as i32) {
            self.sidebar.select_row(Some(&one_row));
        }
        self.sidebar.show_all();
    }

    pub(crate) fn update_ssr_list_view(&self, group_name: String, ssr_list: &[SsrConfig]) {
        self.ssr_listbox.foreach(|w| {
            self.ssr_listbox.remove(w);
        });

        let mut index = 0;
        ssr_list.iter().for_each(|config| {
            let gtkbox = gtk::Box::new(gtk::Orientation::Horizontal, 4);
            gtkbox.set_margin_start(25);
            gtkbox.set_margin_end(25);
            let remarks = Label::new(Some(&config.remarks));
            remarks.set_hexpand(true);
            remarks.set_vexpand(true);
            remarks.set_halign(gtk::Align::Start);
            remarks.set_ellipsize(pango::EllipsizeMode::End);
            remarks.set_max_width_chars(30);
            gtkbox.add(&remarks);

            let delay = Label::new(Some(&config.delay));
            delay.set_hexpand(true);
            delay.set_halign(gtk::Align::End);
            delay.set_margin_end(30);
            gtkbox.add(&delay);

            //let setting = Button::new_from_icon_name(
            //Some("applications-system-symbolic"),
            //gtk::IconSize::Button,
            //);
            //setting.set_margin_start(5);
            //setting.set_margin_end(5);
            //setting.set_margin_top(5);
            //setting.set_margin_bottom(5);
            //gtkbox.add(&setting);
            let remove =
                Button::new_from_icon_name(Some("user-trash-symbolic"), gtk::IconSize::Button);
            remove.set_margin_start(5);
            remove.set_margin_top(5);
            remove.set_margin_bottom(5);
            let sender = self.sender.clone();
            remove.connect_clicked(move |_| {
                sender.send(Action::RemoveSSR).unwrap_or(());
            });
            gtkbox.add(&remove);

            let row = ListBoxRow::new();
            row.add(&gtkbox);

            self.ssr_listbox.insert(&row, -1);
            index += 1;
        });
        self.group.set_text(&group_name);
        self.ssr_listbox.show_all();
    }

    pub(crate) fn get_gr_id(&self) -> (Option<u8>, Option<u8>) {
        (self.group_index, self.ssr_index)
    }
}
