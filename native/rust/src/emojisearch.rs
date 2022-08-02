use std::cmp::{max, min};
use std::iter;
use std::ops::ControlFlow;

use jni::objects::{JClass, JList, JObject, JString, JValue};
use jni::sys::jboolean;
use jni::JNIEnv;
use levenshtein::levenshtein;

use crate::expect_droid::ResultExt;

include!(concat!(env!("OUT_DIR"), "/emoji_data.rs"));

const SCORE_CUTOFF: i32 = 87;
const SCORE_MAX: i32 = 100;

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn Java_gay_crimew_inputmethod_latin_emojisearch_EmojiSearch_searchNative(
    env: JNIEnv,
    _class: JClass,
    query: JString,
    exact: jboolean,
    outArray: JObject,
) {
    let query: String = env
        .get_string(query)
        .expect_droid(&env, "Couldn't get 'query' in rust context")
        .into();
    let exact: bool = JValue::Bool(exact)
        .z()
        .expect_droid(&env, "Couldn't get 'exact' in rust context");

    let len_query = query.chars().count();
    let mut results: Vec<(String, i32)> = EMOJI_DATA
        .iter()
        .filter_map(|e| {
            let mut maxScore: i32 = 0;
            e.1.iter().try_for_each(|&keyword| {
                let score = if keyword == query {
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
                } else if keyword.ends_with(format!("_{}", query).as_str()) {
                    97
                } else if !should_ignore_exact_query(&query)
                    && keyword.contains(format!("_{}_", query).as_str())
                {
                    96
                } else {
                    0
                };
                maxScore = max(maxScore, score);
                if maxScore == SCORE_MAX {
                    return ControlFlow::Break(());
                }
                ControlFlow::Continue(())
            });
            if maxScore >= SCORE_CUTOFF {
                Some((String::from(e.0), maxScore))
            } else {
                None
            }
        })
        .collect();
    results.sort_by_key(|e| e.1);
    results.reverse();

    let output_list = JList::from_env(&env, outArray).expect_droid(&env, "Couldn't wrap ArrayList");
    results.iter().for_each(|e| {
        let entry = JObject::from(
            env.new_string(String::from(e.0.as_str()))
                .expect_droid(&env, "Couldn't turn entry into jni string"),
        );
        output_list
            .add(entry)
            .expect_droid(&env, "Couldn't add to output list");
        env.delete_local_ref(entry)
            .expect_droid(&env, "Couldn't delete local ref to entry");
    });
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

fn should_ignore_exact_query(query: &str) -> bool {
    vec!["with", "in", "no", "and", "of", "the", "me", "on", "a"].contains(&query)
}
