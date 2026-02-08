use netstat2::{get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo, TcpState};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use sysinfo::{System, Process, Pid};
use std::fs;
use std::io::Cursor;
#[cfg(target_os = "windows")]
use image;
#[cfg(target_os = "windows")]
use winapi::shared::windef::HICON;
#[cfg(target_os = "macos")]
use tauri::Manager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpConnection {
    pub protocol: String,
    pub local_addr: String,
    pub local_port: u16,
    pub remote_addr: String,
    pub remote_port: u16,
    pub state: String,
    pub pid: Option<u32>,
    pub process_name: Option<String>,
    pub icon: Option<String>, // Base64 encoded icon data
    pub start_time: Option<u64>, // Process start time in seconds since Unix epoch
    pub fill_column: String, // Fill column for filling remaining space
}

// 定义进程详情结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessDetails {
    pub pid: u32,
    pub name: String,
    pub command_line: String,
    pub executable_path: String,
    pub memory_usage: u64,
    pub cpu_usage: f32,
    pub parent_pid: Option<u32>,
    pub start_time: u64,
}

// 将 netstat2 的 TCP 状态转换为字符串
fn tcp_state_to_string(state: TcpState) -> String {
    match state {
        TcpState::Established => "ESTABLISHED".to_string(),
        TcpState::SynSent => "SYN_SENT".to_string(),
        TcpState::SynReceived => "SYN_RECV".to_string(),
        TcpState::FinWait1 => "FIN_WAIT1".to_string(),
        TcpState::FinWait2 => "FIN_WAIT2".to_string(),
        TcpState::TimeWait => "TIME_WAIT".to_string(),
        TcpState::Closed => "CLOSED".to_string(),
        TcpState::CloseWait => "CLOSE_WAIT".to_string(),
        TcpState::LastAck => "LAST_ACK".to_string(),
        TcpState::Listen => "LISTEN".to_string(),
        TcpState::Closing => "CLOSING".to_string(),
        TcpState::DeleteTcb => "DELETE_TCB".to_string(),
        TcpState::Unknown => "UNKNOWN".to_string(),
    }
}

// 获取进程的应用图标
#[cfg(target_os = "windows")]
fn get_process_icon_by_pid(pid: u32) -> Option<String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;
    use winapi::um::shellapi::ExtractIconW;
    use winapi::um::winuser::{DestroyIcon};
    use winapi::shared::windef::HICON;
    use winapi::shared::minwindef::HINSTANCE;
    
    // 尝试通过PID获取进程的可执行文件路径
    if let Ok(exe_path) = get_executable_path_from_pid(pid) {
        // 将路径转换为宽字符字符串
        let wide_path: Vec<u16> = OsStr::new(&exe_path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        
        // 使用ExtractIconW从EXE文件中提取图标
        unsafe {
            let h_instance = ptr::null_mut() as HINSTANCE;
            let h_icon = ExtractIconW(
                h_instance,
                wide_path.as_ptr(),
                0, // 第一个图标
            );
            
            // 检查图标句柄是否有效
            if h_icon as usize > 1 { // 0和1是特殊值，表示没有图标或错误
                // 尝试将图标转换为图像数据
                let icon_data = extract_icon_to_png(h_icon);
                
                // 销毁图标句柄
                DestroyIcon(h_icon);
                
                return icon_data;
            } else {
                // 销毁无效图标句柄
                if h_icon as usize > 1 {
                    DestroyIcon(h_icon);
                }
            }
        }
    }
    
    None
}

// 辅助函数：将图标转换为PNG数据的Base64编码
#[cfg(target_os = "windows")]
unsafe fn extract_icon_to_png(h_icon: HICON) -> Option<String> {
    use winapi::um::winuser::{GetIconInfo, GetDC, ReleaseDC, DrawIconEx};
    use winapi::um::wingdi::{GetDIBits, CreateDIBSection, DeleteObject, 
                             BI_RGB, DIB_RGB_COLORS, SelectObject, CreateCompatibleDC, DeleteDC};
    use winapi::shared::windef::{HDC, HBITMAP};
    use winapi::um::winnt::HANDLE;
    use std::ptr;
    use std::mem;
    
    // 获取图标信息
    let mut icon_info: winapi::um::winuser::ICONINFO = mem::zeroed();
    if GetIconInfo(h_icon, &mut icon_info) == 0 {
        return None;
    }
    
    // 获取屏幕DC
    let hdc_screen = GetDC(ptr::null_mut());
    if hdc_screen.is_null() {
        // 清理资源
        if !icon_info.hbmColor.is_null() {
            DeleteObject(icon_info.hbmColor as *mut _);
        }
        if !icon_info.hbmMask.is_null() {
            DeleteObject(icon_info.hbmMask as *mut _);
        }
        return None;
    }
    
    // 创建兼容DC
    let hdc_mem = CreateCompatibleDC(hdc_screen);
    if hdc_mem.is_null() {
        ReleaseDC(ptr::null_mut(), hdc_screen);
        if !icon_info.hbmColor.is_null() {
            DeleteObject(icon_info.hbmColor as *mut _);
        }
        if !icon_info.hbmMask.is_null() {
            DeleteObject(icon_info.hbmMask as *mut _);
        }
        return None;
    }
    
    // 选择位图到内存DC
    let hbm_old = SelectObject(hdc_mem, icon_info.hbmColor as *mut _);
    
    // 获取颜色位图信息
    let mut bitmap: winapi::um::wingdi::BITMAP = mem::zeroed();
    if winapi::um::wingdi::GetObjectW(
        icon_info.hbmColor as HANDLE,
        mem::size_of::<winapi::um::wingdi::BITMAP>() as i32,
        &mut bitmap as *mut _ as *mut _,
    ) == 0 {
        SelectObject(hdc_mem, hbm_old);
        DeleteDC(hdc_mem);
        ReleaseDC(ptr::null_mut(), hdc_screen);
        if !icon_info.hbmColor.is_null() {
            DeleteObject(icon_info.hbmColor as *mut _);
        }
        if !icon_info.hbmMask.is_null() {
            DeleteObject(icon_info.hbmMask as *mut _);
        }
        return None;
    }
    
    let width = bitmap.bmWidth;
    let height = bitmap.bmHeight.abs();
    
    // 准备BITMAPINFO结构
    let mut bmi: winapi::um::wingdi::BITMAPINFO = mem::zeroed();
    bmi.bmiHeader.biSize = mem::size_of::<winapi::um::wingdi::BITMAPINFOHEADER>() as u32;
    bmi.bmiHeader.biWidth = width;
    bmi.bmiHeader.biHeight = height;
    bmi.bmiHeader.biPlanes = 1;
    bmi.bmiHeader.biBitCount = 32;
    bmi.bmiHeader.biCompression = BI_RGB;
    
    // 创建DIB Section
    let mut bits: *mut winapi::ctypes::c_void = ptr::null_mut();
    let h_bitmap = CreateDIBSection(
        hdc_mem,
        &bmi,
        DIB_RGB_COLORS,
        &mut bits,
        ptr::null_mut(),
        0,
    );
    
    if h_bitmap.is_null() {
        SelectObject(hdc_mem, hbm_old);
        DeleteDC(hdc_mem);
        ReleaseDC(ptr::null_mut(), hdc_screen);
        if !icon_info.hbmColor.is_null() {
            DeleteObject(icon_info.hbmColor as *mut _);
        }
        if !icon_info.hbmMask.is_null() {
            DeleteObject(icon_info.hbmMask as *mut _);
        }
        return None;
    }
    
    // 选择新位图到内存DC
    let hbm_old_dib = SelectObject(hdc_mem, h_bitmap as *mut _);
    
    // 将原图标绘制到位图上
    DrawIconEx(
        hdc_mem,
        0, 0,
        h_icon,
        width, height,
        0,
        ptr::null_mut(),
        0x0003, // DI_NORMAL
    );
    
    // 获取DIB位
    let buffer_size = (width * height * 4) as usize;
    let mut buffer: Vec<u8> = vec![0; buffer_size];
    
    let nbits = GetDIBits(
        hdc_mem,
        h_bitmap,
        0,
        height as u32,
        buffer.as_mut_ptr() as *mut _,
        &mut bmi,
        DIB_RGB_COLORS,
    );
    
    // 恢复DC状态
    SelectObject(hdc_mem, hbm_old_dib);
    SelectObject(hdc_mem, hbm_old);
    
    // 清理资源
    DeleteObject(h_bitmap as *mut _);
    DeleteDC(hdc_mem);
    ReleaseDC(ptr::null_mut(), hdc_screen);
    
    if nbits != 0 {
        // 将BGRA转换为RGBA（Windows使用BGRA，而PNG使用RGBA）
        for chunk in buffer.chunks_exact_mut(4) {
            chunk.swap(0, 2); // 交换B和R通道
        }
        
        // 将图标数据转换为PNG格式
        use std::io::Cursor;
        use image::ImageFormat;
        
        if let Some(img) = image::RgbaImage::from_raw(width as u32, height as u32, buffer) {
            let mut png_data: Vec<u8> = Vec::new();
            if let Ok(_) = img.write_to(&mut Cursor::new(&mut png_data), ImageFormat::Png) {
                let base64_icon = base64::encode(&png_data);
                
                if !icon_info.hbmColor.is_null() {
                    DeleteObject(icon_info.hbmColor as *mut _);
                }
                if !icon_info.hbmMask.is_null() {
                    DeleteObject(icon_info.hbmMask as *mut _);
                }
                
                return Some(base64_icon);
            }
        }
    }
    
    // 清理资源
    if !icon_info.hbmColor.is_null() {
        DeleteObject(icon_info.hbmColor as *mut _);
    }
    if !icon_info.hbmMask.is_null() {
        DeleteObject(icon_info.hbmMask as *mut _);
    }
    
    None
}

// 辅助函数：从EXE文件中提取图标
#[cfg(target_os = "windows")]
unsafe fn extract_icon_from_exe(exe_path: &str) -> Option<String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use winapi::um::winuser::{DrawIconEx, GetDC, ReleaseDC};
    use winapi::um::wingdi::{CreateCompatibleDC, DeleteDC, GetDIBits, CreateDIBSection, DeleteObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS};
    use winapi::shared::windef::{HICON, HDC, HBITMAP};
    use winapi::um::libloaderapi::{LoadLibraryW, FreeLibrary};
    use winapi::shared::minwindef::HMODULE;
    use std::ptr;
    use std::mem;
    
    // 将路径转换为宽字符字符串
    let wide_path: Vec<u16> = OsStr::new(exe_path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    
    // 加载模块
    let h_module: HMODULE = LoadLibraryW(wide_path.as_ptr());
    if h_module.is_null() {
        return None;
    }
    
    // 这里我们尝试使用Windows API从EXE文件中提取图标
    // 由于这涉及复杂的Windows API调用，我们简化处理
    // 实际上，我们会使用一个专门的库来处理这个
    
    // 释放模块
    FreeLibrary(h_module);
    
    // 由于手动实现从EXE提取图标非常复杂，我们暂时返回None
    // 在实际应用中，建议使用专门的库如'icon-park'或'windows-icons'
    None
}

#[cfg(all(not(target_os = "windows"), target_os = "linux"))]
fn get_process_icon_by_pid(pid: u32) -> Option<String> {
    use std::process::Command;
    use std::path::Path;

    // 尝试通过PID获取进程的可执行文件路径
    let exe_path = format!("/proc/{}/exe", pid);
    let process_name = if Path::new(&exe_path).exists() {
        // 读取符号链接以获取可执行文件路径
        std::fs::read_link(&exe_path)
            .ok()?
            .to_string_lossy()
            .to_string()
    } else {
        // 如果无法读取exe链接，尝试从cmdline获取
        let cmdline_path = format!("/proc/{}/cmdline", pid);
        if let Ok(cmdline) = std::fs::read_to_string(&cmdline_path) {
            cmdline.split('\0').next().unwrap_or("").to_string()
        } else {
            return None;
        }
    };

    // 从可执行文件路径提取进程名
    let basename = Path::new(&process_name)
        .file_name()?
        .to_string_lossy()
        .to_string();

    // 尝试多种可能的图标路径
    let icon_sizes = ["16x16", "24x24", "32x32", "48x48", "64x64", "128x128", "256x256", "scalable"];
    let icon_themes = ["hicolor", "oxygen", "gnome", "breeze"];
    let icon_types = ["apps", "categories", "devices", "mimetypes"];

    // 首先尝试桌面文件中指定的图标
    let desktop_file_path = format!("/usr/share/applications/{}.desktop", &basename);
    if Path::new(&desktop_file_path).exists() {
        if let Ok(desktop_content) = std::fs::read_to_string(&desktop_file_path) {
            for line in desktop_content.lines() {
                if line.starts_with("Icon=") {
                    let icon_name = line.strip_prefix("Icon=").unwrap_or("");
                    if !icon_name.is_empty() {
                        // 尝试查找该图标名称
                        for size in &icon_sizes {
                            for theme in &icon_themes {
                                for icon_type in &icon_types {
                                    let icon_path = format!(
                                        "/usr/share/icons/{}/{}/{}/{}.png",
                                        theme, size, icon_type, icon_name
                                    );
                                    if Path::new(&icon_path).exists() {
                                        if let Ok(image_data) = std::fs::read(&icon_path) {
                                            return Some(base64::encode(&image_data));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 如果桌面文件没有帮助，尝试基于进程名查找图标
    for size in &icon_sizes {
        for theme in &icon_themes {
            for icon_type in &icon_types {
                let icon_paths = vec![
                    format!("/usr/share/icons/{}/{}/{}/{}.png", theme, size, icon_type, &basename),
                    format!("/usr/share/icons/{}/{}/{}/{}.svg", theme, size, icon_type, &basename),
                    format!("/usr/share/pixmaps/{}.png", &basename),
                    format!("/usr/share/pixmaps/{}.svg", &basename),
                ];

                for icon_path in icon_paths {
                    if Path::new(&icon_path).exists() {
                        if let Ok(image_data) = std::fs::read(&icon_path) {
                            return Some(base64::encode(&image_data));
                        }
                    }
                }
            }
        }
    }

    None
}

#[cfg(all(not(target_os = "windows"), target_os = "macos"))]
fn get_process_icon_by_pid(pid: u32) -> Option<String> {
    use std::process::Command;

    // 首先尝试使用lsof命令获取进程的可执行文件路径
    let lsof_output = Command::new("lsof")
        .args(&["-p", &pid.to_string(), "-F", "n"])
        .output()
        .ok()?;

    if !lsof_output.status.success() {
        return None;
    }

    let lsof_stdout = String::from_utf8(lsof_output.stdout).ok()?;
    let executable_path = lsof_stdout
        .lines()
        .find(|line| line.starts_with('n') && (line.contains("/Contents/MacOS/") || line.contains(".app")))
        .map(|line| line[1..].to_string()); // 移除'n'前缀

    // 如果找到了.app包路径，构建对应的Resources路径
    if let Some(path) = executable_path {
        if path.contains("/Contents/MacOS/") {
            // 从可执行文件路径推导出Resources路径
            if let Some(contents_index) = path.find("/Contents/MacOS/") {
                let resources_path = format!("{}{}", 
                    &path[..contents_index], 
                    "/Contents/Resources/");
                
                // 尝试找到Info.plist来确定主图标文件名
                let info_plist_path = format!("{}{}", resources_path, "Info.plist");
                
                // 默认图标名称
                let mut icon_name = "App.icns".to_string();
                
                // 尝试从Info.plist中读取CFBundleIconFile
                if std::path::Path::new(&info_plist_path).exists() {
                    if let Ok(plist_content) = std::fs::read_to_string(&info_plist_path) {
                        // 简单解析plist文件查找图标名称
                        if let Some(start) = plist_content.find("<key>CFBundleIconFile</key>") {
                            if let Some(content_start) = plist_content[start..].find("<string>").map(|i| start + i + 8) {
                                if let Some(content_end) = plist_content[content_start..].find("</string>").map(|i| content_start + i) {
                                    let extracted_icon_name = &plist_content[content_start..content_end];
                                    if !extracted_icon_name.is_empty() {
                                        icon_name = extracted_icon_name.to_string();
                                        
                                        // 确保图标文件有.icns扩展名
                                        if !icon_name.ends_with(".icns") {
                                            icon_name.push_str(".icns");
                                        }
                                    }
                                }
                            }
                        }
                        
                        // 如果上面没找到，尝试找SF Bundle Icons
                        if icon_name == "App.icns" {
                            if let Some(start) = plist_content.find("<key>CFBundleIcons</key>") {
                                if let Some(key_pos) = plist_content[start..].find("<key>CFBundleIconFile</key>").map(|i| start + i) {
                                    if let Some(content_start) = plist_content[key_pos..].find("<string>").map(|i| key_pos + i + 8) {
                                        if let Some(content_end) = plist_content[content_start..].find("</string>").map(|i| content_start + i) {
                                            let extracted_icon_name = &plist_content[content_start..content_end];
                                            if !extracted_icon_name.is_empty() {
                                                icon_name = extracted_icon_name.to_string();
                                                
                                                // 确保图标文件有.icns扩展名
                                                if !icon_name.ends_with(".icns") {
                                                    icon_name.push_str(".icns");
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                
                let icon_path = format!("{}{}", resources_path, icon_name);
                if std::path::Path::new(&icon_path).exists() {
                    // 尝试使用sips将icns转换为png
                    if let Ok(png_data) = convert_icns_to_png(&icon_path) {
                        return Some(base64::encode(&png_data));
                    }
                }
                
                // 如果指定的图标文件不存在，尝试常见的图标文件名
                let common_icon_names = vec![
                    "App.icns".to_string(),
                    "app.icns".to_string(),
                    format!("{}.icns", icon_name.replace(".icns", "")),
                    "Icon.icns".to_string(),
                    "icon.icns".to_string(),
                    "AppIcon.icns".to_string(),
                    "appicon.icns".to_string()
                ];
                
                for icon_name in &common_icon_names {
                    let fallback_icon_path = format!("{}{}", resources_path, icon_name);
                    if std::path::Path::new(&fallback_icon_path).exists() {
                        if let Ok(png_data) = convert_icns_to_png(&fallback_icon_path) {
                            return Some(base64::encode(&png_data));
                        }
                    }
                }
            }
        }
    } else {
        // 如果无法从lsof获取.app路径，尝试使用ps命令获取进程名并查找应用
        let ps_output = Command::new("ps")
            .args(&["-o", "comm=", "-p", &pid.to_string()])
            .output()
            .ok()?;

        if !ps_output.status.success() {
            return None;
        }

        let raw_process_name = String::from_utf8(ps_output.stdout).ok()?;
        let process_name = raw_process_name.trim();
        
        // 提取进程名（去除路径）
        let clean_process_name = process_name
            .rsplit('/')
            .next()
            .unwrap_or(process_name)
            .trim();

        // 尝试在多个位置查找.app包
        let app_paths = [
            format!("/Applications/{}.app", clean_process_name),
            format!("/Applications/{}/{}.app", clean_process_name, clean_process_name),
            format!("/System/Applications/{}.app", clean_process_name),
            format!("/System/Applications/{}/{}.app", clean_process_name, clean_process_name),
            format!("/Users/Shared/Applications/{}.app", clean_process_name),
        ];

        for app_path in &app_paths {
            if std::path::Path::new(app_path).exists() {
                // 尝试从Info.plist获取图标名
                let info_plist_path = format!("{}/Contents/Resources/Info.plist", app_path);
                let mut icon_name = "App.icns".to_string();
                
                if std::path::Path::new(&info_plist_path).exists() {
                    if let Ok(plist_content) = std::fs::read_to_string(&info_plist_path) {
                        if let Some(start) = plist_content.find("<key>CFBundleIconFile</key>") {
                            if let Some(content_start) = plist_content[start..].find("<string>").map(|i| start + i + 8) {
                                if let Some(content_end) = plist_content[content_start..].find("</string>").map(|i| content_start + i) {
                                    let extracted_icon_name = &plist_content[content_start..content_end];
                                    if !extracted_icon_name.is_empty() {
                                        icon_name = extracted_icon_name.to_string();
                                        
                                        // 确保图标文件有.icns扩展名
                                        if !icon_name.ends_with(".icns") {
                                            icon_name.push_str(".icns");
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                let resources_path = format!("{}/Contents/Resources/", app_path);
                let icon_path = format!("{}{}", resources_path, icon_name);
                if std::path::Path::new(&icon_path).exists() {
                    if let Ok(png_data) = convert_icns_to_png(&icon_path) {
                        return Some(base64::encode(&png_data));
                    }
                }
                
                // 如果指定的图标文件不存在，尝试常见的图标文件名
                let common_icon_names = vec![
                    "App.icns".to_string(),
                    "app.icns".to_string(),
                    format!("{}.icns", icon_name.replace(".icns", "")),
                    "Icon.icns".to_string(),
                    "icon.icns".to_string(),
                    "AppIcon.icns".to_string(),
                    "appicon.icns".to_string()
                ];
                
                for icon_name in &common_icon_names {
                    let fallback_icon_path = format!("{}{}", resources_path, icon_name);
                    if std::path::Path::new(&fallback_icon_path).exists() {
                        if let Ok(png_data) = convert_icns_to_png(&fallback_icon_path) {
                            return Some(base64::encode(&png_data));
                        }
                    }
                }
            }
        }
    }

    // 如果找不到应用图标，尝试使用系统默认方法
    None
}

// macOS辅助函数：将ICNS转换为PNG
#[cfg(target_os = "macos")]
fn convert_icns_to_png(icns_path: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    use std::process::Command;
    use std::fs;
    use tempfile::NamedTempFile;
    use std::io::{Seek, Read};

    // 创建临时文件用于输出PNG
    let mut temp_file = NamedTempFile::new()?;
    
    // 使用sips命令将ICNS转换为PNG
    let output = Command::new("sips")
        .args(&[
            "-s", "format", "png",
            icns_path,
            "--out", temp_file.path().to_str().unwrap()
        ])
        .output()?;

    if !output.status.success() {
        return Err(format!("sips command failed: {}", String::from_utf8_lossy(&output.stderr)).into());
    }

    // 读取临时文件内容
    temp_file.seek(std::io::SeekFrom::Start(0))?;
    let mut png_data = Vec::new();
    std::io::Read::read_to_end(&mut temp_file, &mut png_data)?;

    Ok(png_data)
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
fn get_process_icon_by_pid(_pid: u32) -> Option<String> {
    // 对于其他平台，返回None
    None
}

// 通过PID获取可执行文件路径的辅助函数
#[cfg(target_os = "windows")]
fn get_executable_path_from_pid(pid: u32) -> Result<String, Box<dyn std::error::Error>> {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use winapi::um::processthreadsapi::OpenProcess;
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::winnt::PROCESS_QUERY_INFORMATION;
    use std::ptr;

    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_INFORMATION, 0, pid);
        if handle.is_null() {
            return Err("Failed to open process".into());
        }

        let mut buffer: [u16; 1024] = [0; 1024];
        let size = buffer.len();

        let success = winapi::um::psapi::GetModuleFileNameExW(
            handle,
            ptr::null_mut(),
            buffer.as_mut_ptr(),
            size as u32,
        );

        CloseHandle(handle);

        if success == 0 {
            return Err("Failed to get module filename".into());
        }

        // 找到字符串的结束位置
        let len = buffer.iter().position(|&c| c == 0).unwrap_or(size);
        let os_string = OsString::from_wide(&buffer[..len]);
        Ok(os_string.to_string_lossy().to_string())
    }
}

#[cfg(not(target_os = "windows"))]
fn get_executable_path_from_pid(pid: u32) -> Result<String, Box<dyn std::error::Error>> {
    use std::fs;
    use std::path::Path;

    // 在Unix-like系统上，可以通过/proc文件系统获取可执行文件路径
    let exe_path = format!("/proc/{}/exe", pid);
    
    if Path::new(&exe_path).exists() {
        // 读取符号链接以获取可执行文件的实际路径
        let path = fs::read_link(&exe_path)?;
        Ok(path.to_string_lossy().to_string())
    } else {
        // 如果无法读取/proc/<pid>/exe，返回错误
        Err(format!("Could not read executable path for PID {}", pid).into())
    }
}

// 获取系统网络连接列表
#[tauri::command]
async fn get_connections() -> Result<Vec<TcpConnection>, String> {
    let mut connections = Vec::new();

    // 设置地址族标志 (IPv4 和 IPv6)
    let af_flags = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
    // 设置协议标志 (TCP 和 UDP)
    let proto_flags = ProtocolFlags::TCP | ProtocolFlags::UDP;

    // 获取网络连接信息
    let sockets_info = get_sockets_info(af_flags, proto_flags).map_err(|e| e.to_string())?;

    // 创建系统信息实例以获取进程名称
    let mut system = System::new_all();
    // 刷新所有系统信息，确保获取最完整的进程列表
    system.refresh_all();

    for si in sockets_info {
        match si.protocol_socket_info {
            ProtocolSocketInfo::Tcp(tcp_si) => {
                let protocol = "TCP".to_string();

                // 处理本地地址和端口
                let local_addr = tcp_si.local_addr.to_string();
                let local_port = tcp_si.local_port;

                // 处理远程地址和端口
                let remote_addr = tcp_si.remote_addr.to_string();
                let remote_port = tcp_si.remote_port;

                // 获取连接状态
                let state = tcp_state_to_string(tcp_si.state);

                // 获取 PID (如果有)
                let pid = if !si.associated_pids.is_empty() {
                    Some(si.associated_pids[0]) // 取第一个关联的PID
                } else {
                    None
                };

                // 根据 PID 获取进程名称
                let process_name = if let Some(pid_val) = pid {
                    if let Some(process) = system.process((pid_val as usize).into()) {
                        Some(process.name().to_string())
                    } else {
                        // 如果无法获取进程信息，可能是内核进程或权限不足，显示特殊标识
                        Some("[KERNEL]".to_string())
                    }
                } else {
                    None
                };

                // 获取进程图标
                let icon = if let Some(pid_val) = pid {
                    if system.process((pid_val as usize).into()).is_some() {
                        get_process_icon_by_pid(pid_val)
                    } else {
                        // 内核进程的图标可以设置为特殊图标
                        None
                    }
                } else {
                    None
                };

                // 获取进程启动时间
                let start_time = if let Some(pid_val) = pid {
                    if let Some(process) = system.process((pid_val as usize).into()) {
                        Some(process.start_time())
                    } else {
                        // 内核进程的时间信息可能不可用，返回0
                        Some(0)
                    }
                } else {
                    None
                };

                connections.push(TcpConnection {
                    protocol,
                    local_addr,
                    local_port,
                    remote_addr,
                    remote_port,
                    state,
                    pid,
                    process_name,
                    icon, // 保持原有的图标数据
                    start_time,
                    fill_column: String::new(), // 填充列，留空
                });
            },
            ProtocolSocketInfo::Udp(udp_si) => {
                let protocol = "UDP".to_string();

                // 处理本地地址和端口
                let local_addr = udp_si.local_addr.to_string();
                let local_port = udp_si.local_port;

                // UDP 没有远程地址和端口的概念，通常设置为通配符
                let remote_addr = "*".to_string();
                let remote_port = 0;

                // UDP 没有连接状态，设置为 UNCONN
                let state = "UNCONN".to_string();

                // 获取 PID (如果有)
                let pid = if !si.associated_pids.is_empty() {
                    Some(si.associated_pids[0]) // 取第一个关联的PID
                } else {
                    None
                };

                // 根据 PID 获取进程名称
                let process_name = if let Some(pid_val) = pid {
                    if let Some(process) = system.process((pid_val as usize).into()) {
                        Some(process.name().to_string())
                    } else {
                        // 如果无法获取进程信息，可能是内核进程或权限不足，显示特殊标识
                        Some("[KERNEL]".to_string())
                    }
                } else {
                    None
                };

                // 获取进程图标
                let icon = if let Some(pid_val) = pid {
                    if system.process((pid_val as usize).into()).is_some() {
                        get_process_icon_by_pid(pid_val)
                    } else {
                        // 内核进程的图标可以设置为特殊图标
                        None
                    }
                } else {
                    None
                };

                // 获取进程启动时间
                let start_time = if let Some(pid_val) = pid {
                    if let Some(process) = system.process((pid_val as usize).into()) {
                        Some(process.start_time())
                    } else {
                        // 内核进程的时间信息可能不可用，返回0
                        Some(0)
                    }
                } else {
                    None
                };

                connections.push(TcpConnection {
                    protocol,
                    local_addr,
                    local_port,
                    remote_addr,
                    remote_port,
                    state,
                    pid,
                    process_name,
                    icon, // 保持原有的图标数据
                    start_time,
                    fill_column: String::new(), // 填充列，留空
                });
            }
        }
    }

    Ok(connections)
}

// 获取进程详情
#[tauri::command]
async fn get_process_details(pid: u32) -> Result<ProcessDetails, String> {
    let mut system = System::new_all();
    system.refresh_processes();

    if let Some(process) = system.process(Pid::from_u32(pid)) {
        Ok(ProcessDetails {
            pid,
            name: process.name().to_string(),
            command_line: process.cmd().join(" "),
            executable_path: process.exe().map(|path| path.to_string_lossy().to_string()).unwrap_or_default(),
            memory_usage: process.memory(),
            cpu_usage: process.cpu_usage(),
            parent_pid: process.parent().map(|p| p.as_u32()),
            start_time: process.start_time(),
        })
    } else {
        Err(format!("Process with PID {} not found", pid))
    }
}

// 杀死进程
#[tauri::command]
async fn kill_process(pid: u32) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::process::Command;

        // 尝试使用 SIGTERM 优雅地终止进程
        let output = Command::new("kill")
            .arg("-TERM")
            .arg(pid.to_string())
            .output()
            .map_err(|e| format!("Failed to execute kill command: {}", e))?;

        if !output.status.success() {
            // 如果 SIGTERM 失败，尝试使用 SIGKILL 强制终止
            let output = Command::new("kill")
                .arg("-KILL")
                .arg(pid.to_string())
                .output()
                .map_err(|e| format!("Failed to execute kill command: {}", e))?;

            if !output.status.success() {
                return Err(format!("Failed to kill process with PID {}", pid));
            }
        }

        Ok(())
    }

    #[cfg(windows)]
    {
        use std::mem;
        use winapi::um::processthreadsapi::OpenProcess;
        use winapi::um::handleapi::CloseHandle;
        use winapi::um::winnt::PROCESS_TERMINATE;
        use winapi::um::processthreadsapi::TerminateProcess;

        unsafe {
            let handle = OpenProcess(PROCESS_TERMINATE, 0, pid);
            if handle.is_null() {
                return Err(format!("Failed to open process with PID {}", pid));
            }

            let result = TerminateProcess(handle, 1);
            CloseHandle(handle);

            if result != 0 {
                Ok(())
            } else {
                Err(format!("Failed to kill process with PID {}", pid))
            }
        }
    }
}

// 打开文件所在目录
#[tauri::command]
async fn open_folder(path: String) -> Result<(), String> {
    use std::path::Path;
    use std::process::Command;

    // 确保我们获取的是文件所在的目录，而不是文件本身
    let path_obj = Path::new(&path);
    let dir = if path_obj.is_file() {
        // 如果是文件路径，获取其父目录
        path_obj.parent().unwrap_or(Path::new(&path))
    } else {
        // 如果已经是目录路径，直接使用
        path_obj
    };

    let dir_str = dir.to_string_lossy().to_string();

    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg(&dir_str)
            .spawn()
            .map_err(|e| format!("Failed to open explorer: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(&dir_str)
            .spawn()
            .map_err(|e| format!("Failed to open finder: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(&dir_str)
            .spawn()
            .map_err(|e| format!("Failed to open file manager: {}", e))?;
    }

    Ok(())
}

// 获取应用版本号
#[tauri::command]
async fn get_app_version() -> Result<String, String> {
    let version = env!("CARGO_PKG_VERSION");
    Ok(version.to_string())
}

// 更新窗口主题的备用实现（当平台特定代码不可用时）
#[tauri::command]
async fn update_window_theme(_window: tauri::Window, _is_dark_mode: bool) -> Result<(), String> {
    // 目前，由于平台API限制，我们无法动态更改原生窗口标题栏的外观
    // 这是一个占位符函数，用于前端调用，但不执行任何操作
    Ok(())
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, get_connections, get_process_details, kill_process, open_folder, get_app_version, update_window_theme])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
