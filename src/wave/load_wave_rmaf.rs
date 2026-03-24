use std::fs::File;
use std::io::Bytes;
use anyhow::anyhow;
use crate::wave::Wave;


#[cfg(feature = "rmaf")]
pub fn load_bytes(raw_data: Bytes<File>) -> Result<Wave, anyhow::Error> {
    let mut rmaf_file_indicator = ".RMAF   ".chars();

    let mut raw_data = raw_data;

    let characters_count = rmaf_file_indicator.clone().count();

    for _ in 0..characters_count {
        if raw_data.next().unwrap().ok() != Some(rmaf_file_indicator.next().unwrap() as u8) {
            return Err(anyhow!("Expected RMAF file but another file type was provided."))
        }
    }

    let version: u32 = convert_bits_32(&mut raw_data, false);

    if version != 0 {
        return Err(anyhow!("RMAF file version unknown. Known versions: 0. Version Found: {version}"));
    }

    let section_index: u32 = convert_bits_32(&mut raw_data, false);

    let sample_rate: f32 = unsafe { std::mem::transmute_copy(&convert_bits_32::<u32>(&mut raw_data, true)) };

    //let header_length = 8 + 4 + 4 + 4 + 4;

    let header_end_code = 0xFF_CC_BB_AAu32;
    let actual_code: u32 = convert_bits_32(&mut raw_data, true);

    if header_end_code != actual_code {
        return Err(anyhow!("RMAF file header seems to have been corrupted. Expected {:#x}, received: {:#x}", header_end_code, actual_code));
    }

    let mut samples: Vec<f32> = Vec::new();

    while let Some(byte1) = raw_data.next() {
        let byte1 = byte1? as u16;
        let byte2 = raw_data.next().unwrap().unwrap() as u16;

        let data = (byte1 << 8) | byte2;

        let sample = f16_to_f32(data);
        samples.push(sample);
    }



    let wave = Wave::new(section_index, sample_rate, samples.into());

    Ok(wave)
}

fn f16_to_f32(f16_value: u16) -> f32 {
    // The base value containing the sign for the mantissa
    let f32_mantissa = if f16_value & 0x03_00 != 0 { 0x7F_FC_00u32 } else { 0x0u32 };
    let f16_mantissa = (f16_value as u32) & 0x03_FF;

    let mantissa = f16_mantissa | f32_mantissa;

    let f32_exponent = if f16_value & 0x80_00 != 0 { 0xF0_00u32 } else { 0x0u32 };
    let f16_exponent = ((f16_value as u32) & 0xFC_00) >> 3;

    let exponent = f32_exponent | f16_exponent;

    let value_u32 = mantissa | exponent;

    f32::from_bits(value_u32)
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