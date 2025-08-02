import { invoke } from '@tauri-apps/api/core'

export async function ping(value: string): Promise<string | null> {
  return await invoke<{value?: string}>('plugin:printer|ping', {
    payload: {
      value,
    },
  }).then((r) => (r.value ? r.value : null));
}

export async function getPrinters(): Promise<string> {
  return await invoke<string>('plugin:printer|get_printers');
}

export async function getPrinterByName(printerName: string): Promise<string> {
  return await invoke<string>('plugin:printer|get_printers_by_name', {
    printername: printerName,
  });
}
