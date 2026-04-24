pub mod builder;
pub mod driver;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrinterInfo {
    pub name: String,
    pub is_default: bool,
    pub connection_type: String,
}

/// List all available printers from the OS spooler and serial ports.
pub fn list_available_printers() -> Vec<PrinterInfo> {
    let mut results: Vec<PrinterInfo> = Vec::new();

    // 1. OS-registered printers (CUPS on macOS/Linux, WinSpool on Windows)
    for p in printers::get_printers() {
        results.push(PrinterInfo {
            name: p.name.clone(),
            is_default: p.is_default,
            connection_type: "cups".to_string(),
        });
    }

    // 2. Serial ports (for printers connected via RS-232/USB-serial)
    if let Ok(ports) = serialport::available_ports() {
        for port in ports {
            // Skip ports already represented by a CUPS entry
            let already_listed = results.iter().any(|r| r.name == port.port_name);
            if !already_listed {
                results.push(PrinterInfo {
                    name: port.port_name,
                    is_default: false,
                    connection_type: "serial".to_string(),
                });
            }
        }
    }

    results
}
