import { initTabs } from "../common.js";

var tabs = [];

async function tabUrlHook(tabId) {
    const tab = tabId.replace("-tab", "");
    window.history.pushState("", "", `/deposit?tab=${tab}`);
}

function preInitTabs() {
    const tabIds = Array.from(document.getElementById("tabs-ul").getElementsByTagName("li"), el => el.id);
    tabs.push(...tabIds);

    const [selectedTab] = document.getElementById("tabs-ul").getElementsByClassName("is-active");
    return selectedTab !== undefined ? tabs.indexOf(selectedTab.id) : 0;
}

async function init() {
    const selectedTab = preInitTabs();
    initTabs(tabs, tabUrlHook, selectedTab);
}

document.addEventListener("headerLoaded", init);
