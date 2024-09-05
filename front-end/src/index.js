import { SpreadsheetCMP } from "./components/index.js";
import "./main.css";

const ROOT_ELEMENT = document.getElementById('root');

if( !ROOT_ELEMENT ) console.error('Could not find the root element!');

console.log('Hello, World!');
console.log('ROOT: ', ROOT_ELEMENT);

ROOT_ELEMENT.insertAdjacentElement("beforeend", document.createElement(SpreadsheetCMP));