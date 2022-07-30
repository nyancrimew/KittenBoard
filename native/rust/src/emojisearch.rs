use crate::log::log_d;
use jni::objects::{JClass, JList, JMap, JObject, JString, JValue};
use jni::sys::jboolean;
use jni::JNIEnv;
use lazy_static::lazy_static;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::sync::RwLock;

const CUTOFF: i32 = 60;

lazy_static! {
    static ref DATA: RwLock<HashMap<String, Vec<String>>> = RwLock::new(HashMap::new());
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn Java_gay_crimew_inputmethod_latin_emojisearch_EmojiSearch_initNative(
    env: JNIEnv,
    class: JClass,
) {
    let mut d = DATA.write().unwrap();
    *d = get_data(&env, class);
}

#[no_mangle]
#[allow(non_snake_case)]
// TODO: fuzzy matching, support non exact search
pub extern "C" fn Java_gay_crimew_inputmethod_latin_emojisearch_EmojiSearch_searchNative(
    env: JNIEnv,
    _class: JClass,
    query: JString,
    exact: jboolean,
    outArray: JObject,
) {
    let query: String = env.get_string(query).unwrap().into();
    let exact: bool = JValue::Bool(exact).z().unwrap();

    let d = DATA.read().unwrap();

    let mut results: Vec<(&String, i32)> = d
        .iter()
        .map(|e| {
            let score =
                e.1.iter()
                    .map(|keyword| {
                        let keyword = keyword.borrow();
                        if keyword == query {
                            100
                        } else if !exact {
                            if keyword.starts_with(query.as_str()) {
                                95
                            } else if keyword.contains(query.as_str()) {
                                90
                            } else {
                                0
                            }
                        } else if keyword.starts_with(format!("{}_", query).as_str()) {
                            99
                        } else {
                            // TODO: fuzzy matching
                            0
                        }
                    })
                    .max()
                    .unwrap();
            (e.0, score)
        })
        .filter(|e| e.1 >= CUTOFF)
        .collect();
    results.sort_by_key(|e| e.1);
    results.reverse();

    let output_list = JList::from_env(&env, outArray).expect("Couldn't wrap ArrayList");

    results.iter().for_each(|e| {
        output_list.add(JObject::from(env.new_string(e.0).unwrap()));
    });
}

// TODO: we should eventually just move data loading to rust entirely,
// especially since for now this results in the data being in memory twice, for now this seems easier
fn get_data(env: &JNIEnv, emoji_search_class: JClass) -> HashMap<String, Vec<String>> {
    let data_field = env
        .get_static_field(
            env.get_object_class(emoji_search_class).unwrap(),
            "data",
            "Ljava/util/Map;",
        )
        .unwrap()
        .l()
        .unwrap();
    let data_map = JMap::from_env(env, data_field).unwrap();
    let data_iter = data_map.iter().expect("Couldn't get data iterator");

    data_iter
        .map(|f| {
            let key: String = env.get_string(JString::from(f.0)).unwrap().into();
            let values: Vec<String> = JList::from_env(env, f.1)
                .unwrap()
                .iter()
                .unwrap()
                .map(|f| env.get_string(JString::from(f)).unwrap().into())
                .collect();
            (key, values)
        })
        .collect()
}
