use windows_sys::Win32::Foundation::NO_ERROR;
use windows_sys::Win32::NetworkManagement::WNet;
use std::ffi::CString;

pub fn create_connection(server: &str, username: Option<&str>, password: Option<&str>) -> Result<(), i32>
{
    let mut resources = WNet::NETRESOURCEA {
        dwDisplayType: WNet::RESOURCEDISPLAYTYPE_SHAREADMIN,
        dwScope: WNet::RESOURCE_GLOBALNET,
        dwType: WNet::RESOURCETYPE_DISK,
        dwUsage: WNet::RESOURCEUSAGE_ALL,
        lpComment: std::ptr::null_mut(),
        lpLocalName: std::ptr::null_mut(),
        lpProvider: std::ptr::null_mut(),
        lpRemoteName: CString::new(server).unwrap().into_raw() as *mut u8,
    };

    let username = username.as_ref().map(|username| CString::new(*username).unwrap());
    let password = password.as_ref().map(|password| CString::new(*password).unwrap());

    let result = unsafe {
        let username_ptr = username
            .as_ref()
            .map(|username| username.as_ptr())
            .unwrap_or(std::ptr::null());
        let password_ptr = password
            .as_ref()
            .map(|password| password.as_ptr())
            .unwrap_or(std::ptr::null());
        WNet::WNetAddConnection2A(
            &mut resources as *mut WNet::NETRESOURCEA,
            password_ptr as *const u8,
            username_ptr as *const u8,
            WNet::CONNECT_TEMPORARY,
        )
    };

    if result == NO_ERROR {
        return Ok(())
    } else {
        return Err(result as i32)
    }
}