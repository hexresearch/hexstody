import { initDropDowns } from "../common.js";
import { setPageLang } from "../localize.js";

async function logout() {
    return await fetch("/logout").then(r => r.json());
}

async function postLang(lang){
    return await fetch("/profile/language",
    {
        method: "POST",
        body: JSON.stringify(lang)
    })
}

export function alphaToLang(alpha){
    switch (alpha) {
        case "EN": return "English";
        case "RU": return "Russian";
        default: return alpha;
    }
}

function handleLangChange(lang){
    return async function(){
        const alpha = document.getElementById("lang-span").innerText;
        if (lang !== alphaToLang(alpha)){
            await postLang(lang)
            location.reload()
        }
    }
}

async function init() {
    initDropDowns();

    const lang = document.getElementById("lang-span").innerText;
    setPageLang(lang);

    const logoutBtn = document.getElementById("logout-btn");
    logoutBtn.onclick = logout;

    const enEl = document.getElementById("lang-en");
    const ruEl = document.getElementById("lang-ru");
    enEl.onclick = handleLangChange("English");
    ruEl.onclick = handleLangChange("Russian");
    const event = new Event('headerLoaded');
    document.dispatchEvent(event);
}

document.addEventListener("DOMContentLoaded", init);