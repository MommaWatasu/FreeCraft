use std::f32::consts::PI;

pub fn to_radians(x: f32) -> f32 { x * PI / 180.0 }

pub fn decimal_round(x: f32) -> f32 { (x * 100.0).round() / 100.0 }