use std::net::TcpStream;
use std::time::Duration;
use std::{fs, process::exit};

/**
 * Simple lib of basic functions to detect a sandbox or virtual environment.
 * Will maybe expand as the project progresses.
 */

pub fn check_environment() {
    let cpu_core_count = test_cpu_core_count();
    let vm_hardware = detect_vm_hardware();
    let vm_mac = check_vm_mac();
    let processes = check_processes();
    let debugged = is_debugged();
    let vm = is_vm();
    let internet_missing = !has_internet_conn();

    eprintln!(
        "anti_analysis: cpu_core_count={cpu_core_count} vm_hardware={vm_hardware} vm_mac={vm_mac} processes={processes} debugged={debugged} vm={vm} internet_missing={internet_missing}"
    );

    if cpu_core_count || vm_hardware || vm_mac || processes || debugged || vm || internet_missing {
        exit(0);
    }
}

fn has_internet_conn() -> bool {
    TcpStream::connect_timeout(&"8.8.8.8:53".parse().unwrap(), Duration::from_secs(3)).is_ok()
}

fn test_cpu_core_count() -> bool {
    std::thread::available_parallelism()
        .map(|n| n.get() < 2)
        .unwrap_or(false)
}

fn detect_vm_hardware() -> bool {
    // check for hypervisor bit in cpu id
    #[cfg(target_arch = "x86_64")]
    unsafe {
        let ecx: u32;
        std::arch::asm!(
            "mov eax, 1",
            "cpuid",
            out("eax") _,
            out("ecx") ecx,
            out("edx") _,
            options(preserves_flags, nostack)
        );

        // bit 31 of ecx indicates hypervisor
        if (ecx & (1 << 31)) != 0 {
            return true;
        }
    }

    #[cfg(target_os = "linux")]
    {
        // check dmi
        if let Ok(vendor) = std::fs::read_to_string("/sys/class/dmi/id/sys_vendor") {
            let lower = vendor.to_lowercase();
            if lower.contains("qemu")
                || lower.contains("vmware")
                || lower.contains("virtualbox")
                || lower.contains("bochs")
                || lower.contains("xen")
                || lower.contains("kvm")
            {
                return true;
            }
        }

        // check for vm devices
        let vm_devices = ["/dev/vboxguest", "/dev/vboxuser", "/proc/vz", "/proc/xen"];

        if vm_devices
            .iter()
            .any(|path| std::path::Path::new(path).exists())
        {
            return true;
        }
    }

    #[cfg(target_os = "windows")]
    {
        // check registry for VM indicators
        if let Ok(output) = std::process::Command::new("reg")
            .args([
                "query",
                "HKLM\\SYSTEM\\CurrentControlSet\\Services\\Disk\\Enum",
                "/v",
                "0",
            ])
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout).to_lowercase();
            if output_str.contains("vmware")
                || output_str.contains("vbox")
                || output_str.contains("qemu")
                || output_str.contains("virtual")
            {
                return true;
            }
        }

        // check BIOS info
        if let Ok(output) = std::process::Command::new("wmic")
            .args(["bios", "get", "serialnumber"])
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout).to_lowercase();
            if output_str.contains("vmware")
                || output_str.contains("vbox")
                || output_str.contains("qemu")
            {
                return true;
            }
        }
    }

    false
}

fn check_vm_mac() -> bool {
    // known mac oui prefixes
    let vm_mac_prefixes = [
        "00:05:69", "00:0c:29", "00:1c:14", "00:50:56", // vmware
        "08:00:27", // virtualbox
        "00:16:3e", // xen
        "52:54:00", // qemu/kvm
        "00:03:ff", // microsoft hyper-v
    ];

    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = std::process::Command::new("ip")
            .args(["link", "show"])
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout).to_lowercase();

            for prefix in &vm_mac_prefixes {
                if output_str.contains(prefix) {
                    return true;
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(output) = std::process::Command::new("getmac").output() {
            let output_str = String::from_utf8_lossy(&output.stdout).to_lowercase();

            for prefix in &vm_mac_prefixes {
                // Windows uses - instead of :
                let windows_prefix = prefix.replace(':', "-");
                if output_str.contains(&windows_prefix) {
                    return true;
                }
            }
        }
    }

    false
}

fn check_processes() -> bool {
    #[cfg(target_os = "linux")]
    let suspicious = [
        "strace",
        "ltrace",
        "gdb",
        "valgrind",
        "wireshark",
        "tcpdump",
        "qemu",
        "vboxservice",
        "vmtoolsd",
        "x64dbg",
        "ollydbg",
    ];

    #[cfg(target_os = "windows")]
    let suspicious = [
        "wireshark",
        "procmon",
        "processhacker",
        "procexp",
        "x64dbg",
        "x32dbg",
        "ollydbg",
        "windbg",
        "vboxservice",
        "vmtoolsd",
        "vmsrvc",
        "vmusrvc",
    ];

    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = std::process::Command::new("ps").args(["aux"]).output() {
            let output_str = String::from_utf8_lossy(&output.stdout).to_lowercase();
            for proc in &suspicious {
                if output_str.contains(proc) {
                    return true;
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(output) = std::process::Command::new("tasklist").output() {
            let output_str = String::from_utf8_lossy(&output.stdout).to_lowercase();
            for proc in &suspicious {
                if output_str.contains(proc) {
                    return true;
                }
            }
        }
    }

    false
}

#[cfg(target_os = "linux")]
fn is_debugged() -> bool {
    if let Ok(status) = fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            if !line.starts_with("TracerPid:") {
                continue;
            };

            if let Some(pid) = line.split_whitespace().nth(1) {
                if pid != "0" {
                    return true;
                }
            }
        }
    }

    unsafe {
        let ret = libc::ptrace(libc::PTRACE_TRACEME, 0, 0, 0);
        if ret == -1 {
            // already being traced
            return true;
        }
        libc::ptrace(libc::PTRACE_DETACH, 0, 0, 0);
    }

    if std::env::var("LD_PRELOAD").is_ok() {
        return true;
    }

    false
}

#[cfg(target_os = "windows")]
fn is_debugged() -> bool {
    // check PEB BeingDebugged flag
    #[cfg(target_arch = "x86_64")]
    unsafe {
        let being_debugged: u8;
        std::arch::asm!(
            "mov rax, qword ptr gs:[0x60]",  // PEB address
            "mov {0}, byte ptr [rax + 0x02]", // BeingDebugged flag
            out(reg_byte) being_debugged,
            out("rax") _,
        );

        if being_debugged != 0 {
            return true;
        }
    }

    #[cfg(target_arch = "x86")]
    unsafe {
        let being_debugged: u8;
        std::arch::asm!(
            "mov eax, dword ptr fs:[0x30]",  // PEB address
            "mov {0}, byte ptr [eax + 0x02]", // BeingDebugged flag
            out(reg_byte) being_debugged,
            out("eax") _,
        );

        if being_debugged != 0 {
            return true;
        }
    }

    // check for common debugger windows
    if let Ok(output) = std::process::Command::new("tasklist").args(["/v"]).output() {
        let output_str = String::from_utf8_lossy(&output.stdout).to_lowercase();
        let debuggers = ["x64dbg", "x32dbg", "ollydbg", "ida", "windbg", "devenv"];
        for debugger in &debuggers {
            if output_str.contains(debugger) {
                return true;
            }
        }
    }

    false
}

#[cfg(target_os = "linux")]
fn is_vm() -> bool {
    let indicators = [
        "/proc/scsi/scsi",
        "/sys/class/dmi/id/product_name",
        "/sys/class/dmi/id/sys_vendor",
    ];

    for path in &indicators {
        if let Ok(content) = fs::read_to_string(path) {
            let lower = content.to_lowercase();
            if lower.contains("vmware")
                || lower.contains("virtualbox")
                || lower.contains("qemu")
                || lower.contains("kvm")
            {
                return true;
            }
        }
    }

    false
}

#[cfg(target_os = "windows")]
fn is_vm() -> bool {
    // check for VM-specific registry keys
    let reg_checks = [
        ("HKLM\\SYSTEM\\CurrentControlSet\\Services", "VBoxGuest"),
        ("HKLM\\SYSTEM\\CurrentControlSet\\Services", "VBoxMouse"),
        ("HKLM\\SYSTEM\\CurrentControlSet\\Services", "VBoxSF"),
        ("HKLM\\SYSTEM\\CurrentControlSet\\Services", "vmhgfs"),
        ("HKLM\\SYSTEM\\CurrentControlSet\\Services", "vmmouse"),
        ("HKLM\\SYSTEM\\CurrentControlSet\\Services", "VMTools"),
    ];

    for (key, _) in &reg_checks {
        if let Ok(output) = std::process::Command::new("reg")
            .args(["query", key])
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout).to_lowercase();
            if output_str.contains("vbox")
                || output_str.contains("vmware")
                || output_str.contains("qemu")
                || output_str.contains("virtual")
            {
                return true;
            }
        }
    }

    // check system manufacturer using wmic
    if let Ok(output) = std::process::Command::new("wmic")
        .args(["computersystem", "get", "manufacturer"])
        .output()
    {
        let output_str = String::from_utf8_lossy(&output.stdout).to_lowercase();
        if output_str.contains("vmware")
            || output_str.contains("virtualbox")
            || output_str.contains("qemu")
            || output_str.contains("microsoft corporation")
        {
            return true;
        }
    }

    // check model
    if let Ok(output) = std::process::Command::new("wmic")
        .args(["computersystem", "get", "model"])
        .output()
    {
        let output_str = String::from_utf8_lossy(&output.stdout).to_lowercase();
        if output_str.contains("virtualbox")
            || output_str.contains("vmware")
            || output_str.contains("virtual")
        {
            return true;
        }
    }

    false
}
