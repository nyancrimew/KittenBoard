use std::cmp::{min, Ordering};
use std::collections::HashMap;

use jni::sys::{jfloat, jfloatArray, jint, jintArray};
use jni::JNIEnv;

use crate::defines::{KEYCODE_SPACE, MAX_KEY_COUNT_IN_A_KEYBOARD};
use crate::expect_droid::ResultExt;
use crate::math::{get_distance, get_distance_int, square_float};
use crate::suggest::core::layout::proximity_info_params::VERTICAL_SWEET_SPOT_SCALE_G;

pub const MAX_PROXIMITY_CHARS_SIZE: usize = 16;

// TODO: a lot of the vecs here can technically be slices since the max size is known
// TODO: turn coordinates into unsigned ints
pub struct ProximityInfo {
    grid_width: i32,
    grid_height: i32,
    most_common_key_width: i32,
    most_common_key_width_square: i32,
    normalized_squared_most_common_key_hypotenuse: f32,
    cell_width: i32,
    cell_height: i32,
    key_count: usize,
    keyboard_width: i32,
    keyboard_height: i32,
    keyboard_hypotenuse: f32,
    has_touch_position_correction_data: bool,
    proximity_chars: Vec<char>,
    key_x_coordinates: Vec<i32>,
    key_y_coordinates: Vec<i32>,
    key_widths: Vec<i32>,
    key_heights: Vec<i32>,
    key_char_codes: Vec<char>,
    sweet_spot_center_xs: Vec<f32>,
    sweet_spot_center_ys: Vec<f32>,
    sweet_spot_radii: Vec<f32>,
    sweet_spot_center_ys_g: Vec<f32>,
    lower_code_point_to_key_map: HashMap<char, usize>,
    key_index_to_orginal_codepoint: Vec<char>,
    key_index_to_lower_codepoint: Vec<char>,
    center_xs_g: Vec<i32>,
    center_ys_g: Vec<i32>,
    key_key_distances_g: Vec<Vec<i32>>,
}

impl ProximityInfo {
    pub fn new(
        env: JNIEnv,
        keyboard_width: jint,
        keyboard_height: jint,
        grid_width: jint,
        grid_height: jint,
        most_commonkey_width: jint,
        most_commonkey_height: jint,
        proximity_chars: jintArray,
        key_count: jint,
        key_x_coordinates: jintArray,
        key_y_coordinates: jintArray,
        key_widths: jintArray,
        key_heights: jintArray,
        key_char_codes: jintArray,
        sweet_spot_center_xs: jfloatArray,
        sweet_spot_center_ys: jfloatArray,
        sweet_spot_radii: jfloatArray,
    ) -> ProximityInfo {
        let proximity_chars_length: usize = env
            .get_array_length(proximity_chars)
            .expect_droid(&env, "Couldn't get length of proximity_chars array")
            as usize;
        if proximity_chars_length
            != (grid_width as usize * grid_height as usize * MAX_PROXIMITY_CHARS_SIZE)
        {
            // we panic since we cant just not return anything here
            panic!("Invalid proximity_chars array length")
        }

        let key_count = min(key_count as usize, MAX_KEY_COUNT_IN_A_KEYBOARD);
        let mut pi = ProximityInfo {
            grid_width,
            grid_height,
            most_common_key_width: most_commonkey_width,
            most_common_key_width_square: most_commonkey_width * most_commonkey_width,
            normalized_squared_most_common_key_hypotenuse: square_float(
                most_commonkey_height as f32 / most_commonkey_width as f32,
            ),
            cell_width: (keyboard_width + grid_width - 1) / grid_width,
            cell_height: (keyboard_height + grid_height - 1) / grid_height,
            key_count,
            keyboard_width,
            keyboard_height,
            keyboard_hypotenuse: (keyboard_width as f32).hypot(keyboard_height as f32),
            // todo: proper checks for the arrays
            has_touch_position_correction_data: key_count > 0,
            proximity_chars: jint_array_region_to_char_vec(
                env,
                proximity_chars,
                proximity_chars_length,
            ),
            key_x_coordinates: jint_array_region_to_vec(env, key_x_coordinates, key_count),
            key_y_coordinates: jint_array_region_to_vec(env, key_y_coordinates, key_count),
            key_widths: jint_array_region_to_vec(env, key_widths, key_count),
            key_heights: jint_array_region_to_vec(env, key_heights, key_count),
            key_char_codes: jint_array_region_to_char_vec(env, key_char_codes, key_count),
            sweet_spot_center_xs: jfloat_array_region_to_vec(env, sweet_spot_center_xs, key_count),
            sweet_spot_center_ys: jfloat_array_region_to_vec(env, sweet_spot_center_ys, key_count),
            sweet_spot_radii: jfloat_array_region_to_vec(env, sweet_spot_radii, key_count as usize),
            sweet_spot_center_ys_g: vec![0.0; key_count],
            lower_code_point_to_key_map: Default::default(),
            key_index_to_orginal_codepoint: vec![0 as char; key_count],
            key_index_to_lower_codepoint: vec![0 as char; key_count],
            center_xs_g: vec![0; key_count],
            center_ys_g: vec![0; key_count],
            key_key_distances_g: vec![vec![0; key_count]; key_count],
        };
        pi.initialize_geometry();
        pi
    }

    pub fn has_space_proximity(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 {
            // TODO: panic
            return false;
        }

        let start_index = get_start_index_from_coordinates(
            x,
            y,
            self.cell_height,
            self.cell_width,
            self.grid_width,
        );
        for i in 0..MAX_PROXIMITY_CHARS_SIZE {
            if self.proximity_chars[start_index + i] == KEYCODE_SPACE {
                return true;
            }
        }
        false
    }

    pub fn get_normalized_square_distance_from_center_float_g(
        &self,
        key_id: usize,
        x: i32,
        y: i32,
        is_geometric: bool,
    ) -> f32 {
        // TODO: either make all these origin values float or skip float conversion until after squaring
        let center_x = self.get_key_center_x_of_key_id_g(key_id, Some(x), is_geometric) as f32;
        let center_y = self.get_key_center_y_of_key_id_g(key_id, Some(y), is_geometric) as f32;
        square_float(center_x - center_y)
            + square_float(x as f32 - y as f32) / self.get_most_common_key_width_square() as f32
    }

    // referencePointX is used only for keys wider than most common key width. When the referencePointX
    // is NOT_A_COORDINATE, this method calculates the return value without using the line segment.
    // isGeometric is currently not used because we don't have extra X coordinates sweet spots for
    // geometric input.
    // TODO: convert to float?
    pub fn get_key_center_x_of_key_id_g(
        &self,
        key_id: usize,
        // TODO: is this ever none?
        reference_point_x: Option<i32>,
        _is_geometric: bool,
    ) -> i32 {
        let mut center_x = if self.has_touch_position_correction_data() {
            self.sweet_spot_center_xs[key_id] as i32
        } else {
            self.center_xs_g[key_id]
        };
        let key_width = self.key_widths[key_id];
        if let Some(reference_point_x) = reference_point_x {
            if key_width > self.get_most_common_key_width() {
                // For keys wider than most common keys, we use a line segment instead of the center point;
                // thus, centerX is adjusted depending on referencePointX.
                let key_width_half_diff = (key_width - self.get_most_common_key_width()) / 2;
                center_x = match reference_point_x.cmp(&(center_x + key_width_half_diff)) {
                    Ordering::Greater => center_x + key_width_half_diff,
                    Ordering::Less => center_x - key_width_half_diff,
                    Ordering::Equal => reference_point_x,
                };
            }
        }
        center_x
    }

    // When the referencePointY is NOT_A_COORDINATE, this method calculates the return value without
    // using the line segment.
    // TODO: convert to float?
    pub fn get_key_center_y_of_key_id_g(
        &self,
        key_id: usize,
        // TODO: is this ever none?
        reference_point_y: Option<i32>,
        is_geometric: bool,
    ) -> i32 {
        // TODO: Remove "is_geometric" and have separate "proximity_info"s for gesture and typing.
        let center_y = if !self.has_touch_position_correction_data() {
            self.center_ys_g[key_id]
        } else if is_geometric {
            self.sweet_spot_center_ys_g[key_id] as i32
        } else {
            self.sweet_spot_center_ys[key_id] as i32
        };
        if let Some(reference_point_y) = reference_point_y {
            if center_y + self.key_heights[key_id] > self.keyboard_height
                && center_y < reference_point_y
            {
                // When the distance between center point and bottom edge of the keyboard is shorter than
                // the key height, we assume the key is located at the bottom row of the keyboard.
                // The center point is extended to the bottom edge for such keys.
                return reference_point_y;
            }
        }
        center_y
    }

    #[inline]
    pub fn get_key_index_of(&self, c: char) -> Option<usize> {
        if self.key_count == 0 {
            // We do not have the coordinate data
            return None;
        }
        // TODO: support non-ascii (see char_util.h)
        self.lower_code_point_to_key_map
            .get(&c.to_ascii_lowercase())
            .copied()
    }

    #[inline]
    pub fn is_code_point_on_keyboard(&self, c: char) -> bool {
        self.get_key_index_of(c).is_some()
    }

    pub fn initialize_proximities(
        &self,
        input_codes: &Vec<char>,
        input_x_coordinates: &Vec<i32>,
        input_y_coordinates: &Vec<i32>,
        input_size: &usize,
        locale: &String,
    ) -> Vec<Vec<char>> {
        // Initialize
        // - mInputCodes
        // - mNormalizedSquaredDistances
        // TODO: Merge
        (0..*input_size)
            .map(|i| {
                let primary_key = input_codes[i];
                let x = input_x_coordinates[i];
                let y = input_y_coordinates[i];
                self.calculate_proximities(x, y, primary_key, locale)
            })
            .collect()
    }

    fn calculate_proximities(
        &self,
        x: i32,
        y: i32,
        primary_key: char,
        locale: &String,
    ) -> Vec<char> {
        let mut insert_pos = 0;
        let mut proximities = vec![primary_key];
        insert_pos += 1;

        if x < 0 || y < 0 {
            // TODO: panic or handle properly
            return proximities;
        }

        let start_index = get_start_index_from_coordinates(
            x,
            y,
            self.cell_height,
            self.cell_width,
            self.grid_width,
        );
        for i in 0..MAX_PROXIMITY_CHARS_SIZE {
            let c = self.proximity_chars[start_index + i];
            if c < KEYCODE_SPACE || c == primary_key {
                continue;
            }
            let key_id = self.get_key_index_of(c).unwrap();
            let on_key = self.is_on_key(key_id, x, y);
            let distance = self.squared_length_to_edge(key_id, x, y);
            if on_key || distance < self.most_common_key_width_square {
                proximities.push(c);
                if proximities.len() >= MAX_PROXIMITY_CHARS_SIZE {
                    // TODO: panic
                    return proximities;
                }
            }
        }
        // TODO: additional proximities
        // TODO: do we also need delimiters??
        proximities
    }

    fn squared_length_to_edge(&self, key_id: usize, x: i32, y: i32) -> i32 {
        let left = self.key_x_coordinates[key_id];
        let top = self.key_y_coordinates[key_id];
        let right = left + self.key_widths[key_id];
        let bottom = top + self.key_heights[key_id];
        let edge_x = if x < left {
            left
        } else if x > right {
            right
        } else {
            x
        };
        let edge_y = if y < top {
            top
        } else if y > bottom {
            bottom
        } else {
            x
        };
        let dx = x - edge_x;
        let dy = y - edge_y;

        dx * dx + dy * dy
    }

    fn is_on_key(&self, key_id: usize, x: i32, y: i32) -> bool {
        let left = self.key_x_coordinates[key_id];
        let top = self.key_y_coordinates[key_id];
        let right = left + self.key_widths[key_id] + 1;
        let bottom = top + self.key_heights[key_id];
        left < right && top < bottom && x >= left && x < right && y >= top && y < bottom
    }

    fn initialize_geometry(&mut self) {
        for i in 0..self.key_count {
            let code = self.key_char_codes[i];
            // TODO: support non-ascii lowercase (see char_utils.h)
            let lower_code = code.to_ascii_lowercase();
            self.center_xs_g[i] = self.key_x_coordinates[i] + self.key_widths[i] / 2;
            self.center_ys_g[i] = self.key_y_coordinates[i] + self.key_heights[i] / 2;
            if self.has_touch_position_correction_data() {
                // Computes sweet spot center points for geometric input.
                let gap_y = self.sweet_spot_center_ys[i] - self.center_ys_g[i] as f32;
                // TODO: the cpp impl coerces the value to int before setting it, this is probably wrong
                // so i didn't do it like that, but we might need to change this code if that results in any bugs
                self.sweet_spot_center_ys_g[i] =
                    (self.center_ys_g[i] as f32) + gap_y * VERTICAL_SWEET_SPOT_SCALE_G
            }
            self.lower_code_point_to_key_map.insert(lower_code, i);
            self.key_index_to_orginal_codepoint[i] = code;
            self.key_index_to_lower_codepoint[i] = lower_code;
        }
        for i in 0..self.key_count {
            self.key_key_distances_g[i][i] = 0;
            for j in i + 1..self.key_count {
                if self.has_touch_position_correction_data() {
                    // Computes distances using sweet spots if they exist.
                    // We have two types of Y coordinate sweet spots, for geometric and for the others.
                    // The sweet spots for geometric input are used for calculating key-key distances
                    // here.
                    self.key_key_distances_g[i][j] = get_distance(
                        self.sweet_spot_center_xs[i],
                        self.sweet_spot_center_ys_g[i],
                        self.sweet_spot_center_xs[j],
                        self.sweet_spot_center_ys_g[j],
                    );
                } else {
                    self.key_key_distances_g[i][j] = get_distance_int(
                        self.center_xs_g[i],
                        self.center_ys_g[i],
                        self.center_xs_g[j],
                        self.center_ys_g[j],
                    );
                }
                self.key_key_distances_g[j][i] = self.key_key_distances_g[i][j];
            }
        }
    }

    pub fn get_code_point_of(&self, key_index: usize) -> Option<char> {
        if !(0..self.key_count).contains(&key_index) {
            return None;
        }
        Some(self.key_index_to_lower_codepoint[key_index])
    }

    pub fn get_original_code_point_of(&self, key_index: usize) -> Option<char> {
        if !(0..self.key_count).contains(&key_index) {
            return None;
        }
        Some(self.key_index_to_orginal_codepoint[key_index])
    }

    #[inline]
    pub fn has_sweet_spot_data(&self, key_index: usize) -> bool {
        // When there are no calibration data for a key,
        // the radius of the key is assigned to zero.
        self.sweet_spot_radii[key_index] > 0.0
    }

    #[inline]
    pub fn get_sweet_spot_radii_at(&self, key_index: usize) -> f32 {
        self.sweet_spot_radii[key_index]
    }

    #[inline]
    pub fn get_sweet_spot_center_x_at(&self, key_index: usize) -> f32 {
        self.sweet_spot_center_xs[key_index]
    }

    #[inline]
    pub fn get_sweet_spot_center_y_at(&self, key_index: usize) -> f32 {
        self.sweet_spot_center_ys[key_index]
    }

    #[inline]
    pub fn has_touch_position_correction_data(&self) -> bool {
        self.has_touch_position_correction_data
    }

    #[inline]
    pub fn get_most_common_key_width(&self) -> i32 {
        self.most_common_key_width
    }

    #[inline]
    pub fn get_most_common_key_width_square(&self) -> i32 {
        self.most_common_key_width_square
    }

    #[inline]
    pub fn get_normalized_squared_most_common_key_hypotenuse(&self) -> f32 {
        self.normalized_squared_most_common_key_hypotenuse
    }

    #[inline]
    pub fn get_key_count(&self) -> usize {
        self.key_count
    }

    #[inline]
    pub fn get_cell_height(&self) -> i32 {
        self.cell_height
    }

    #[inline]
    pub fn get_cell_width(&self) -> i32 {
        self.cell_width
    }

    #[inline]
    pub fn get_grid_height(&self) -> i32 {
        self.grid_height
    }

    #[inline]
    pub fn get_grid_width(&self) -> i32 {
        self.grid_width
    }

    #[inline]
    pub fn get_keyboard_height(&self) -> i32 {
        self.keyboard_height
    }

    #[inline]
    pub fn get_keyboard_width(&self) -> i32 {
        self.keyboard_width
    }

    #[inline]
    pub fn get_keyboard_hypotenuse(&self) -> f32 {
        self.keyboard_hypotenuse
    }

    #[inline]
    pub fn get_key_key_distance_g(&self, key_id0: usize, key_id1: usize) -> i32 {
        self.key_key_distances_g[key_id0][key_id1]
    }
}

fn jint_array_region_to_vec(env: JNIEnv, arr: jintArray, len: usize) -> Vec<i32> {
    let mut buf = vec![0_i32; len];
    let buf = buf.as_mut_slice();
    env.get_int_array_region(arr, 0, buf)
        .expect_droid(&env, "Couldn't get int array region");
    buf.iter().map(|&v| v as i32).collect()
}

fn jfloat_array_region_to_vec(env: JNIEnv, arr: jintArray, len: usize) -> Vec<f32> {
    let mut buf = vec![0 as jfloat; len];
    let buf = buf.as_mut_slice();
    env.get_float_array_region(arr, 0, buf)
        .expect_droid(&env, "Couldn't get int array region");
    buf.iter().map(|&v| v as f32).collect()
}

fn jint_array_region_to_char_vec(env: JNIEnv, arr: jintArray, len: usize) -> Vec<char> {
    let mut buf = vec![0_i32; len];
    let buf = buf.as_mut_slice();
    env.get_int_array_region(arr, 0, buf)
        .expect_droid(&env, "Couldn't get int array region");
    // TODO: expect_droid for options
    buf.iter()
        .map(|&v| char::from_u32(v as u32).unwrap())
        .collect()
}

#[inline]
fn get_start_index_from_coordinates(
    x: i32,
    y: i32,
    cell_height: i32,
    cell_width: i32,
    grid_width: i32,
) -> usize {
    (((y / cell_height) * grid_width + (x / cell_width)) * MAX_PROXIMITY_CHARS_SIZE as i32) as usize
}
