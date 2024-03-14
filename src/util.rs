pub fn smootherstep(t: f32) -> f32 {
	t * t * t * (t * (6.0 * t - 15.0) + 10.0)
}
