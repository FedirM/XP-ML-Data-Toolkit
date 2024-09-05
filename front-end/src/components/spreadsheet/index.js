import css from "!!raw-loader!./index.raw.css";
import { Cell } from "./cell";
import { getCurrentMonitorLogicalSize, CHAR_HEIGHT, CHAR_WIDTH } from "../../shared/js/helpers";


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

        this.spreadsheetData = [
            ["Company Name", "Country", "Name", "Sell date", "Order Identificator (just kek)", "B"],
            ["Tagcat", "United Kingdom", "Classic Vest", "11/10/2020", "01-2331942", "true"],
            ["Tagcat", "United Kingdom", "Classic Vest", "11/10/2020", "01-2331942", "true"],
            ["Tagcat", "United Kingdom", "Classic Vest", "11/10/2020", "01-2331942", "true"],
            ["Tagcat", "United Kingdom", "Classic Vest", "11/10/2020", "01-2331942", "true"],
            ["Tagcat", "United Kingdom", "Classic Vest", "11/10/2020", "01-2331942", "true"],
            ["Tagcat", "United Kingdom", "Classic Vest", "11/10/2020", "01-2331942", "true"],
            ["Tagcat", "United Kingdom", "Classic Vest", "11/10/2020", "01-2331942", "true"],
            ["Tagcat", "United Kingdom", "Classic Vest", "11/10/2020", "01-2331942", "true"],
            ["Tagcat", "United Kingdom", "Classic Vest", "11/10/2020", "01-2331942", "true"],
            ["Tagcat", "United Kingdom", "Classic Vest", "11/10/2020", "01-2331942", "true"]
        ];

        console.log('Calc size call...\n');
        this.calcSize().then((size) => {
            console.log('Size: ', size);
            let containerHTML = this.shadowRoot.getElementById('sp-container');
            containerHTML.innerHTML = '';

            this.spreadsheetData.forEach((rowData, rowId) => {
                let rowHTML = document.createElement("div");
                rowHTML.classList.add('spreadsheet-row');
                let rowTmpl = rowData.map((el, i) => {
                    let s = size[i];
                    let c = new Cell(rowId, i, el, s.width, s.height, "center");
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

        // TODO: recalc according real width according window width.

        for(let i = 0; i < this.spreadsheetData.length; i++) {

        }

        let min = [];
        let totalWidth = 0;

        for(let h of this.spreadsheetData[0]) {
            let currWidth = h.split('').length * CHAR_WIDTH;
            console.debug("Curr width: ", currWidth);
            totalWidth += currWidth;
            min.push({
                height: CHAR_HEIGHT,
                width: Math.max(currWidth, 70)
            });
        }

        let maxWidth = (await getCurrentMonitorLogicalSize()).width;
        console.debug('sp-container width: ', maxWidth);
        let size = min;

        if(totalWidth !== maxWidth) {
            let ratio = [];
            let percent = totalWidth / 100;
            let offset = maxWidth - totalWidth;

            console.debug(`Percent: ${percent}\nOffset: ${offset}`);

            for(let i = 0; i < min.length; i++) {
                ratio.push( min[i].width / percent );
                size[i].width += (offset / 100) * ratio[i];
            }

            console.debug('RATIO: ', ratio);
            console.debug('Min: ', min);
        }

        return size;
    }
}