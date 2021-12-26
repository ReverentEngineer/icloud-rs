use crate::error::Error;
use std::ffi::CStr;

#[allow(non_camel_case_types)]
type uuid_t = [u8; 16];

#[link(name = "uuid")]
extern "C" {

    fn uuid_generate_time(out: *mut uuid_t);
    fn uuid_unparse(uu: *const uuid_t, out: *mut libc::c_char);

}

pub fn generate_uuid() -> Result<String, Error> {
    unsafe {
        let mut uuid: uuid_t = [0; 16];
        let mut uuid_cstr = vec![0; 256];
        uuid_generate_time(&mut uuid);
        uuid_unparse(&uuid, uuid_cstr.as_mut_ptr());
        Ok(CStr::from_ptr(uuid_cstr.as_ptr()).to_str()?.to_owned())
    }
}
