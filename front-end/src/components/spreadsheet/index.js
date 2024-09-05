import css from "!!raw-loader!./index.raw.css";
import { Cell } from "./cell";
import { getCurrentMonitorLogicalSize, CHAR_WIDTH, COLUMN_MIN_WIDTH, COLUMN_MAX_WIDTH } from "../../shared/js/helpers";

import { appWindow } from "@tauri-apps/api/window";

const template = document.createElement("template");
template.innerHTML = `
<style>${css}</style>
<div id="sp-container" class="spreadsheet-container"></div>
`;

export const SpreadsheetComponentName = "ml-spreadsheet";
export class SpreadsheetComponent extends HTMLElement {


    constructor() {
        super();

        const shadow = this.attachShadow({ mode: "open" });
        shadow.appendChild(template.content.cloneNode(true));

        this.spreadsheetData = [];

        this._rerender();

        appWindow.onResized(() => {
            this._rerender();
        }).then().catch(console.error);
    }

    _rerender() {
        console.log('Calc rerender...\n');
        this.calcSize().then((size) => {
            console.debug('Size: ', size);
            console.debug('Total width: ', size.reduce((a, v) => a + v, 0));
            let containerHTML = this.shadowRoot.getElementById('sp-container');
            containerHTML.innerHTML = '';

            this.spreadsheetData.forEach((rowData, rowId) => {
                let rowHTML = document.createElement("div");
                if(rowId === 0) {
                    rowHTML.classList.add("sticky");
                }
                rowHTML.classList.add('spreadsheet-row');
                let rowTmpl = rowData.map((el, i) => {
                    let w = size[i];
                    let c = new Cell(rowId, i, el, w, "center");
                    rowHTML.appendChild(c.htmlTemplate());
                });
                let html = document.createElement('div');
                html.innerHTML = rowTmpl.join('\n');
                
                containerHTML.appendChild(rowHTML);
            });
        }).catch(e => console.error(e));
    }

    async calcSize() {
        if(!this.spreadsheetData.length || !this.spreadsheetData[0].length) {
            console.error("There is no spreadsheet data found!");
            return null;
        }

        let chWidth = [];

        for(let i = 0; i < this.spreadsheetData.length; i++) {
            for(let j = 0; j < this.spreadsheetData[0].length; j++) {
                let currWidth = this.spreadsheetData[i][j].split('').length * CHAR_WIDTH;
                if( i === 0 ) {
                    chWidth.push(Math.min(currWidth, COLUMN_MIN_WIDTH));
                } else {
                    chWidth[j] = Math.min(Math.max(currWidth, chWidth[j]), COLUMN_MAX_WIDTH);
                }

            }
        }

        
        let logicalSize = await getCurrentMonitorLogicalSize();
        let scaleFactor = (logicalSize.width - 15) / chWidth.reduce((acc, val) => acc + val, 0);

        return chWidth.map((el) => scaleFactor * Math.floor(el));
    }
}