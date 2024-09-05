
import { currentMonitor, getCurrent } from '@tauri-apps/api/window';

export const CHAR_WIDTH = 10; // Char width in px;
export const CHAR_HEIGHT = 18; // Char height in px;


export async function getCurrentMonitorLogicalSize() {
    let physicalSize = await getCurrent().innerSize();
    let scaleFactor = (await currentMonitor()).scaleFactor;
    let logicalSize = physicalSize.toLogical(scaleFactor);

    console.debug("Physical monitor size: ", physicalSize);
    console.debug("Monitor scale factor: ", scaleFactor);
    console.debug('Logical monitor size: ', logicalSize);

    return logicalSize;
}