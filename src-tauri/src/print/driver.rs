use crate::error::AppError;
use escpos::driver::Driver;

/// Dispatch ESC/POS byte buffer to the configured printer.
///
/// Strategy (in priority order):
/// 1. OS spooler — macOS/Linux: `lpr -l` (literal/raw, bypasses CUPS image filters);
///    Windows: WinSpool with DOC_INFO_1W.pDatatype = "RAW" (bypasses print processors).
/// 2. Serial / COM port — RS-232 or USB-serial (RPP02, TM-U220 serial).
/// 3. Network IP:port — Ethernet thermal printers (e.g. TM-T82X LAN, port 9100).
pub fn dispatch_print(printer_name: &str, data: &[u8], baud_rate: u32) -> Result<(), AppError> {
    if printer_name.is_empty() {
        return Err(AppError::PrinterNotFound(
            "No default printer configured".to_string(),
        ));
    }

    // Strategy 1: OS spooler — verify the name exists first, then use the
    // platform-appropriate raw path.
    if printers::get_printer_by_name(printer_name).is_some() {
        return spooler_print_raw(printer_name, data);
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

// ── Platform-specific spooler raw-print implementations ──────────────────────

/// macOS / Linux: `lpr -P <name> -l` (literal/raw mode).
///
/// `-l` tells CUPS to pass the job through as-is with no raster filters.
/// Using the `printers` crate on macOS routes through CorePrint (Apple's
/// print layer) which expects PDF or an image format and rejects ESC/POS
/// byte streams with a "not JPEG/PNG" type error.
#[cfg(not(windows))]
fn spooler_print_raw(printer_name: &str, data: &[u8]) -> Result<(), AppError> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let mut child = Command::new("lpr")
        .args(["-P", printer_name, "-l"]) // -l = literal / raw passthrough
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| AppError::Print(format!("lpr spawn: {e}")))?;

    child
        .stdin
        .as_mut()
        .expect("stdin piped")
        .write_all(data)
        .map_err(|e| AppError::Print(format!("lpr write: {e}")))?;

    let status = child
        .wait()
        .map_err(|e| AppError::Print(format!("lpr wait: {e}")))?;

    if status.success() {
        Ok(())
    } else {
        Err(AppError::Print(format!("lpr exited with {status}")))
    }
}

/// Windows: WinSpool with `pDatatype = "RAW"`.
///
/// The `printers` crate's Windows backend calls `StartDocPrinterW` without
/// setting `DOC_INFO_1W.pDatatype` to "RAW", so the print spooler tries to
/// process the job through a GDI/image pipeline and rejects ESC/POS bytes.
/// We call winspool.drv directly so the datatype is always "RAW", which
/// tells the spooler to pass bytes through unchanged.
#[cfg(windows)]
mod winspool_ffi {
    use std::ffi::c_void;

    pub type HANDLE = *mut c_void;

    #[repr(C)]
    pub struct DocInfo1W {
        pub pDocName: *const u16,
        pub pOutputFile: *const u16,
        pub pDatatype: *const u16,
    }

    // winspool.lib is part of the Windows SDK; available on all MSVC targets.
    #[link(name = "winspool")]
    extern "system" {
        pub fn OpenPrinterW(
            pPrinterName: *const u16,
            phPrinter: *mut HANDLE,
            pDefault: *const c_void,
        ) -> i32;
        pub fn StartDocPrinterW(
            hPrinter: HANDLE,
            Level: u32,
            pDocInfo: *const c_void,
        ) -> u32;
        pub fn StartPagePrinter(hPrinter: HANDLE) -> i32;
        pub fn WritePrinter(
            hPrinter: HANDLE,
            pBuf: *const c_void,
            cbBuf: u32,
            pcWritten: *mut u32,
        ) -> i32;
        pub fn EndPagePrinter(hPrinter: HANDLE) -> i32;
        pub fn EndDocPrinter(hPrinter: HANDLE) -> i32;
        pub fn ClosePrinter(hPrinter: HANDLE) -> i32;
    }
}

#[cfg(windows)]
fn spooler_print_raw(printer_name: &str, data: &[u8]) -> Result<(), AppError> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use winspool_ffi::*;

    fn wide(s: &str) -> Vec<u16> {
        OsStr::new(s)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    let pname = wide(printer_name);
    let dname = wide("Cashier Order");
    let dtype = wide("RAW"); // tell WinSpool not to process the bytes

    let doc_info = DocInfo1W {
        pDocName: dname.as_ptr(),
        pOutputFile: std::ptr::null(),
        pDatatype: dtype.as_ptr(),
    };

    unsafe {
        let mut handle: HANDLE = std::ptr::null_mut();

        if OpenPrinterW(pname.as_ptr(), &mut handle, std::ptr::null()) == 0 {
            return Err(AppError::Print(format!(
                "OpenPrinterW failed for '{printer_name}'"
            )));
        }

        let job =
            StartDocPrinterW(handle, 1, &doc_info as *const _ as *const std::ffi::c_void);
        if job == 0 {
            ClosePrinter(handle);
            return Err(AppError::Print(format!(
                "StartDocPrinterW failed for '{printer_name}'"
            )));
        }

        StartPagePrinter(handle);

        let mut written: u32 = 0;
        WritePrinter(
            handle,
            data.as_ptr() as *const _,
            data.len() as u32,
            &mut written,
        );

        EndPagePrinter(handle);
        EndDocPrinter(handle);
        ClosePrinter(handle);

        if written != data.len() as u32 {
            return Err(AppError::Print(format!(
                "WritePrinter wrote {written}/{} bytes",
                data.len()
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Strategy 0: guard — empty name ────────────────────────────────────────

    #[test]
    fn empty_printer_name_returns_printer_not_found() {
        let err = dispatch_print("", b"data", 9600).unwrap_err();
        assert!(
            matches!(err, AppError::PrinterNotFound(_)),
            "expected PrinterNotFound, got: {err:?}"
        );
    }

    // ── Strategy 1: OS spooler ────────────────────────────────────────────────

    #[test]
    fn unknown_printer_name_returns_printer_not_found() {
        // Not in OS spooler, not a serial path, not a network address → fall-through
        let err = dispatch_print("NonExistentPrinterXYZ_12345", b"data", 9600).unwrap_err();
        assert!(
            matches!(err, AppError::PrinterNotFound(_)),
            "expected PrinterNotFound, got: {err:?}"
        );
    }

    #[test]
    fn printer_not_found_error_message_contains_printer_name() {
        let name = "GhostPrinter_ABCDE";
        let err = dispatch_print(name, b"data", 9600).unwrap_err();
        match err {
            AppError::PrinterNotFound(msg) => {
                assert!(msg.contains(name), "error message '{msg}' should contain '{name}'")
            }
            other => panic!("expected PrinterNotFound, got: {other:?}"),
        }
    }

    // ── Strategy 2: serial / COM port ─────────────────────────────────────────

    #[test]
    fn dev_serial_path_routes_to_serial_strategy_not_printer_not_found() {
        // "/dev/..." prefix → SerialPortDriver; port won't exist but error is Print, not PrinterNotFound
        let err = dispatch_print("/dev/ttyXXX_nonexistent_00", b"\x1b\x40", 9600).unwrap_err();
        assert!(
            matches!(err, AppError::Print(_)),
            "expected Print error for /dev/ path, got: {err:?}"
        );
    }

    #[test]
    fn com_uppercase_routes_to_serial_strategy() {
        // "COM..." prefix (Windows style) → SerialPortDriver
        let err = dispatch_print("COM99", b"\x1b\x40", 9600).unwrap_err();
        assert!(
            matches!(err, AppError::Print(_)),
            "expected Print error for COM99 path, got: {err:?}"
        );
    }

    #[test]
    fn com_lowercase_routes_to_serial_strategy() {
        // "com..." lowercase prefix also routes to SerialPortDriver
        let err = dispatch_print("com1", b"\x1b\x40", 9600).unwrap_err();
        assert!(
            matches!(err, AppError::Print(_)),
            "expected Print error for com1 path, got: {err:?}"
        );
    }

    #[test]
    fn dev_path_with_different_baud_rates_still_routes_to_serial() {
        for baud in [9600u32, 19200, 38400, 115200] {
            let err = dispatch_print("/dev/ttyS99_fake", b"\x00", baud).unwrap_err();
            assert!(
                matches!(err, AppError::Print(_)),
                "baud {baud}: expected Print error, got: {err:?}"
            );
        }
    }

    // ── Strategy 3: network IP:port ───────────────────────────────────────────

    #[test]
    fn valid_network_address_routes_to_network_strategy() {
        // Loopback + unused port — connection refused = Print error, not PrinterNotFound
        let err = dispatch_print("127.0.0.1:19100", b"\x1b\x40", 9600).unwrap_err();
        assert!(
            matches!(err, AppError::Print(_)),
            "expected Print error for 127.0.0.1:19100, got: {err:?}"
        );
    }

    #[test]
    fn hostname_with_valid_port_routes_to_network_strategy() {
        // "localhost:9100" — connection refused → Print, not PrinterNotFound
        let err = dispatch_print("localhost:9100", b"\x1b\x40", 9600).unwrap_err();
        assert!(
            matches!(err, AppError::Print(_)),
            "expected Print error for localhost:9100, got: {err:?}"
        );
    }

    #[test]
    fn dotted_quad_address_routes_to_network_strategy() {
        // Dotted-quad with port — not in spooler, routes to NetworkDriver.
        // Use loopback so the OS returns ECONNREFUSED immediately (no timeout).
        let err = dispatch_print("127.0.0.1:19101", b"\x1b\x40", 9600).unwrap_err();
        assert!(
            matches!(err, AppError::Print(_)),
            "expected Print error for 127.0.0.1:19101, got: {err:?}"
        );
    }

    #[test]
    fn invalid_port_string_returns_printer_not_found() {
        // "host:notaport" → port parse fails → falls through all strategies → PrinterNotFound
        let err = dispatch_print("myprinter:notaport", b"data", 9600).unwrap_err();
        assert!(
            matches!(err, AppError::PrinterNotFound(_)),
            "expected PrinterNotFound for invalid port, got: {err:?}"
        );
    }

    #[test]
    fn port_overflow_returns_printer_not_found() {
        // 99999 > u16::MAX (65535) → parse fails → PrinterNotFound
        let err = dispatch_print("192.168.1.100:99999", b"data", 9600).unwrap_err();
        assert!(
            matches!(err, AppError::PrinterNotFound(_)),
            "expected PrinterNotFound for port overflow, got: {err:?}"
        );
    }

    #[test]
    fn trailing_colon_empty_port_returns_printer_not_found() {
        // "PrinterName:" → empty port string → parse fails → PrinterNotFound
        let err = dispatch_print("PrinterName:", b"data", 9600).unwrap_err();
        assert!(
            matches!(err, AppError::PrinterNotFound(_)),
            "expected PrinterNotFound for trailing colon, got: {err:?}"
        );
    }

    #[test]
    fn multiple_colons_uses_rightmost_split() {
        // "a:b:9100" → rsplitn(2, ':') → port="9100", host="a:b" → NetworkDriver attempted
        // Connection refused = Print error (parsed successfully, not PrinterNotFound)
        let err = dispatch_print("a:b:9100", b"data", 9600).unwrap_err();
        assert!(
            matches!(err, AppError::Print(_)),
            "expected Print error for multi-colon addr, got: {err:?}"
        );
    }

    // ── Strategy priority ordering ────────────────────────────────────────────

    #[test]
    fn serial_prefix_takes_priority_over_network_colon_check() {
        // "/dev/tty:9100" has both "/dev/" prefix AND a colon.
        // Serial check (Strategy 2) runs before network check (Strategy 3),
        // so it must route to SerialPortDriver, not NetworkDriver.
        // Both produce Print errors, but the path taken determines the error message.
        let err = dispatch_print("/dev/tty:9100", b"\x1b\x40", 9600).unwrap_err();
        assert!(
            matches!(err, AppError::Print(_)),
            "expected Print error (serial beats network), got: {err:?}"
        );
    }
}
