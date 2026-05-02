use core::ptr;

const ERROR_OUT_OF_RANGE: i32 = -882;

#[no_mangle]
pub extern "C" fn oboe_rust_fifo_full_frames_available(
    capacity_in_frames: u32,
    read_counter: u64,
    write_counter: u64,
) -> u32 {
    if read_counter > write_counter {
        return 0;
    }
    let delta = write_counter - read_counter;
    if delta >= capacity_in_frames as u64 {
        capacity_in_frames
    } else {
        delta as u32
    }
}

#[no_mangle]
pub extern "C" fn oboe_rust_fifo_empty_frames_available(
    capacity_in_frames: u32,
    read_counter: u64,
    write_counter: u64,
) -> u32 {
    capacity_in_frames.saturating_sub(oboe_rust_fifo_full_frames_available(
        capacity_in_frames,
        read_counter,
        write_counter,
    ))
}

#[no_mangle]
pub extern "C" fn oboe_rust_fifo_read_index(capacity_in_frames: u32, read_counter: u64) -> u32 {
    if capacity_in_frames == 0 {
        0
    } else {
        (read_counter % capacity_in_frames as u64) as u32
    }
}

#[no_mangle]
pub extern "C" fn oboe_rust_fifo_write_index(capacity_in_frames: u32, write_counter: u64) -> u32 {
    if capacity_in_frames == 0 {
        0
    } else {
        (write_counter % capacity_in_frames as u64) as u32
    }
}

#[no_mangle]
/// # Safety
///
/// `storage` must be valid for `bytes_per_frame * capacity_in_frames` bytes and `destination`
/// must be valid for `frames_to_read * bytes_per_frame` bytes when `frames_to_read` is positive.
pub unsafe extern "C" fn oboe_rust_fifo_copy_read(
    storage: *const u8,
    bytes_per_frame: u32,
    capacity_in_frames: u32,
    read_counter: u64,
    write_counter: u64,
    destination: *mut u8,
    frames_to_read: i32,
) -> i32 {
    if frames_to_read <= 0 {
        return 0;
    }
    if storage.is_null() || destination.is_null() || bytes_per_frame == 0 || capacity_in_frames == 0
    {
        return 0;
    }

    let available =
        oboe_rust_fifo_full_frames_available(capacity_in_frames, read_counter, write_counter);
    let frames = (frames_to_read as u32).min(available);
    let read_index = oboe_rust_fifo_read_index(capacity_in_frames, read_counter);
    match copy_from_ring(
        storage,
        destination,
        bytes_per_frame,
        capacity_in_frames,
        read_index,
        frames,
    ) {
        Ok(()) => frames as i32,
        Err(error) => error,
    }
}

#[no_mangle]
/// # Safety
///
/// `storage` must be valid for `bytes_per_frame * capacity_in_frames` bytes and `source`
/// must be valid for `frames_to_write * bytes_per_frame` bytes when `frames_to_write` is positive.
pub unsafe extern "C" fn oboe_rust_fifo_copy_write(
    storage: *mut u8,
    bytes_per_frame: u32,
    capacity_in_frames: u32,
    read_counter: u64,
    write_counter: u64,
    source: *const u8,
    frames_to_write: i32,
) -> i32 {
    if frames_to_write <= 0 {
        return 0;
    }
    if storage.is_null() || source.is_null() || bytes_per_frame == 0 || capacity_in_frames == 0 {
        return 0;
    }

    let available =
        oboe_rust_fifo_empty_frames_available(capacity_in_frames, read_counter, write_counter);
    let frames = (frames_to_write as u32).min(available);
    let write_index = oboe_rust_fifo_write_index(capacity_in_frames, write_counter);
    match copy_to_ring(
        storage,
        source,
        bytes_per_frame,
        capacity_in_frames,
        write_index,
        frames,
    ) {
        Ok(()) => frames as i32,
        Err(error) => error,
    }
}

#[no_mangle]
/// # Safety
///
/// `storage` must be valid for `bytes_per_frame * capacity_in_frames` bytes and `destination`
/// must be valid for `num_frames * bytes_per_frame` bytes when `num_frames` is positive.
pub unsafe extern "C" fn oboe_rust_fifo_copy_read_now(
    storage: *const u8,
    bytes_per_frame: u32,
    capacity_in_frames: u32,
    read_counter: u64,
    write_counter: u64,
    destination: *mut u8,
    num_frames: i32,
) -> i32 {
    let frames_read = oboe_rust_fifo_copy_read(
        storage,
        bytes_per_frame,
        capacity_in_frames,
        read_counter,
        write_counter,
        destination,
        num_frames,
    );
    if frames_read < 0 || destination.is_null() || bytes_per_frame == 0 || num_frames <= 0 {
        return frames_read;
    }

    let frames_left = num_frames - frames_read;
    if frames_left > 0 {
        let offset = match frames_to_bytes(frames_read as u32, bytes_per_frame) {
            Ok(bytes) => bytes,
            Err(error) => return error,
        };
        let bytes_to_zero = match frames_to_bytes(frames_left as u32, bytes_per_frame) {
            Ok(bytes) => bytes,
            Err(error) => return error,
        };
        ptr::write_bytes(destination.add(offset), 0, bytes_to_zero);
    }
    frames_read
}

unsafe fn copy_from_ring(
    storage: *const u8,
    destination: *mut u8,
    bytes_per_frame: u32,
    capacity_in_frames: u32,
    start_index: u32,
    frames: u32,
) -> Result<(), i32> {
    let first_frames = frames.min(capacity_in_frames - start_index);
    let second_frames = frames - first_frames;
    let start_byte = frames_to_bytes(start_index, bytes_per_frame)?;
    let first_bytes = frames_to_bytes(first_frames, bytes_per_frame)?;
    ptr::copy_nonoverlapping(storage.add(start_byte), destination, first_bytes);

    if second_frames > 0 {
        let second_bytes = frames_to_bytes(second_frames, bytes_per_frame)?;
        ptr::copy_nonoverlapping(storage, destination.add(first_bytes), second_bytes);
    }
    Ok(())
}

unsafe fn copy_to_ring(
    storage: *mut u8,
    source: *const u8,
    bytes_per_frame: u32,
    capacity_in_frames: u32,
    start_index: u32,
    frames: u32,
) -> Result<(), i32> {
    let first_frames = frames.min(capacity_in_frames - start_index);
    let second_frames = frames - first_frames;
    let start_byte = frames_to_bytes(start_index, bytes_per_frame)?;
    let first_bytes = frames_to_bytes(first_frames, bytes_per_frame)?;
    ptr::copy_nonoverlapping(source, storage.add(start_byte), first_bytes);

    if second_frames > 0 {
        let second_bytes = frames_to_bytes(second_frames, bytes_per_frame)?;
        ptr::copy_nonoverlapping(source.add(first_bytes), storage, second_bytes);
    }
    Ok(())
}

fn frames_to_bytes(frames: u32, bytes_per_frame: u32) -> Result<usize, i32> {
    let bytes = frames
        .checked_mul(bytes_per_frame)
        .ok_or(ERROR_OUT_OF_RANGE)?;
    if bytes > i32::MAX as u32 {
        return Err(ERROR_OUT_OF_RANGE);
    }
    Ok(bytes as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clips_full_frames_to_capacity() {
        assert_eq!(oboe_rust_fifo_full_frames_available(8, 3, 7), 4);
        assert_eq!(oboe_rust_fifo_full_frames_available(8, 3, 99), 8);
        assert_eq!(oboe_rust_fifo_full_frames_available(8, 9, 7), 0);
        assert_eq!(oboe_rust_fifo_empty_frames_available(8, 3, 7), 4);
    }

    #[test]
    fn wraps_read_and_write_indices() {
        assert_eq!(oboe_rust_fifo_read_index(8, 10), 2);
        assert_eq!(oboe_rust_fifo_write_index(8, 17), 1);
    }

    #[test]
    fn writes_and_reads_wrapped_frames() {
        let mut storage = [0u8; 8];
        let source = [1u8, 2, 3, 4, 5, 6];
        let written = unsafe {
            oboe_rust_fifo_copy_write(storage.as_mut_ptr(), 1, 8, 0, 6, source.as_ptr(), 6)
        };
        assert_eq!(written, 2);
        assert_eq!(&storage[6..8], &[1, 2]);

        let mut output = [0u8; 4];
        let read = unsafe {
            oboe_rust_fifo_copy_read(storage.as_ptr(), 1, 8, 6, 8, output.as_mut_ptr(), 4)
        };
        assert_eq!(read, 2);
        assert_eq!(&output, &[1, 2, 0, 0]);
    }

    #[test]
    fn read_now_zero_fills_unavailable_frames() {
        let storage = [9u8, 8, 7, 6];
        let mut output = [1u8; 6];
        let read = unsafe {
            oboe_rust_fifo_copy_read_now(storage.as_ptr(), 1, 4, 0, 2, output.as_mut_ptr(), 6)
        };
        assert_eq!(read, 2);
        assert_eq!(&output, &[9, 8, 0, 0, 0, 0]);
    }
}
