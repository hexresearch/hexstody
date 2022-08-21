import { initTabs } from "./common.js";

var tabs = [];

function preInitTabs() {
    var selectedIndex = 0;
    const tabEls = document.getElementById("tabs-ul").getElementsByTagName("li");
    for (let i = 0; i < tabEls.length; i++) {
        tabs.push(tabEls[i].id);
    }
    const selectedTab = document.getElementById("tabs-ul").getElementsByClassName("is-active");
    if (selectedTab.length != 0) {
        selectedIndex = tabs.indexOf(selectedTab[0].id)
    }
    return selectedIndex
}

async function init() {
    const selectedTab = preInitTabs();
    initTabs(tabs, () => { }, selectedTab);
};


document.addEventListener("headerLoaded", init);
