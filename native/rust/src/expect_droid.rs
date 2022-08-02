use std::fmt;

use jni::JNIEnv;

use crate::log::log_e;

pub trait ResultExt<T, E> {
    fn expect_droid(self, env: &JNIEnv, msg: &str) -> T
    where
        E: fmt::Debug;
}

impl<T, E> ResultExt<T, E> for Result<T, E> {
    #[inline]
    fn expect_droid(self, env: &JNIEnv, msg: &str) -> T
    where
        E: fmt::Debug,
    {
        match self {
            Ok(t) => t,
            Err(e) => unwrap_failed(env, msg, &e),
        }
    }
}

#[inline(never)]
fn unwrap_failed(env: &JNIEnv, msg: &str, error: &dyn fmt::Debug) -> ! {
    log_e(env, "latinimers", format!("{msg}: {error:?}").as_str());
    panic!("{msg}: {error:?}")
}
