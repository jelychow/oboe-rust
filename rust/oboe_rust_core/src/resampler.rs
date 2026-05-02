const PRIMES: &[i32] = &[
    2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97,
    101, 103, 107, 109, 113, 127, 131, 137, 139, 149, 151, 157, 163, 167, 173, 179, 181, 191, 193,
    197, 199,
];

#[no_mangle]
/// # Safety
///
/// `numerator` and `denominator` must point to valid `i32` values.
pub unsafe extern "C" fn oboe_rust_integer_ratio_reduce(
    numerator: *mut i32,
    denominator: *mut i32,
) {
    if numerator.is_null() || denominator.is_null() {
        return;
    }

    let mut top_value = *numerator;
    let mut bottom_value = *denominator;
    for &prime in PRIMES {
        if top_value < prime || bottom_value < prime {
            break;
        }

        loop {
            let top = top_value / prime;
            let bottom = bottom_value / prime;
            if top >= 1
                && bottom >= 1
                && top.saturating_mul(prime) == top_value
                && bottom.saturating_mul(prime) == bottom_value
            {
                top_value = top;
                bottom_value = bottom;
            } else {
                break;
            }
        }
    }

    *numerator = top_value;
    *denominator = bottom_value;
}

#[no_mangle]
/// # Safety
///
/// `x` must be valid for `num_taps * channel_count * 2` samples, `frame` must be valid for
/// `channel_count` samples, and `cursor` must point to a valid cursor value.
pub unsafe extern "C" fn oboe_rust_resampler_write_frame(
    x: *mut f32,
    frame: *const f32,
    num_taps: i32,
    channel_count: i32,
    cursor: *mut i32,
) {
    if x.is_null() || frame.is_null() || cursor.is_null() || num_taps <= 0 || channel_count <= 0 {
        return;
    }

    let mut next_cursor = *cursor - 1;
    if next_cursor < 0 {
        next_cursor = num_taps - 1;
    }
    *cursor = next_cursor;

    let channel_count = channel_count as usize;
    let destination = x.add(next_cursor as usize * channel_count);
    let duplicate_offset = num_taps as usize * channel_count;
    for channel in 0..channel_count {
        let sample = *frame.add(channel);
        *destination.add(channel) = sample;
        *destination.add(channel + duplicate_offset) = sample;
    }
}

#[no_mangle]
/// # Safety
///
/// `previous`, `current`, and `destination` must be valid for `channel_count` samples.
pub unsafe extern "C" fn oboe_rust_linear_resampler_read_frame(
    previous: *const f32,
    current: *const f32,
    destination: *mut f32,
    channel_count: i32,
    integer_phase: i32,
    denominator: i32,
) {
    if previous.is_null()
        || current.is_null()
        || destination.is_null()
        || channel_count <= 0
        || denominator == 0
    {
        return;
    }

    let phase = integer_phase as f32 / denominator as f32;
    for channel in 0..channel_count as usize {
        let f0 = *previous.add(channel);
        let f1 = *current.add(channel);
        *destination.add(channel) = f0 + (phase * (f1 - f0));
    }
}

#[no_mangle]
/// # Safety
///
/// `x` must be valid for at least `(cursor + num_taps) * channel_count` samples, `coefficients`
/// for `num_taps` samples, and `destination` for `channel_count` samples.
pub unsafe extern "C" fn oboe_rust_polyphase_resampler_read_frame(
    x: *const f32,
    coefficients: *const f32,
    destination: *mut f32,
    num_taps: i32,
    channel_count: i32,
    cursor: i32,
) {
    if x.is_null()
        || coefficients.is_null()
        || destination.is_null()
        || num_taps <= 0
        || channel_count <= 0
        || cursor < 0
    {
        return;
    }

    for channel in 0..channel_count as usize {
        *destination.add(channel) = 0.0;
    }

    let channel_count = channel_count as usize;
    let mut sample_index = cursor as usize * channel_count;
    for tap in 0..num_taps as usize {
        let coefficient = *coefficients.add(tap);
        for channel in 0..channel_count {
            let accumulated = *destination.add(channel) + (*x.add(sample_index) * coefficient);
            *destination.add(channel) = accumulated;
            sample_index += 1;
        }
    }
}

#[no_mangle]
/// # Safety
///
/// `x` must be valid for `num_taps * channel_count` samples, coefficient rows for `num_taps`
/// samples each, and `destination` for `channel_count` samples.
pub unsafe extern "C" fn oboe_rust_sinc_resampler_read_frame(
    x: *const f32,
    coefficients_low: *const f32,
    coefficients_high: *const f32,
    destination: *mut f32,
    num_taps: i32,
    channel_count: i32,
    fraction: f32,
) {
    if x.is_null()
        || coefficients_low.is_null()
        || coefficients_high.is_null()
        || destination.is_null()
        || num_taps <= 0
        || channel_count <= 0
    {
        return;
    }

    let channel_count = channel_count as usize;
    for channel in 0..channel_count {
        let mut low = 0.0;
        let mut high = 0.0;
        for tap in 0..num_taps as usize {
            let sample = *x.add(tap * channel_count + channel);
            low += sample * *coefficients_low.add(tap);
            high += sample * *coefficients_high.add(tap);
        }
        *destination.add(channel) = low + (fraction * (high - low));
    }
}
