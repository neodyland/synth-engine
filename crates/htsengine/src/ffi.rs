use std::{
    ffi::{CString, c_char},
    mem::MaybeUninit,
};

mod bindings {
    #![allow(warnings)]
    include!(concat!(env!("OUT_DIR"), "/", "hts_engine_bindings.rs"));
}

pub struct HTSEngine {
    inner: bindings::HTS_Engine,
}

impl HTSEngine {
    pub fn init() -> Self {
        Self {
            inner: unsafe {
                let mut engine = MaybeUninit::uninit();
                bindings::HTS_Engine_initialize(engine.as_mut_ptr());
                engine.assume_init()
            },
        }
    }
    pub fn set_speed(&mut self, f: f64) {
        unsafe {
            bindings::HTS_Engine_set_speed(&mut self.inner, f);
        }
    }
    pub fn load(&mut self, voices: &[&str]) -> crate::Result<()> {
        let mut c_voices = Vec::with_capacity(voices.len());
        for v in voices {
            c_voices.push(CString::new(*v).map_err(|_| crate::Error::InvalidAsCString)?);
        }

        let mut c_voices: Vec<*mut c_char> =
            c_voices.iter().map(|v| v.as_ptr() as *mut c_char).collect();
        if unsafe {
            bindings::HTS_Engine_load(&mut self.inner, c_voices.as_mut_ptr(), voices.len())
        } != 1
        {
            return Err(crate::Error::LoadFailed);
        }
        Ok(())
    }
    pub fn synthesize(&mut self, label: &str) -> crate::Result<()> {
        let lines: Vec<_> = label.lines().collect();
        let mut c_lines = Vec::with_capacity(lines.len());
        for l in &lines {
            c_lines.push(CString::new(*l).map_err(|_| crate::Error::InvalidAsCString)?);
        }

        let mut c_lines: Vec<*mut c_char> =
            c_lines.iter().map(|l| l.as_ptr() as *mut c_char).collect();
        if unsafe {
            bindings::HTS_Engine_synthesize_from_strings(
                &mut self.inner,
                c_lines.as_mut_ptr(),
                lines.len(),
            )
        } != 1
        {
            return Err(crate::Error::SynthesizeFailed);
        };
        Ok(())
    }
    pub fn write<E>(&mut self, mut f: impl FnMut(i16) -> Result<(), E>) -> Result<(), E> {
        let length = unsafe { bindings::HTS_GStreamSet_get_total_nsamples(&mut self.inner.gss) };
        for i in 0..length {
            f(
                unsafe { bindings::HTS_GStreamSet_get_speech(&mut self.inner.gss, i) }
                    .clamp(-32768.0, 32767.0) as i16,
            )?;
        }
        Ok(())
    }
}

impl Drop for HTSEngine {
    fn drop(&mut self) {
        unsafe {
            bindings::HTS_Engine_clear(&mut self.inner);
        }
    }
}
