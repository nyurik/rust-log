//! WARNING: this is not part of the crate's public API and is subject to change at any time

use self::sealed::KVs;
use crate::{Level, Metadata, Record};
use std::fmt::Arguments;
pub use std::{file, format_args, line, module_path, stringify};

#[cfg(not(feature = "kv_unstable"))]
pub type Value<'a> = &'a str;

mod sealed {
    /// Types for the `kv` argument.
    pub trait KVs<'a> {
        fn into_kvs(self) -> Option<&'a [(&'a str, super::Value<'a>)]>;
    }
}

// Types for the `kv` argument.

impl<'a> KVs<'a> for &'a [(&'a str, Value<'a>)] {
    #[inline]
    fn into_kvs(self) -> Option<&'a [(&'a str, Value<'a>)]> {
        Some(self)
    }
}

impl<'a> KVs<'a> for () {
    #[inline]
    fn into_kvs(self) -> Option<&'a [(&'a str, Value<'a>)]> {
        None
    }
}

// Log implementation.

fn log_impl(
    args: Arguments,
    level: Level,
    &(target, module_path, file): &(&str, &'static str, &'static str),
    line: u32,
    kvs: Option<&[(&str, Value)]>,
) {
    #[cfg(not(feature = "kv_unstable"))]
    if kvs.is_some() {
        panic!(
            "key-value support is experimental and must be enabled using the `kv_unstable` feature"
        )
    }

    let mut builder = Record::builder();

    builder
        .args(args)
        .level(level)
        .target(target)
        .module_path_static(Some(module_path))
        .file_static(Some(file))
        .line(Some(line));

    #[cfg(feature = "kv_unstable")]
    builder.key_values(&kvs);

    crate::logger().log(&builder.build());
}

pub fn log<'a, K>(
    args: Arguments,
    level: Level,
    target_module_path_and_file: &(&str, &'static str, &'static str),
    line: u32,
    kvs: K,
) where
    K: KVs<'a>,
{
    log_impl(
        args,
        level,
        target_module_path_and_file,
        line,
        kvs.into_kvs(),
    )
}

pub fn enabled(level: Level, target: &str) -> bool {
    crate::logger().enabled(&Metadata::builder().level(level).target(target).build())
}

#[cfg(feature = "kv_unstable")]
mod kv_support {
    use super::*;

    use crate::kv;

    pub type Value<'a> = kv::Value<'a>;

    pub fn capture_to_value<'a, V: kv::value::ToValue + ?Sized>(v: &'a &'a V) -> Value<'a> {
        v.to_value()
    }

    pub fn capture_debug<'a, V: core::fmt::Debug + ?Sized>(v: &'a &'a V) -> Value<'a> {
        Value::from_debug(v)
    }

    pub fn capture_display<'a, V: core::fmt::Display + ?Sized>(v: &'a &'a V) -> Value<'a> {
        Value::from_display(v)
    }

    #[cfg(feature = "kv_unstable_std")]
    pub fn capture_error<'a>(v: &'a (dyn std::error::Error + 'static)) -> Value<'a> {
        Value::from_dyn_error(v)
    }

    #[cfg(feature = "kv_unstable_sval")]
    pub fn capture_sval<'a, V: sval::Value + ?Sized>(v: &'a &'a V) -> Value<'a> {
        Value::from_sval(v)
    }

    #[cfg(feature = "kv_unstable_serde")]
    pub fn capture_serde<'a, V: serde::Serialize + ?Sized>(v: &'a &'a V) -> Value<'a> {
        Value::from_serde(v)
    }
}

#[cfg(feature = "kv_unstable")]
pub use self::kv_support::*;
