export interface Order {
  id: number;
  customer_name: string;
  content: string;
  created_at: string;
}

export interface AppSettings {
  default_printer: string;
  paper_size: '58mm' | '75mm' | '80mm';
  store_name: string;
  footer_text: string;
  /** Baud rate for serial/COM port connections. Ignored for CUPS/network. */
  serial_baud_rate: number;
  /** Send auto-cut command after each receipt. Disable for TM-U220 without cutter. */
  auto_cut: boolean;
  /** Workstation display name shown in history and at the bottom of receipts. */
  pc_name: string;
}

export interface PrinterInfo {
  name: string;
  is_default: boolean;
  connection_type: string;
}

export interface AppError {
  type: 'Database' | 'Print' | 'PrinterNotFound' | 'Settings' | 'NotFound';
  message: string;
}
