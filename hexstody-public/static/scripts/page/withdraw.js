import { initTabs } from "./common.js";

async function init() {
    const tabs = ["btc-tab", "eth-tab"];
    initTabs(tabs);
}

document.addEventListener("DOMContentLoaded", init);