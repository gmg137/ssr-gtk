//
// db.rs
// Copyright (C) 2019 gmg137 <gmg137@live.com>
// Distributed under terms of the GPLv3 license.
//

use crate::{ssr::*, CONFIG_PATH};
use sled::{Db, Error};

pub struct Data {
    db: Result<Db, Error>,
}

impl Data {
    pub fn new() -> Self {
        Data {
            db: Db::open(format!("{}/db", CONFIG_PATH.to_owned())),
        }
    }

    pub fn add_sub(
        &mut self,
        url: String,
        group: String,
        configs: Vec<SsrConfig>,
    ) -> Option<Vec<(String, Option<String>, Vec<SsrConfig>)>> {
        if let Some(mut data) = self.get_all() {
            data.push((group, Some(url), configs));
            if self.set_all(&data).is_ok() {
                return Some(data);
            }
        } else {
            let data = vec![(group, Some(url), configs)];
            if self.set_all(&data).is_ok() {
                return Some(data);
            }
        }
        None
    }

    pub fn add_ssr_url(
        &mut self,
        config: SsrConfig,
    ) -> Option<(u8, Vec<(String, Option<String>, Vec<SsrConfig>)>)> {
        if let Some(mut data) = self.get_all() {
            let mut f = false;
            let mut i = 0;
            for (group, _, configs) in data.iter_mut() {
                if group.eq(&"默认") {
                    configs.push(config.to_owned());
                    f = true;
                    break;
                }
                i += 1;
            }
            if !f {
                data.push(("默认".to_owned(), None, vec![config]));
            }
            if self.set_all(&data).is_ok() {
                return Some((i, data));
            }
        } else {
            let data = vec![("默认".to_owned(), None, vec![config])];
            if self.set_all(&data).is_ok() {
                return Some((0, data));
            }
        }
        None
    }

    pub fn get_all(&self) -> Option<Vec<(String, Option<String>, Vec<SsrConfig>)>> {
        let ssr_data = self.db.as_ref().ok()?.get(b"ssr_data").ok()??;
        serde_json::from_slice::<Vec<(String, Option<String>, Vec<SsrConfig>)>>(&ssr_data).ok()
    }

    pub fn set_all(
        &mut self,
        configs: &Vec<(String, Option<String>, Vec<SsrConfig>)>,
    ) -> Result<(), Error> {
        if let Ok(configs_vec) = serde_json::to_vec(&configs) {
            if let Ok(db) = self.db.as_ref() {
                db.insert(b"ssr_data", configs_vec)?;
                db.flush()?;
            }
        }
        Ok(())
    }
}
