pub fn calc_new_rating(ra: f32, rb: f32, sa: u8) -> f32 {
    let ea = get_expected_score(ra, rb);

    let ka = if ra > 1600.0 { 8.0 } else { 16.0 };

    ra + ka * (sa as f32 - ea)
}

fn get_expected_score(ra: f32, rb: f32) -> f32 {
    1.0 / (1.0 + 10.0f32.powf((rb - ra) / 400.0))
}
