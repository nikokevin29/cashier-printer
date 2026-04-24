use crate::error::AppError;
use escpos::driver::Driver;
use printers::common::base::job::PrinterJobOptions;

/// Dispatch ESC/POS byte buffer to the configured printer.
///
/// Strategy (in priority order):
/// 1. OS spooler (CUPS on macOS/Linux, WinSpool on Windows) — covers
///    most properly-installed USB and network printers.
/// 2. Serial / COM port — for printers connected via RS-232 or USB-serial.
///    Uses `baud_rate` from settings (9600 for RPP02, 19200 for TM-U220 serial).
/// 3. Network (IP:port) — for Ethernet-connected thermal printers (e.g. TM-T82X LAN).
///    Standard EPSON raw port is 9100, e.g. "192.168.1.100:9100".
pub fn dispatch_print(printer_name: &str, data: &[u8], baud_rate: u32) -> Result<(), AppError> {
    if printer_name.is_empty() {
        return Err(AppError::PrinterNotFound(
            "No default printer configured".to_string(),
        ));
    }

    // Strategy 1: OS spooler
    if let Some(printer) = printers::get_printer_by_name(printer_name) {
        printer
            .print(
                data,
                PrinterJobOptions {
                    name: Some("Cashier Order"),
                    raw_properties: &[("document-format", "application/vnd.cups-raw")],
                    converter: printers::common::converters::Converter::None,
                },
            )
            .map_err(|e| AppError::Print(e.message))?;
        return Ok(());
    }

    // Strategy 2: Serial / COM port
    if printer_name.starts_with("/dev/")
        || printer_name.starts_with("COM")
        || printer_name.starts_with("com")
    {
        use escpos::driver::SerialPortDriver;
        let driver = SerialPortDriver::open(printer_name, baud_rate, None)
            .map_err(|e| AppError::Print(e.to_string()))?;
        driver
            .write(data)
            .map_err(|e| AppError::Print(e.to_string()))?;
        driver
            .flush()
            .map_err(|e| AppError::Print(e.to_string()))?;
        return Ok(());
    }

    // Strategy 3: Network IP:port (e.g. "192.168.1.100:9100")
    if printer_name.contains(':') {
        let parts: Vec<&str> = printer_name.rsplitn(2, ':').collect();
        if let (Some(port_str), Some(host)) = (parts.first(), parts.get(1)) {
            if let Ok(port) = port_str.parse::<u16>() {
                use escpos::driver::NetworkDriver;
                let driver = NetworkDriver::open(host, port, None)
                    .map_err(|e| AppError::Print(e.to_string()))?;
                driver
                    .write(data)
                    .map_err(|e| AppError::Print(e.to_string()))?;
                driver
                    .flush()
                    .map_err(|e| AppError::Print(e.to_string()))?;
                return Ok(());
            }
        }
    }

    Err(AppError::PrinterNotFound(printer_name.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_printer_name_returns_printer_not_found() {
        let err = dispatch_print("", b"data", 9600).unwrap_err();
        assert!(
            matches!(err, AppError::PrinterNotFound(_)),
            "expected PrinterNotFound, got: {err:?}"
        );
    }

    #[test]
    fn unknown_printer_name_returns_printer_not_found() {
        // A name that is not in the OS spooler, not a serial path, not a network address
        let err = dispatch_print("NonExistentPrinterXYZ_12345", b"data", 9600).unwrap_err();
        assert!(
            matches!(err, AppError::PrinterNotFound(_)),
            "expected PrinterNotFound, got: {err:?}"
        );
    }
}
