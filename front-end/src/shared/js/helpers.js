
import { currentMonitor, getCurrent } from '@tauri-apps/api/window';

export const CHAR_WIDTH = 10; // Char width in px;

export const COLUMN_MIN_WIDTH = 70; // in px;
export const COLUMN_MAX_WIDTH = 350;


export async function getCurrentMonitorLogicalSize() {
    let physicalSize = await getCurrent().innerSize();
    return await convertPhysicalSizeToLogical(physicalSize);
}

export async function convertPhysicalSizeToLogical(physicalSize) {
    let scaleFactor = (await currentMonitor()).scaleFactor;
    let logicalSize = physicalSize.toLogical(scaleFactor);

    console.debug("Physical monitor size: ", physicalSize);
    console.debug("Monitor scale factor: ", scaleFactor);
    console.debug('Logical monitor size: ', logicalSize);

    return logicalSize;
}