import { setPageLang } from "./localize.js";

async function logout() {
    return await fetch("/logout").then(r => r.json());
};

async function postLang(lang){
    return await fetch("/profile/language",
    {
        method: "POST",
        body: JSON.stringify(lang)
    })
}

function alpha_to_lang(alpha){
    switch (alpha){
        case "EN": return "English";
        case "RU": return "Russian";
        default: return alpha;
    }
}

function handleLangChange(lang){
    return async function(){
        const alpha = document.getElementById("lang-span").innerText;
        if (lang !== alpha_to_lang(alpha)){
            await postLang(lang)
            location.reload()
        }
    }
}

async function init() {
    // Init dropdowns
    const lang = document.getElementById("lang-span").innerText;
    setPageLang(lang);

    var $dropdowns = getAll('.dropdown:not(.is-hoverable)');

    if ($dropdowns.length > 0) {
        $dropdowns.forEach(function ($el) {
            $el.addEventListener('click', function (event) {
                event.stopPropagation();
                $el.classList.toggle('is-active');
            });
        });

        document.addEventListener('click', function (event) {
            closeDropdowns();
        });
    }

    function closeDropdowns() {
        $dropdowns.forEach(function ($el) {
            $el.classList.remove('is-active');
        });
    }

    // Close dropdowns if ESC pressed
    document.addEventListener('keydown', function (event) {
        var e = event || window.event;
        if (e.key === "Escape") {
            closeDropdowns();
        }
    });

    // Functions

    function getAll(selector) {
        return Array.prototype.slice.call(document.querySelectorAll(selector), 0);
    }

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