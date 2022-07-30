use jni::objects::{JClass, JList, JMap, JObject, JString, JValue};
use jni::sys::jboolean;
use jni::JNIEnv;
use lazy_static::lazy_static;
use levenshtein::levenshtein;
use std::borrow::Borrow;
use std::cmp::min;
use std::collections::HashMap;
use std::iter;
use std::sync::RwLock;

const SCORE_CUTOFF: i32 = 87;
const SCORE_MAX: i32 = 100;

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

    let len_query = query.chars().count();
    let mut results: Vec<(&String, i32)> = d
        .iter()
        // TODO: somehow exit early when score 100 reached
        .filter_map(|e| {
            let score = e
                .1
                .iter()
                .map(|keyword| {
                    let keyword = keyword.borrow();

                    if keyword == query {
                        SCORE_MAX
                    } else if !exact {
                        let len_keyword = keyword.chars().count();
                        if keyword.starts_with(query.as_str()) {
                            let (len_long, len_short) = if len_keyword > len_query {
                                (len_keyword, len_query)
                            } else {
                                (len_query, len_keyword)
                            };
                            SCORE_MAX - ((len_long as f32 / len_short as f32) * 10.0).round() as i32
                        } else if keyword.contains(query.as_str()) {
                            // TODO: calculate more appropriate score here
                            90
                        } else {
                            // TODO: split up keyword/query for partial match calculation (?)
                            let distance = levenshtein(query.as_str(), keyword);
                            let lensum = len_query + len_keyword;
                            let ratio = (lensum - distance) as f32 / lensum as f32;
                            // boost result based on how much of the beginnings match
                            let bonus = mismatch(query.as_bytes(), keyword.as_bytes()) as f32;
                            min(SCORE_MAX, (ratio * 100.0 + bonus * 5.5).round() as i32)
                        }
                    } else if keyword.starts_with(format!("{}_", query).as_str()) {
                        99
                    } else {
                        0
                    }
                })
                .max()
                .unwrap();
            if score >= SCORE_CUTOFF {
                Some((e.0, score))
            } else {
                None
            }
        })
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

fn mismatch(xs: &[u8], ys: &[u8]) -> usize {
    mismatch_chunks::<128>(xs, ys)
}

fn mismatch_chunks<const N: usize>(xs: &[u8], ys: &[u8]) -> usize {
    let off = iter::zip(xs.chunks_exact(N), ys.chunks_exact(N))
        .take_while(|(x, y)| x == y)
        .count()
        * N;
    off + iter::zip(&xs[off..], &ys[off..])
        .take_while(|(x, y)| x == y)
        .count()
}
