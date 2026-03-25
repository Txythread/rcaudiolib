use std::fs::File;
use std::io::Bytes;
use anyhow::anyhow;
use log::info;
use crate::wave::Wave;


#[cfg(feature = "rmaf")]
pub fn load_bytes(raw_data: Bytes<File>) -> Result<Wave, anyhow::Error> {
    let mut rmaf_file_indicator = ".RMAF   ".chars();

    let mut raw_data = raw_data;

    let characters_count = rmaf_file_indicator.clone().count();

    for _ in 0..characters_count {
        if raw_data.next().unwrap().ok() != Some(rmaf_file_indicator.next().unwrap() as u8) {
            return Err(anyhow!("rcaudio: Expected RMAF file but another file type was provided."))
        }
    }

    let version: u32 = convert_bits_32(&mut raw_data, false);

    if version != 0 {
        return Err(anyhow!("rcaudio: RMAF file version unknown. Known versions: 0. Version Found: {version}"));
    }

    let section_index: u32 = convert_bits_32(&mut raw_data, false);

    let sample_rate: f32 = unsafe { std::mem::transmute_copy(&convert_bits_32::<u32>(&mut raw_data, true)) };

    //let header_length = 8 + 4 + 4 + 4 + 4;

    let header_end_code = 0xFF_CC_BB_AAu32;
    let actual_code: u32 = convert_bits_32(&mut raw_data, true);

    if header_end_code != actual_code {
        return Err(anyhow!("rcaudio: RMAF file header seems to have been corrupted. Expected {:#x}, received: {:#x}", header_end_code, actual_code));
    }

    let mut samples: Vec<f32> = Vec::new();

    let mut zeroes_count = 0;

    while let Some(byte1) = raw_data.next() {
        let byte1 = byte1? as u16;

        if let Some(byte2_raw) = raw_data.next() {
            let byte2 = match byte2_raw { Ok(t) => t, _ => break } as u16;

            _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let data = ((byte2 & 0x00FF) << 8) | byte1;



                let sample = f16_to_f32(data);

                if sample == 0f32 { zeroes_count += 1; }

                samples.push(sample);
            }));

        } else {
            break;
        }
    }

    crate::setup_logger();
    info!("rcaudio: {zeroes_count}/{} samples were zeroes.", samples.len());



    let wave = Wave::new(section_index, sample_rate, samples.into());

    Ok(wave)
}

fn f16_to_f32(f16_value: u16) -> f32 {
    // TODO: This can be done on the GPU for large datasets instead of the CPU
    let h = f16_value as u32;

    let sign     = (h & 0x8000) << 16;           // bit 15 → bit 31
    let exponent = (h & 0x7C00) >> 10;            // bits 14-10
    let mantissa = (h & 0x03FF) << 13;            // bits 9-0 → bits 22-13

    let f32_bits = if exponent == 0 {
        // zero or subnormal
        sign | mantissa
    } else if exponent == 31 {
        // inf or NaN
        sign | 0x7F80_0000 | mantissa
    } else {
        // normal: rebias exponent from 15 to 127 (+112)
        sign | ((exponent + 112) << 23) | mantissa
    };

    f32::from_bits(f32_bits)
}

fn convert_bits_32<T: std::ops::BitOrAssign<T>>(from: &mut Bytes<File>, little_endian: bool) -> T {
    let size = size_of::<T>();

    if size != 4 {
        panic!("This function was made to work with 32-bit values only")
    }

    let mut value_u32 = 0u32;

    for i in 0..size {
        let offset_bytes = if !little_endian { size - i - 1 } else { i };

        let new_value_part = (from.next().unwrap().unwrap() as u32) << (offset_bytes * 8);
        value_u32 |= new_value_part;
    }

    let value: T = unsafe { std::mem::transmute_copy(&value_u32) };

    value
}