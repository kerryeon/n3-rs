use std::ops::Deref;

use super::core::{Query, Vars};
use crate::error::Result;
use crate::graph::{ToValues, Values};

use inflector::Inflector;

#[derive(Clone, Debug)]
pub struct EnvVars {
    inner: Vars,
}

impl Deref for EnvVars {
    type Target = Vars;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl EnvVars {
    pub fn load(mut query: Vec<Query>) -> Result<Self> {
        for entry in &mut query {
            if entry.value.is_none() {
                entry.value = Self::load_from_env(entry.name);
            }
        }

        Ok(Self {
            inner: Vars::load(query)?,
        })
    }

    fn load_from_env(key: &str) -> Option<String> {
        let key = format!("N3_{}", key.to_screaming_snake_case());
        std::env::var(&key).ok()
    }
}

impl ToValues for EnvVars {
    fn to_values(&self) -> Values {
        self.inner.to_values()
    }
}
