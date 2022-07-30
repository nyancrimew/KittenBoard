use jni::objects::{JObject, JValue};
use jni::JNIEnv;

#[allow(dead_code)]
pub fn log_d(env: &JNIEnv, tag: &str, message: &str) {
    let class = env.find_class("android/util/Log").unwrap();
    let tag = env.new_string(String::from(tag)).unwrap();
    let message = env.new_string(String::from(message)).unwrap();

    env.call_static_method(
        class,
        "d",
        "(Ljava/lang/String;Ljava/lang/String;)I",
        &[
            JValue::Object(JObject::from(tag)),
            JValue::Object(JObject::from(message)),
        ],
    )
    .unwrap();
}

#[allow(dead_code)]
pub fn log_e(env: &JNIEnv, tag: &str, message: &str) {
    let class = env.find_class("android/util/Log").unwrap();
    let tag = env.new_string(String::from(tag)).unwrap();
    let message = env.new_string(String::from(message)).unwrap();

    env.call_static_method(
        class,
        "e",
        "(Ljava/lang/String;Ljava/lang/String;)I",
        &[
            JValue::Object(JObject::from(tag)),
            JValue::Object(JObject::from(message)),
        ],
    )
    .unwrap();
}

#[allow(dead_code)]
pub fn log_i(env: &JNIEnv, tag: &str, message: &str) {
    let class = env.find_class("android/util/Log").unwrap();
    let tag = env.new_string(String::from(tag)).unwrap();
    let message = env.new_string(String::from(message)).unwrap();

    env.call_static_method(
        class,
        "i",
        "(Ljava/lang/String;Ljava/lang/String;)I",
        &[
            JValue::Object(JObject::from(tag)),
            JValue::Object(JObject::from(message)),
        ],
    )
    .unwrap();
}
