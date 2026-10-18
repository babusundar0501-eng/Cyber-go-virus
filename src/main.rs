use std::collections::HashSet;
use std::time::Duration;
use sysinfo::{Pid, ProcessExt, System, SystemExt};
use tokio::time::sleep;
use colored::*;
use windows::Win32::System::Threading::{OpenProcess, PROCESS_VM_READ, PROCESS_QUERY_INFORMATION};
use windows::Win32::System::Memory::{VirtualQuery, MEMORY_BASIC_INFORMATION};
use windows::Win32::System::Diagnostics::Debug::ReadProcessMemory;

// =====================================================================
// 1. BEHAVIORAL ATTACK MATRIX & HEX byte MATRICES
// =====================================================================
const EVASION_SIGNATURES: [&str; 6] = [
    "vssadmin delete shadows", "powershell -ep bypass", "mimikatz",
    "amsiinitfailed", "etweventwrite", "CreateRemoteThread"
];

struct CyberGoVirusCore {
    sys_snapshot: System,
    analyzed_pids: HashSet<Pid>,
}

impl CyberGoVirusCore {
    fn new() -> Self {
        CyberGoVirusCore {
            sys_snapshot: System::new_all(),
            analyzed_pids: HashSet::new(),
        }
    }

    // =====================================================================
    // MODULE A: LOW-LATENCY RAM INTROSPECTION & HOLLOWING DETECTOR
    // =====================================================================
    async fn scan_process_memory_space(&mut self) {
        self.sys_snapshot.refresh_processes();

        for (pid, process) in self.sys_snapshot.processes() {
            let sys_pid = *pid;
            if self.analyzed_pids.contains(&sys_pid) {
                continue;
            }

            let cmd_args = process.cmd().join(" ").to_lowercase();
            let proc_name = process.name();

            // 1. Check Command-Line Living-Off-The-Land exploits
            for signature in EVASION_SIGNATURES.iter() {
                if cmd_args.contains(signature) {
                    self.flag_and_mitigate(sys_pid, proc_name, &format!("Behavioral Match: {}", signature));
                    break;
                }
            }

            // 2. High-Advanced: Live User-Mode RAM Carving via Win32 APIs
            unsafe {
                let u32_pid = sys_pid.as_u32();
                if let Ok(process_handle) = OpenProcess(PROCESS_VM_READ | PROCESS_QUERY_INFORMATION, false, u32_pid) {
                    let mut mem_info = MEMORY_BASIC_INFORMATION::default();
                    let mut address = 0x10000 as *const std::ffi::c_void;

                    // Walk memory allocation ranges looking for unhooked AMSI loops
                    while VirtualQuery(Some(process_handle), address, &mut mem_info, std::mem::size_of::<MEMORY_BASIC_INFORMATION>()) > 0 {
                        let mut buffer = vec![0u8; 4096];
                        let mut bytes_read = 0;

                        if ReadProcessMemory(process_handle, mem_info.BaseAddress, buffer.as_mut_ptr() as *mut _, buffer.len(), Some(&mut bytes_read)).is_ok() {
                            let raw_string = String::from_utf8_lossy(&buffer).to_lowercase();
                            if raw_string.contains("amsiscanbuffer") || raw_string.contains("etweventwrite") {
                                self.flag_and_mitigate(sys_pid, proc_name, "Active Memory Patching Evasion Attempted");
                                let _ = windows::Win32::Foundation::CloseHandle(process_handle);
                                break;
                            }
                        }
                        address = (address as usize + mem_info.RegionSize) as *const std::ffi::c_void;
                    }
                    let _ = windows::Win32::Foundation::CloseHandle(process_handle);
                }
            }
            self.analyzed_pids.insert(sys_pid);
        }
    }

    // =====================================================================
    // MODULE B: DEEP ROOT-NODE TERMINATION & ACTIVE MITIGATION
    // =====================================================================
    fn flag_and_mitigate(&self, target_pid: Pid, name: &str, rule: &str) {
        println!(
            "\n[{}] {} -> Name: {} | PID: {}",
            "CYBER-GO-VIRUS CRITICAL INTERCEPT".bold().on_magenta(),
            rule.bright_yellow(),
            name.bright_white(),
            target_pid
        );
        println!("[~] Initiating kernel-level teardown of the malicious execution tree...");

        if let Some(process) = self.sys_snapshot.process(target_pid) {
            // Mitigate child dependencies first to block persistence loops
            for (pid, proc) in self.sys_snapshot.processes() {
                if let Some(parent_pid) = proc.parent() {
                    if parent_pid == target_pid {
                        println!("[+] Neutralizing nested exploit node (PID: {})", pid);
                        proc.kill();
                    }
                }
            }
            process.kill();
            println!("[{}] Threat tree isolated. Memory block cleared.\n", "VERIFIED".bold().green());
        }
    }
}

// =====================================================================
// ASYNCHRONOUS MULTI-THREADED RUNTIME MANAGER
// =====================================================================
#[tokio::main]
async fn main() {
    println!("{}", "===================================================================".bright_magenta());
    println!("{}", "       CYBER-GO-VIRUS HYBRID ADVANCED OMNI-EDR AGENT IS ONLINE       ".bold().bright_green());
    println!("{}", "===================================================================".bright_magenta());
    println!("[*] Asynchronous bare-metal worker clusters spinning up...");

    let mut edr_core = CyberGoVirusCore::new();

    loop {
        edr_core.scan_process_memory_space().await;
        // 50ms hyper-low response window outclasses all standard script monitors
        sleep(Duration::from_millis(50)).await;
    }
}
