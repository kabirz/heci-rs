use std::ffi::c_void;
use std::mem::MaybeUninit;
use std::mem::size_of;
use std::ptr;
use crate::*;

use windows::{
    core::{GUID, PCSTR, PCWSTR},
    Win32::{
        Devices::DeviceAndDriverInstallation::{
            CM_Get_Device_Interface_ListW, CM_Get_Device_Interface_List_SizeA,
            CM_GET_DEVICE_INTERFACE_LIST_PRESENT, CR_SUCCESS,
        },
        Foundation::{BOOL, HANDLE, CloseHandle},
        Storage::FileSystem::*,
        System::IO::*,
    },
};

enum DeviceError {
    FailedToGetDeviceInterfaceListSize,
    FailedToGetDeviceInterfaceList,
    EmptyDeviceInterfaceList,
    NoValidHandle,
    NotFound,
}

unsafe fn find_handler(guid: &GUID) -> Result<HANDLE, DeviceError> {
    let mut maybe_device_interface_list_length = MaybeUninit::<u32>::uninit();

    let interface_list_size_result = CM_Get_Device_Interface_List_SizeA(
        maybe_device_interface_list_length.as_mut_ptr(),
        guid,
        PCSTR::null(),
        CM_GET_DEVICE_INTERFACE_LIST_PRESENT,
    );

    if interface_list_size_result != CR_SUCCESS {
        return Err(DeviceError::FailedToGetDeviceInterfaceListSize);
    }
    let interface_list_length = maybe_device_interface_list_length.assume_init() as usize;

    // Will be 1 if no interfaces found.
    if interface_list_length <= 1 {
        return Err(DeviceError::EmptyDeviceInterfaceList);
    }
    let mut interface_list_vec: Vec<u16> = Vec::new();
    interface_list_vec.set_len(0);
    interface_list_vec.reserve(interface_list_length as usize);

    // let mut l_buffer: [u16; interface_list_length as usize];
    let interface_list_result = CM_Get_Device_Interface_ListW(
        guid,
        PCWSTR::null(),
        &mut interface_list_vec[..],
        CM_GET_DEVICE_INTERFACE_LIST_PRESENT,
    );
    if interface_list_result != CR_SUCCESS {
        return Err(DeviceError::FailedToGetDeviceInterfaceList);
    }

    let interface_list_str = String::from_utf16(&interface_list_vec).unwrap();

    for interface in interface_list_str.split("\0") {
        if interface.trim().is_empty() {
            continue;
        }
        let handle = CreateFileA(
            PCSTR::from_raw(interface.as_ptr() as *const u8),
            FILE_GENERIC_WRITE,
            FILE_SHARE_WRITE,
            Some(ptr::null()),
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            None,
        );
        if handle.is_err() {
            return Err(DeviceError::NoValidHandle);
        } else {
            let handle = handle.unwrap();
            return Ok(handle);
        }
    }
    return Err(DeviceError::NotFound);
}
impl Heci {
    pub fn new(guid: &str) -> Self {
        let guid = GUID::from(guid);
        let handle = unsafe { find_handler(&guid) };
        let h = match handle {
            Ok(handle) => handle.0,
            _ => -1,
        };
        Self { device: h }
    }
}

impl HeciOp for Heci {
    fn connect(&self, guid: &str) -> i32 {
        if self.device == -1isize {
            return self.device as i32;
        }
        let guid = GUID::from(guid);
        let device = HANDLE(self.device);
        let out_ptr = ptr::null_mut();
        let out_size = 0;

        unsafe {
            let state_ptr: *const c_void = &guid as *const _ as *const c_void;
            let ret = DeviceIoControl(
                device,
                0x8000e004,
                Some(state_ptr),
                size_of::<GUID>() as _,
                Some(out_ptr),
                out_size,
                Some(ptr::null_mut()),
                Some(ptr::null_mut()),
            );
            ret.0
        }
    }
    fn write(&self, data: &[u8]) -> i32 {
        if self.device == -1isize {
            return self.device as i32;
        }
        let mut write_len: u32 = data.len() as u32;
        let device = HANDLE(self.device);
        let mut opd: OVERLAPPED = OVERLAPPED::default();
        unsafe {
            let ret = WriteFile(
                device,
                Some(data as *const _ as *const c_void),
                data.len() as u32,
                Some(&mut write_len),
                Some(&mut opd),
            );
            if ret.0 != 0 {
                return ret.0;
            }
            let wait = BOOL(0);
            let ret = GetOverlappedResult(
                device,
                &opd,
                &mut write_len,
                wait
            );
            if ret.0 != 0 {
                return ret.0;
            }
            write_len as i32
        }
    }

    fn read(&self, data: &mut [u8]) -> i32 {
        if self.device == -1isize {
            return self.device as i32;
        }
        let mut read_len = data.len() as u32;
        let device = HANDLE(self.device);
        let mut opd: OVERLAPPED = OVERLAPPED::default();
        unsafe {
            let ret = ReadFile(
                device,
                Some(data as *mut _ as *mut c_void),
                data.len() as u32,
                Some(&mut read_len),
                Some(&mut opd),
            );
            if ret.0 != 0 {
                return ret.0;
            }
            let wait = BOOL(0);
            let ret = GetOverlappedResult(
                device,
                &opd,
                &mut read_len,
                wait
            );
            if ret.0 != 0 {
                return ret.0;
            }
            read_len as i32
        }

    }
    fn close(&self) {
        if self.device != -1isize {
            unsafe {
                CloseHandle(HANDLE(self.device));
            }
        }
    }
}
