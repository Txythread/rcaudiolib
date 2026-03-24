mod load_wave_rmaf;

use std::fs::File;
use std::io::Bytes;
use std::rc::Rc;
use derive_new::new;

#[derive(new)]
pub struct Wave {

    /// For keeping track of where a fragment of a wave sits if multiple
    /// are present for whatever reason (probably memory fragmentation).
    ///
    /// To be zero and ignored otherwise
    pub section_index: u32,

    /// In this structure (as well as in basically every digital audio/wave
    /// format), **the wave is stored as a list of values for specific times**.
    /// The intervals between them are regular (1/sampling_rate seconds).
    ///
    /// This value defines how many of those samples are taken per second
    /// (**Samples/second**).
    pub sample_rate: f32,

    /// The actual samples, as explained in [sample_rate]
    pub samples: Rc<[f32]>
}

#[cfg(feature="rmaf")]
pub fn load_bytes_rmaf(raw_data: Bytes<File>) -> Result<Wave, anyhow::Error> {
    load_wave_rmaf::load_bytes(raw_data)
}

