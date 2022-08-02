use jni::objects::{JClass, JValue};
use jni::sys::{jfloatArray, jint, jintArray, jlong};
use jni::JNIEnv;

use crate::suggest::core::layout::proximity_info::ProximityInfo;

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn Java_com_android_inputmethod_keyboard_ProximityInfo_setProximityInfoNative(
    env: JNIEnv,
    _class: JClass,
    keyboardWidth: jint,
    keyboardHeight: jint,
    gridWidth: jint,
    gridHeight: jint,
    mostCommonkeyWidth: jint,
    mostCommonkeyHeight: jint,
    proximityChars: jintArray,
    keyCount: jint,
    keyXCoordinates: jintArray,
    keyYCoordinates: jintArray,
    keyWidths: jintArray,
    keyHeights: jintArray,
    keyCharCodes: jintArray,
    sweetSpotCenterXs: jfloatArray,
    sweetSpotCenterYs: jfloatArray,
    sweetSpotRadii: jfloatArray,
) -> jlong {
    let pi = Box::new(ProximityInfo::new(
        env,
        keyboardWidth,
        keyboardHeight,
        gridWidth,
        gridHeight,
        mostCommonkeyWidth,
        mostCommonkeyHeight,
        proximityChars,
        keyCount,
        keyXCoordinates,
        keyYCoordinates,
        keyWidths,
        keyHeights,
        keyCharCodes,
        sweetSpotCenterXs,
        sweetSpotCenterYs,
        sweetSpotRadii,
    ));
    // return raw pointer
    Box::into_raw(pi) as jlong
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn Java_com_android_inputmethod_keyboard_ProximityInfo_releaseProximityInfoNative(
    _env: JNIEnv,
    _class: JClass,
    proximityInfo: jlong,
) {
    unsafe {
        drop(Box::from_raw(proximityInfo as *mut ProximityInfo));
    }
}
