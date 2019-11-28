//
// mod.rs
// Copyright (C) 2019 gmg137 <gmg137@live.com>
// Distributed under terms of the GPLv3 license.
//

pub mod home;

use crate::{app::Action, ssr::SsrConfig};
use crossbeam_channel::Sender;
use gtk::Builder;
use home::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub(crate) struct View {
    home: Rc<RefCell<Home>>,
    sender: Sender<Action>,
    data: Arc<Mutex<u8>>,
}

impl View {
    pub(crate) fn new(
        builder: &Builder,
        sender: &Sender<Action>,
        data: Arc<Mutex<u8>>,
    ) -> Rc<Self> {
        let home = Rc::new(RefCell::new(Home::new(builder, sender.clone())));

        Rc::new(View {
            home,
            sender: sender.clone(),
            data,
        })
    }

    pub(crate) fn update_home_sidebar(
        &self,
        group_id: u8,
        configs: &[(String, Option<String>, Vec<SsrConfig>)],
    ) {
        self.home.borrow_mut().update_sidebar(group_id, configs);
    }

    pub(crate) fn update_home_ssr_list(
        &self,
        id: u8,
        configs: &[(String, Option<String>, Vec<SsrConfig>)],
    ) {
        if let Some(config) = configs.get(id as usize) {
            self.home
                .borrow_mut()
                .update_ssr_list_view(config.0.to_owned(), &config.2);
        }
        self.home.borrow_mut().select_group(id);
    }

    pub(crate) fn update_home_ssr_list_row_id(&self, id: Option<u8>) {
        self.home.borrow_mut().select_ssr(id);
    }

    pub(crate) fn get_home_gr_id(&self) -> (Option<u8>, Option<u8>) {
        self.home.borrow_mut().get_gr_id()
    }
}
