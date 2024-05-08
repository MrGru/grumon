use bevy::prelude::*;

use crate::components::ysort::YSort;

pub fn y_sort(mut query: Query<(&mut Transform, &YSort)>) {
    for (mut tf, ysort) in query.iter_mut() {
        tf.translation.z = ysort.z - (1.0f32 / (1.0f32 + 2.0f32.powf(-0.01 * tf.translation.y)));
    }
}
