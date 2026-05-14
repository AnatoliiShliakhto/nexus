use dioxus::prelude::*;
use std::str::FromStr;

#[macro_export]
macro_rules! gather {
    ($evt:expr, $($name:ident $(as $ty:ty)?),* $(,)?) => {
        {
            let mut vals = $evt.values();
            (
                $(
                    {
                        vals.iter()
                            .position(|(k, _)| k == stringify!($name))
                            .and_then(|idx| {
                                let (_, val) = vals.remove(idx);
                                if let dioxus::events::FormValue::Text(s) = val {
                                    let trimmed = s.trim();
                                    if trimmed.is_empty() {
                                        None
                                    } else {
                                        let temp_s = trimmed;
                                        $crate::parse_internal!(temp_s $(as $ty)?)
                                    }
                                } else {
                                    None
                                }
                            })
                    },
                )*
            )
        }
    };
}

#[macro_export]
macro_rules! parse_internal {
    ($s:ident) => {
        Some($s.to_string())
    };
    ($s:ident as $ty:ty) => {
        $s.parse::<$ty>().ok()
    };
}

pub trait FormEventExt {
    fn field(&self, name: &str) -> FormField;
}

#[derive(Debug, Clone)]
pub struct FormField(Option<String>);

impl FormField {
    pub fn as_str(&self) -> Option<&str> {
        self.0.as_deref()
    }

    pub fn parse<T: FromStr>(self) -> Option<T> {
        self.0?.parse().ok()
    }

    pub fn or_default<T: FromStr + Default>(self) -> T {
        self.parse().unwrap_or_default()
    }
}

impl FormEventExt for FormEvent {
    fn field(&self, name: &str) -> FormField {
        let val = self.values().into_iter().find(|(k, _)| k == name).and_then(|(_, v)| {
            if let FormValue::Text(s) = v {
                let t = s.trim();
                if t.is_empty() { None } else { Some(t.to_string()) }
            } else {
                None
            }
        });
        FormField(val)
    }
}
