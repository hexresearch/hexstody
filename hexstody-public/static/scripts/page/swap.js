import { getAllCurrencies } from "./common.js";



async function init() {
    // Init dropdowns
    const x = [...getAllCurrencies()];

    const templ = Handlebars.compile('<a href="#" class="dropdown-item" dropdown-id> {{this}} </a>');
    const g = x.reduce((acc, e) => acc + (templ(e)), "");
    document.getElementById("currency-from").innerHTML = g;
    document.getElementById("currency-to").innerHTML = g;

    const k = Array.from(document.getElementById("currency-from").getElementsByClassName("dropdown-item"));

    for (const k1 of k) {
          k1.addEventListener("click", event => {
            document.getElementById("currency-selection").innerText = event.target.innerText;
          });
    }

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
}

document.addEventListener("DOMContentLoaded", init);