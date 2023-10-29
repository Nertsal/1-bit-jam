#![allow(dead_code)]

use geng::prelude::*;

type Area = Aabb2<f32>;

pub fn split_left_right(aabb: Area, left_ratio: f32) -> (Area, Area) {
    let left_width = aabb.width() * left_ratio;
    (
        aabb.extend_right(left_width - aabb.width()),
        aabb.extend_left(-left_width),
    )
}

pub fn split_top_down(aabb: Area, top_ratio: f32) -> (Area, Area) {
    let top_height = aabb.height() * top_ratio;
    (
        aabb.extend_down(top_height - aabb.height()),
        aabb.extend_up(-top_height),
    )
}

pub fn split_rows(aabb: Area, rows: usize) -> Vec<Area> {
    let row_height = aabb.height() / rows as f32;
    (0..rows)
        .map(|i| {
            Area::point(aabb.bottom_left() + vec2(0.0, row_height * i as f32))
                .extend_positive(vec2(aabb.width(), row_height))
        })
        .collect()
}

pub fn split_columns(aabb: Area, columns: usize) -> Vec<Area> {
    let column_width = aabb.width() / columns as f32;
    (0..columns)
        .map(|i| {
            Area::point(aabb.bottom_left() + vec2(column_width * i as f32, 0.0))
                .extend_positive(vec2(column_width, aabb.height()))
        })
        .collect()
}
