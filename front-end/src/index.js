import { SpreadsheetCMP } from "./components/index.js";
import "./main.css";

const ROOT_ELEMENT = document.getElementById('root');

if( !ROOT_ELEMENT ) console.error('Could not find the root element!');

console.debug('ROOT: ', ROOT_ELEMENT);


ROOT_ELEMENT.innerHTML = `
<div class="main-layout">
    <div class="config-pannel">
        <h3>Soon</h3>
    </div>
    <div class="main-area">
        <ml-spreadsheet data-height="400"></ml-spreadsheet>
    </div>
</div>
`

// ROOT_ELEMENT.insertAdjacentElement("beforeend", document.createElement(SpreadsheetCMP));