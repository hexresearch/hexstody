export async function loadTemplate(path) {
    const template = await (await fetch(path)).text();
    return Handlebars.compile(template);
}

export function formattedCurrencyValue(currency, value) {
    switch (currency) {
        case "BTC":
            const nf = new Intl.NumberFormat('en-US');
            return nf.format(value);
        default:
            return value;
    }
}