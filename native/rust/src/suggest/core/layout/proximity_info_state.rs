use std::collections::HashMap;

use indexmap::IndexMap;

use crate::defines::DoubleLetterLevel;
use crate::math::{get_angle, get_angle_diff, get_distance_int};
use crate::suggest::core::layout::proximity_info::{MAX_PROXIMITY_CHARS_SIZE, ProximityInfo};
use crate::suggest::core::layout::proximity_info_params::{
    CORNER_ANGLE_THRESHOLD_FOR_POINT_SCORE, CORNER_CHECK_DISTANCE_THRESHOLD_SCALE, CORNER_SCORE,
    CORNER_SUM_ANGLE_THRESHOLD, DISTANCE_BASE_SCALE, LAST_POINT_SKIP_DISTANCE_SCALE,
    LOCALMIN_DISTANCE_AND_NEAR_TO_KEY_SCORE, MARGIN_FOR_PREV_LOCAL_MIN,
    MIN_DOUBLE_LETTER_BEELINE_SPEED_PERCENTILE, NEAR_KEY_THRESHOLD_FOR_DISTANCE,
    NEAR_KEY_THRESHOLD_FOR_POINT_SCORE, NOT_LOCALMIN_DISTANCE_SCORE,
};

type NearKeysDistanceMap = IndexMap<usize, f32>;

pub struct ProximityInfoState {
    // TODO: make reference?
    proximity_info: ProximityInfo,
    max_point_to_key_length: f32,
    average_speed: f32,
    is_continuous_suggestion_possible: bool,
    has_been_updated_by_geometric_input: bool,
    sampled_input_xs: Vec<i32>,
    sampled_input_ys: Vec<i32>,
    sampled_times: Vec<i32>,
    sampled_input_indice: Vec<usize>,
    sampled_length_cache: Vec<i32>,
    beeline_speed_percentiles: Vec<i32>,
    sampled_normalized_squared_length_cache: Vec<f32>,
    speed_rates: Vec<f32>,
    directions: Vec<f32>,
    // probabilities of skipping or mapping to a key for each point
    // TODO: i dont think thats the right types
    char_probabilities: Vec<HashMap<i32, f32>>,
    // The vector for the key code set which holds nearby keys of some trailing sampled input points
    // for each sampled input point. These nearby keys contain the next characters which can be in
    // the dictionary. Specifically, currently we are looking for keys nearby trailing sampled
    // inputs including the current input point.
    // TODO: cpp uses a named bitset here
    sampled_search_key_sets: Vec<Vec<bool>>,
    sampled_search_key_vectors: Vec<Vec<i32>>,
    touch_position_correction_enabled: bool,
    // TODO: sized as MAX_PROXIMITY_CHARS_SIZE * MAX_WORD_LENGTH in cpp
    input_proximities: Vec<Vec<char>>,
    sampled_input_size: usize,
    // TODO: sized as MAX_WORD_LENGTH in cpp
    // TODO: make slice of chars?
    primary_input_word: String,
    most_probable_string_probability: f32,
    // TODO: sized as MAX_WORD_LENGTH in cpp
    // TODO: make slice of chars?
    most_probable_string: String,
}

impl ProximityInfoState {
    // TODO: Remove the dependency of "is_geometric"
    pub fn init_input_params(
        &mut self,
        pointer_id: usize,
        max_point_to_key_length: f32,
        proximity_info: ProximityInfo,
        input_codes: Vec<char>,
        input_size: usize,
        x_coordinates: Vec<i32>,
        y_coordinates: Vec<i32>,
        times: Option<Vec<i32>>,
        pointer_ids: Vec<usize>,
        is_geometric: bool,
        locale: String,
    ) {
        self.is_continuous_suggestion_possible = self.has_been_updated_by_geometric_input
            == is_geometric
            && self.check_and_return_is_continuos_suggestion_possible(
                &input_size,
                &x_coordinates,
                &y_coordinates,
                times,
            );

        self.proximity_info = proximity_info;

        self.input_proximities.clear();
        if !is_geometric && pointer_id == 0 {
            self.input_proximities = self.proximity_info.initialize_proximities(
                &input_codes,
                &x_coordinates,
                &y_coordinates,
                &input_size,
                &locale,
            );
        }

        // setup touch points
        let mut push_touch_point_start_index = 0;
        let mut last_saved_input_size = 0;
        self.max_point_to_key_length = max_point_to_key_length;
        self.sampled_input_size = 0;
        self.most_probable_string_probability = 0.0;

        if self.is_continuous_suggestion_possible && !self.sampled_input_indice.is_empty() {
            // Just update difference.
            // Previous two points are never skipped. Thus, we pop 2 input point data here.
            push_touch_point_start_index = self.trim_last_two_touch_points();
            last_saved_input_size = self.sampled_input_xs.len();
        } else {
            // Clear all data
            self.sampled_input_xs.clear();
            self.sampled_input_ys.clear();
            self.sampled_times.clear();
            self.sampled_input_indice.clear();
            self.sampled_length_cache.clear();
            self.sampled_normalized_squared_length_cache.clear();
            self.sampled_search_key_sets.clear();
            self.speed_rates.clear();
            self.beeline_speed_percentiles.clear();
            self.char_probabilities.clear();
            self.directions.clear();
        }
        // TODO: we assume that we always have x coordinates and y coordinates, the cpp code doesnt
        //self.sampled_input_size =
    }

    #[inline]
    pub fn get_primary_code_point_at(&self, index: usize) -> char {
        self.get_proximity_code_points_at(index)[0]
    }

    pub fn get_primary_original_code_point_at(&self, index: usize) -> Option<char> {
        let primary_code_point = self.get_primary_code_point_at(index);
        if let Some(key_index) = self.proximity_info.get_key_index_of(primary_code_point) {
            return self.proximity_info.get_original_code_point_of(key_index);
        }
        None
    }

    #[inline]
    fn get_proximity_code_points_at(&self, index: usize) -> &Vec<char> {
        &self.input_proximities[index]
    }

    // TODO: this logic is probably completely wrong
    pub fn same_as_typed(&self, word: String, length: usize) -> bool {
        if length != self.sampled_input_size {
            return false;
        }
        let word_vec: Vec<char> = word.chars().collect();
        for i in 0..length {
            if word_vec[i] != self.input_proximities[i][0] {
                return false;
            }
        }
        true
    }

    pub fn exists_code_point_in_proximity_at(&self, index: usize, c: char) -> bool {
        let code_points = self.get_proximity_code_points_at(index);
        code_points.contains(&c)
    }

    pub fn exists_adjacent_proximity_chars(&self, index: usize) -> bool {
        if index > self.sampled_input_size {
            return false;
        }
        let current_code_point = self.get_primary_code_point_at(index);
        let left_index = index - 1;
        if left_index > 0 && self.exists_code_point_in_proximity_at(left_index, current_code_point)
        {
            return true;
        }
        let right_index = index + 1;
        if right_index < self.sampled_input_size
            && self.exists_code_point_in_proximity_at(right_index, current_code_point)
        {
            return true;
        }
        false
    }

    #[inline]
    pub fn touch_positon_correction_enabled(&self) -> bool {
        self.touch_position_correction_enabled
    }

    #[inline]
    pub fn is_used(&self) -> bool {
        self.sampled_input_size > 0
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.sampled_input_size
    }

    #[inline]
    pub fn get_input_x(&self, index: usize) -> i32 {
        self.sampled_input_xs[index]
    }

    #[inline]
    pub fn get_input_y(&self, index: usize) -> i32 {
        self.sampled_input_ys[index]
    }

    #[inline]
    pub fn get_input_index_of_sampled_point(&self, sampled_index: usize) -> usize {
        self.sampled_input_indice[sampled_index]
    }

    #[inline]
    pub fn has_space_proximity(&self, index: usize) -> bool {
        self.proximity_info
            .has_space_proximity(self.get_input_x(index), self.get_input_y(index))
    }

    pub fn get_length_cache(&self, index: usize) -> i32 {
        self.sampled_length_cache[index]
    }

    pub fn is_continuous_suggestion_possible(&self) -> bool {
        self.is_continuous_suggestion_possible
    }

    pub fn get_speed_rate(&self, index: usize) -> f32 {
        self.speed_rates[index]
    }

    pub fn get_beeline_speed_percentile(&self, id: usize) -> i32 {
        self.beeline_speed_percentiles[id]
    }

    pub fn get_double_letter_level(&self, id: usize) -> DoubleLetterLevel {
        let beeline_speed_rate = self.get_beeline_speed_percentile(id);
        if beeline_speed_rate == 0 {
            DoubleLetterLevel::Strong
        } else if beeline_speed_rate < MIN_DOUBLE_LETTER_BEELINE_SPEED_PERCENTILE {
            DoubleLetterLevel::DoubleLetter
        } else {
            DoubleLetterLevel::None
        }
    }

    #[inline]
    pub fn get_direction(&self, index: usize) -> f32 {
        self.directions[index]
    }

    pub fn get_xy_direction(&self, index0: usize, index1: usize) -> f32 {
        if index0 > self.sampled_input_size - 1 {
            return 0.0;
        }
        if index1 > self.sampled_input_size - 1 {
            return 0.0;
        }

        get_angle(
            self.sampled_input_xs[index0],
            self.sampled_input_ys[index0],
            self.sampled_input_xs[index1],
            self.sampled_input_ys[index1],
        )
    }

    fn check_and_return_is_continuos_suggestion_possible(
        &self,
        input_size: &usize,
        x_coordinates: &Vec<i32>,
        y_coordinates: &Vec<i32>,
        times: Option<Vec<i32>>,
    ) -> bool {
        if input_size < &self.sampled_input_size {
            return false;
        }
        for i in 0..self.sampled_input_size {
            let index = self.sampled_input_indice[i];
            if index >= *input_size {
                return false;
            }
            if x_coordinates[index] != self.sampled_input_xs[i]
                || y_coordinates[index] != self.sampled_input_ys[i]
            {
                return false;
            }
            if let Some(t) = &times {
                if t[index] != self.sampled_times[i] {
                    return false;
                }
            }
        }
        true
    }

    fn trim_last_two_touch_points(&mut self) -> usize {
        let next_start_index = self.sampled_input_indice[self.sampled_input_indice.len() - 2];
        self.pop_input_data();
        self.pop_input_data();
        next_start_index
    }

    fn pop_input_data(&mut self) {
        self.sampled_input_xs.pop();
        self.sampled_input_ys.pop();
        self.sampled_times.pop();
        self.sampled_length_cache.pop();
        self.sampled_input_indice.pop();
    }

    fn update_touch_points(
        &mut self,
        x_coordinates: Vec<i32>,
        y_coordinates: Vec<i32>,
        times: Vec<i32>,
        // TODO: this appears to be assumed nullable in cpp, for now we require it
        pointer_ids: Vec<usize>,
        input_size: usize,
        is_geometric: bool,
        pointer_id: usize,
        push_touch_point_start_index: usize,
    ) -> usize {
        // TODO: make coordinates optional u32 or similar instead of using negative value as a state
        let proximity_only = !is_geometric && (x_coordinates[0] < 0 || y_coordinates[0] < 0);
        let mut last_input_index = push_touch_point_start_index;
        for i in last_input_index..input_size {
            if pointer_id == pointer_ids[i] {
                // TODO: break?
                last_input_index = i;
            }
        }
        // Working space to save near keys distances for current, prev and prevprev input point.
        let mut current_near_keys_distances: NearKeysDistanceMap = NearKeysDistanceMap::new();
        let mut prev_near_keys_distances: NearKeysDistanceMap = NearKeysDistanceMap::new();
        let mut prev_prev_near_keys_distances: NearKeysDistanceMap = NearKeysDistanceMap::new();

        // "sum_angle" is accumulated by each angle of input points. And when "sum_angle" exceeds
        // the threshold we save that point, reset sum_angle. This aims to keep the figure of
        // the curve.
        let mut sum_angle = 0_f32;

        for i in push_touch_point_start_index..last_input_index {
            if pointer_id == pointer_ids[i] {
                let c = if is_geometric {
                    None
                } else {
                    Some(self.get_primary_code_point_at(i))
                };
                let (x, y) = if proximity_only {
                    (-1, -1)
                } else {
                    (x_coordinates[i], y_coordinates[i])
                };
                // TODO: times appears to be optional in cpp impl, for now pretend it isnt
                let time = times[i];

                if i > 1 {
                    // TODO: this literally does not make sense with the assumption that x and y can be -1 ???????
                    let prev_angle = get_angle(
                        x_coordinates[i - 2],
                        y_coordinates[i - 2],
                        x_coordinates[i - 1],
                        y_coordinates[i - 1],
                    );
                    let current_angle = get_angle(x_coordinates[i - 1], y_coordinates[i - 1], x, y);
                    sum_angle += get_angle_diff(prev_angle, current_angle);
                }
            }
        }

        self.sampled_input_xs.len()
    }

    fn push_touch_point(
        &mut self,
        input_index: usize,
        node_code_point: Option<char>,
        x: i32,
        y: i32,
        time: i32,
        is_geometric: bool,
        do_sampling: bool,
        is_last_point: bool,
        sum_angle: f32,
        current_near_keys_distances: &mut NearKeysDistanceMap,
        prev_near_keys_distances: &mut NearKeysDistanceMap,
        prev_prev_near_keys_distances: &mut NearKeysDistanceMap,
    ) -> bool {
        let mut size = self.sampled_input_xs.len();
        let mut popped = false;

        if node_code_point.is_none() && do_sampling {
            let nearest =
                self.update_near_keys_distances(x, y, is_geometric, current_near_keys_distances);
            let score = self.get_point_score(
                x,
                y,
                nearest,
                sum_angle,
                current_near_keys_distances,
                prev_near_keys_distances,
                prev_prev_near_keys_distances,
            );
            popped = if score < 0.0 {
                // Pop previous point because it would be useless
                self.pop_input_data();
                size = self.sampled_input_xs.len();
                true
            } else {
                false
            };

            // Check if the last point should be skipped
            if is_last_point
                && size > 0
                && get_distance_int(
                    x,
                    y,
                    self.sampled_input_xs[size - 1],
                    self.sampled_input_ys[size - 1],
                ) * LAST_POINT_SKIP_DISTANCE_SCALE
                    < self.proximity_info.get_most_common_key_width()
            {
                // This point is not used because it's too close to the previous point.
                return popped;
            }
        }

        // TODO: this is ugly as hell lmao
        let (x, y) = if node_code_point.is_some() && (x < 0 || y < 0) {
            let key_id = self
                .proximity_info
                .get_key_index_of(node_code_point.unwrap());
            if let Some(key_id) = key_id {
                (
                    self.proximity_info
                        .get_key_center_x_of_key_id_g(key_id, None, is_geometric),
                    self.proximity_info
                        .get_key_center_y_of_key_id_g(key_id, None, is_geometric),
                )
            } else {
                (x, y)
            }
        } else {
            (x, y)
        };

        // Pushing point information
        if size > 0 {
            self.sampled_length_cache.push(
                self.sampled_length_cache.last().unwrap() + get_distance_int(
                    x,
                    y,
                    *self.sampled_input_xs.last().unwrap(),
                    *self.sampled_input_ys.last().unwrap()
                )
            );
        } else {
            self.sampled_length_cache.push(0);
        }
        self.sampled_input_xs.push(x);
        self.sampled_input_ys.push(y);
        self.sampled_times.push(time);
        self.sampled_input_indice.push(input_index);

        popped
    }

    // Calculating point to key distance for all near keys and returning the distance between
    // the given point and the nearest key position.
    fn update_near_keys_distances(
        &self,
        x: i32,
        y: i32,
        is_geometric: bool,
        current_near_keys_distances: &mut NearKeysDistanceMap,
    ) -> f32 {
        current_near_keys_distances.clear();
        let mut nearest_key_distance = self.max_point_to_key_length;
        for k in 0..self.proximity_info.get_key_count() {
            let dist = self
                .proximity_info
                .get_normalized_square_distance_from_center_float_g(k, x, y, is_geometric);
            if dist < NEAR_KEY_THRESHOLD_FOR_DISTANCE {
                current_near_keys_distances.insert(k, dist);
            }
            if nearest_key_distance > dist {
                nearest_key_distance = dist;
            }
        }
        nearest_key_distance
    }

    // Calculating a point score that indicates usefulness of the point.
    fn get_point_score(
        &self,
        x: i32,
        y: i32,
        nearest: f32,
        sum_angle: f32,
        current_near_keys_distances: &NearKeysDistanceMap,
        prev_near_keys_distances: &NearKeysDistanceMap,
        prev_prev_near_keys_distances: &NearKeysDistanceMap,
    ) -> f32 {
        let size = self.sampled_input_xs.len();
        // If there is only one point, add this point. Besides, if the previous point's distance map
        // is empty, we re-compute nearby keys distances from the current point.
        // Note that the current point is the first point in the incremental input that needs to
        // be re-computed.
        if size <= 1 || prev_near_keys_distances.is_empty() {
            return 0.0;
        }

        let base_sample_rate = self.proximity_info.get_most_common_key_width();
        let dist_prev = get_distance_int(
            self.sampled_input_xs[size - 1],
            self.sampled_input_ys[size - 1],
            self.sampled_input_xs[size - 2],
            self.sampled_input_ys[size - 2],
        ) * DISTANCE_BASE_SCALE;

        let mut score = 0_f32;

        // Location
        if !self.is_prev_local_min(
            current_near_keys_distances,
            prev_near_keys_distances,
            prev_prev_near_keys_distances,
        ) {
            score += NOT_LOCALMIN_DISTANCE_SCORE;
        } else if nearest < NEAR_KEY_THRESHOLD_FOR_POINT_SCORE {
            // Promote points nearby keys
            score += LOCALMIN_DISTANCE_AND_NEAR_TO_KEY_SCORE;
        }

        // Angle
        let angle1 = get_angle(
            x,
            y,
            self.sampled_input_xs[size - 1],
            self.sampled_input_ys[size - 1],
        );
        let angle2 = get_angle(
            self.sampled_input_xs[size - 1],
            self.sampled_input_ys[size - 1],
            self.sampled_input_xs[size - 2],
            self.sampled_input_ys[size - 2],
        );
        let angle_diff = get_angle_diff(angle1, angle2);

        // Save corner
        if dist_prev > base_sample_rate * CORNER_CHECK_DISTANCE_THRESHOLD_SCALE
            && (sum_angle > CORNER_SUM_ANGLE_THRESHOLD
                || angle_diff > CORNER_ANGLE_THRESHOLD_FOR_POINT_SCORE)
        {
            score += CORNER_SCORE;
        }

        score
    }

    fn is_prev_local_min(
        &self,
        current_near_keys_distances: &NearKeysDistanceMap,
        prev_near_keys_distances: &NearKeysDistanceMap,
        prev_prev_near_keys_distances: &NearKeysDistanceMap,
    ) -> bool {
        for (k, d) in prev_near_keys_distances {
            let prev_prev_d = prev_prev_near_keys_distances[k];
            let prev_prev_index = prev_prev_near_keys_distances.get_index_of(k).unwrap();
            let current_d = current_near_keys_distances[k];
            let current_index = current_near_keys_distances.get_index_of(k).unwrap();

            // TODO: is this correct?
            let is_prev_prev_near = prev_prev_index == prev_prev_near_keys_distances.len() - 1
                || prev_prev_d > d + MARGIN_FOR_PREV_LOCAL_MIN;
            let is_current_near = current_index == current_near_keys_distances.len() - 1
                || current_d > d + MARGIN_FOR_PREV_LOCAL_MIN;
            if is_prev_prev_near && is_current_near {
                return true;
            }
        }
        false
    }
}
