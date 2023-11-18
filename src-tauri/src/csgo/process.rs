use std::collections::HashMap;
use std::ffi::CString;
use std::mem::size_of;
use std::ptr::{self, null_mut};
use winapi::shared::basetsd::SIZE_T;
use winapi::shared::minwindef::{DWORD, FALSE, LPCVOID, LPVOID};
use winapi::um::handleapi::CloseHandle;
use winapi::um::memoryapi::ReadProcessMemory;
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::tlhelp32::{
    CreateToolhelp32Snapshot, Module32First, Module32Next, Process32First, Process32Next,
    MODULEENTRY32, PROCESSENTRY32, TH32CS_SNAPMODULE, TH32CS_SNAPMODULE32,
};
use winapi::um::winnt::{HANDLE, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};

pub const TH32CS_SNAPPROCESS: DWORD = 0x00000002;

#[derive(Debug)]
pub struct InternalProcess {
    pub name: String,
    pub process_id: u32, // DWORD
    pub handle: u32,     // HANDLE
    pub modules: HashMap<String, usize>,
    // process_id and handle are put as the wrong types because the types from win32 don't implement the send and sync traits
    // this causes issues when trying to share data between threads
    // from tesing this hasn't caused any issues, but is probably not the best way to do it
}

impl InternalProcess {
    pub fn get_module_base(&self, name: &str) -> usize {
        self.modules.get(name).unwrap().clone()
    }
}

pub fn get_process_handle(process_name: &str) -> Option<InternalProcess> {
    // Used to get a snapshot of all processes
    // the flag 0x00000002 is used to get a snapshot of all processes
    let snapshot = unsafe { CreateToolhelp32Snapshot(0x00000002, 0) };

    // Could run into an error, causing snapshot to be null
    if snapshot == ptr::null_mut() {
        println!("snapshot is null");
        return None;
    }

    // Used to store information about a process
    let mut process_entry: PROCESSENTRY32 = PROCESSENTRY32::default();
    process_entry.dwSize = std::mem::size_of::<PROCESSENTRY32>() as DWORD;

    // Used to store the process handle and id
    let mut process_handle: HANDLE = ptr::null_mut();
    let mut process_id: DWORD = 0; // DWORD is a 32 bit unsigned integer, alis for c_ulong which is an alias for u32

    // Get the first process in the snapshot
    // Returns BOOL type, which is an alias for c_int, which is an alias for i32
    // We pass the process entry by reference, so it can store the information about the process
    if unsafe { Process32First(snapshot, &mut process_entry) } == FALSE {
        return None;
    }

    loop {
        let process_name_cstring = unsafe {
            // CString is a null terminated string,
            // From docs "a sequence of non-nul bytes terminated by a single nul byte"
            CString::from_vec_unchecked(
                process_entry
                    .szExeFile
                    .iter()
                    .map(|&i| i as u8)
                    .take_while(|&i| i != 0)
                    .collect(),
            )
        };
        let process_name_str = process_name_cstring.to_str().unwrap();

        // Compare the process name to the process name we are looking for
        if process_name_str == process_name {
            // If process names match, store the process id and handle
            process_id = process_entry.th32ProcessID;
            process_handle =
                // unsafe { OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, process_id) };
            unsafe { OpenProcess(winapi::um::winnt::PROCESS_ALL_ACCESS, 0, process_id) };
            break;
        }

        // If we have reached the end of the snapshot, break out of the loop
        if unsafe { Process32Next(snapshot, &mut process_entry) } == FALSE {
            break;
        }
    }

    // Close the snapshot handle, NOT the process handle
    unsafe { CloseHandle(snapshot) };

    // Verify we have a valid process handle
    if process_handle == ptr::null_mut() {
        return None;
    }

    Some(InternalProcess {
        name: process_name.to_string(),
        process_id,
        handle: process_handle as u32,
        modules: HashMap::new(),
    })
}

pub fn get_base_module_address(
    proc: &mut InternalProcess,
    module_name: &str,
) -> Result<(), String> {
    let snap_handle = unsafe {
        CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, proc.process_id)
    };

    if snap_handle == ptr::null_mut() {
        return Err("Snapshot is null".to_string());
    }

    let mut module_entry: MODULEENTRY32 = MODULEENTRY32::default();
    module_entry.dwSize = size_of::<MODULEENTRY32>() as u32;

    if unsafe { Module32First(snap_handle, &mut module_entry) } == FALSE {
        return Err("Module32First failed".to_string());
    }

    loop {
        let module_name_str = unsafe {
            CString::from_vec_unchecked(
                module_entry
                    .szModule
                    .iter()
                    .map(|&i| i as u8)
                    .take_while(|&i| i != 0)
                    .collect(),
            )
        };

        if module_name_str.to_str().unwrap() == module_name {
            proc.modules
                .insert(module_name.to_string(), module_entry.modBaseAddr as usize);
            return Ok(());
        }

        if unsafe { Module32Next(snap_handle, &mut module_entry) } == FALSE {
            break;
        }
    }

    Err("Module not found".to_string())
}

pub fn read_process_memory<T>(
    proc: &InternalProcess,
    buffer: *mut T,
    offset: usize,
) -> Result<(), String> {
    let ok = unsafe {
        ReadProcessMemory(
            proc.handle as *mut winapi::ctypes::c_void,
            offset as LPCVOID,
            buffer as *mut T as LPVOID,
            size_of::<T>() as SIZE_T,
            null_mut::<SIZE_T>(),
        )
    };

    if ok == FALSE {
        return Err("ReadProcessMemory failed".to_string());
    }

    return Ok(());
}

pub fn read_multi_level(
    proc: &InternalProcess,
    offsets: Vec<usize>,
    module: &str,
) -> Result<u32, String> {
    let mut buffer: u32 = 0;

    for (i, offset) in offsets.iter().enumerate() {
        if i == 0 {
            if let Err(e) =
                read_process_memory::<u32>(proc, &mut buffer, proc.get_module_base(module) + offset)
            {
                return Err(e);
            }
            continue;
        }
        read_process_memory::<u32>(proc, &mut buffer, (buffer.clone() as usize) + offset)?;
    }

    return Ok(buffer);
}
