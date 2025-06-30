use std::{
    collections::{HashMap, HashSet},
    u8,
};

use crate::colors::{RGB, RGBA};

pub fn get_unique_colors(pixels: &[Vec<RGBA>]) -> Vec<RGBA> {
    let mut colors = HashSet::new();

    for row in pixels {
        for pixel in row {
            colors.insert(pixel.clone());
        }
    }

    colors.into_iter().collect()
}

pub fn create_pallete_from_colors_median_cut(
    unique_colors: &[RGBA],
    number_of_colors_log2: usize,
) -> HashMap<RGBA, (usize, RGBA)> {
    let mut color_buckets: Vec<Vec<RGBA>> = vec![unique_colors.to_vec()];

    if unique_colors.len() > u8::MAX.into() {
        for _i in 0..number_of_colors_log2 {
            color_buckets = color_buckets
                .into_iter()
                .map(median_cut_bucket)
                .flatten()
                .filter(|bucket| !bucket.is_empty())
                .collect();
        }
    }

    color_buckets
        .into_iter()
        .enumerate()
        .map(|(i, bucket)| {
            let avg_color = get_bucket_average_color(&bucket);

            let avg_color_per_color: Vec<(RGBA, (usize, RGBA))> = bucket
                .into_iter()
                .map(|color| (color, (i, avg_color.clone())))
                .collect();

            avg_color_per_color
        })
        .flatten()
        .collect()
}

fn median_cut_bucket(mut bucket: Vec<RGBA>) -> Vec<Vec<RGBA>> {
    let r_range = range_size(bucket.iter().map(|px| px.r).collect());
    let g_range = range_size(bucket.iter().map(|px| px.g).collect());
    let b_range = range_size(bucket.iter().map(|px| px.b).collect());

    if r_range > g_range && r_range > b_range {
        bucket.sort_by_key(|pix| pix.r);
    } else if g_range > b_range {
        bucket.sort_by_key(|pix| pix.g);
    } else {
        bucket.sort_by_key(|pix| pix.b);
    }

    let other_bucket = bucket.split_off(bucket.len() >> 1);

    vec![bucket, other_bucket]
}

fn range_size(values: Vec<u8>) -> u8 {
    let mut max = u8::MIN;
    let mut min = u8::MAX;

    for value in values {
        if value > max {
            max = value;
        }
        if value < min {
            min = value;
        }
    }

    max - min
}

fn get_bucket_average_color(bucket: &[RGBA]) -> RGBA {
    let mut r_sum: f32 = 0.;
    let mut g_sum: f32 = 0.;
    let mut b_sum: f32 = 0.;

    for color in bucket.iter() {
        r_sum += color.r as f32;
        g_sum += color.g as f32;
        b_sum += color.b as f32;
    }

    let r = (r_sum / bucket.len() as f32).round() as u8;
    let g = (g_sum / bucket.len() as f32).round() as u8;
    let b = (b_sum / bucket.len() as f32).round() as u8;

    RGBA::new(r, g, b, u8::MAX)
}

pub fn index_palette(palette: HashMap<RGB, RGB>) -> HashMap<RGB, (RGB, usize)> {
    palette
        .into_iter()
        .enumerate()
        .map(|(i, (k, v))| (k, (v, i)))
        .collect()
}
