mod wave;

use std::fs::File;
use std::io::Read;
use std::rc::Rc;
use std::slice;
use log::__private_api::GlobalLogger;
use log::{info, log, Level, LevelFilter};
use oslog::OsLogger;
use crate::wave::{load_bytes_rmaf, Wave};

/// Returns a wave loaded from a file in an FFI compatible format.
///
/// **Note:** The memory has to be freed after use to prevent a leak.
/// You might want to use [free_wave_samples_unsafe] for handling this.
#[cfg(all(feature = "rmaf", feature = "c-compatible"))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn load_rmaf_file_unsafe(file_name: *const u8, file_name_length: usize, section_number: *mut u32, sample_rate: *mut f32, samples_start_address: *mut *const f32, samples_length: *mut usize) {

    setup_logger();

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        info!("rcaudio: File name that has been passed to function is located at: {:#X?} and has a length of: {} bytes.", &file_name, file_name_length);

        let file_name: &[u8] = unsafe { slice::from_raw_parts(file_name, file_name_length) };
        let file_name = String::from_utf8_lossy(file_name);
        let file_name = file_name.as_ref();
        let wave = load_rmaf_file(file_name).unwrap();

        let samples = wave.samples;

        unsafe {
            *section_number = wave.section_index;
            *sample_rate = wave.sample_rate;
            *samples_start_address = samples.as_ptr();
            *samples_length = samples.len();
        }

        std::mem::forget(samples)
    }));

    if result.is_err() {
        info!("rcaudio: Failed to load RMAF file in c-compatible function")
    } else {
        info!("rcaudio: Successfully loaded RMAF file in c-compatible function")
    }

}

#[cfg(feature = "c-compatible")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn free_wave_samples_unsafe(samples_start_address: *const f32, samples_length: usize) {
    setup_logger();
    info!("rcaudio: Freeing wave samples from {:#X?}, length {}", samples_start_address, samples_length);

    /*std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = Rc::from(unsafe { slice::from_raw_parts(samples_start_address, samples_length) as *const [f32] });
    }));*/

}

#[cfg(feature = "rmaf")]
pub fn load_rmaf_file(file_name: &str) -> Result<Wave, anyhow::Error> {
    let file = File::open(file_name)?;
    setup_logger();
    info!("rcaudio: Loading RMAF file: {}", file_name);
    load_bytes_rmaf(file.bytes())
}


fn setup_logger() {
    let _ = OsLogger::new("com.txythread")
        .level_filter(LevelFilter::Info)
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "rmaf")]
    #[test]
    pub fn test_load_rmaf_file() {
        setup_logger();
        let wave = load_rmaf_file(concat!(env!("CARGO_MANIFEST_DIR"), "/test_files/RMAF_Test.bin")).unwrap();
        println!("Wave is sampled at: {} and has section: {}", wave.sample_rate, wave.section_index);
    }
}

