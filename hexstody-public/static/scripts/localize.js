var lang = "en"

export function setPageLang(language) {
    const l = language.toLowerCase()
    if (l === "en" || l === "ru") {
        lang = l
    }
}

export function getLang() {
    const langSpan = document.getElementById("lang-span")
    if (langSpan) {
        return langSpan.innerText.toLowerCase()
    } else {
        return "en"
    }
}

export function localizeSpan(span) {
    switch (lang) {
        case "en": return span
        case "ru": switch (span) {
            case "Day": return "день"
            case "Week": return "неделю"
            case "Month": return "месяц"
        }
    }
}

export function localizeChangeStatus(status) {
    switch (Object.keys(status)[0]) {
        case "InProgress":
            let body = status["InProgress"]
            switch (lang) {
                case "en": return "In progress (+" + body.confirmations + "/-" + body.rejections + " of 2)"
                case "ru": return "В процессе (+" + body.confirmations + "/-" + body.rejections + " из 2)"
            }
        case "Confirmed":
        case "en": return "Confirmed"
        case "ru": return "Принято"
        case "Rejected":
        case "en": return "Rejected by operators"
        case "ru": return "Отвергнуто операторами"
        default:
            "Unknown"
    };
}

export function localizeWithdrawalStatus(status) {
    switch (lang) {
        case "en": switch (status.type) {
            case "InProgress":
                return "In progress"
            case "Confirmed":
                return "Confirmed"
            case "Completed":
                return "Completed"
            case "OpRejected":
                return "Rejected by operators"
            case "NodeRejected":
                return "Rejected by node"
            default:
                return "Unknown"
        };
        case "ru": switch (status.type) {
            case "InProgress":
                return "В процессе"
            case "Confirmed":
                return "Подтверждено"
            case "Completed":
                return "Завершено"
            case "OpRejected":
                return "Отклонено оператором"
            case "NodeRejected":
                return "Отклонено нодой"
            default:
                return "Unknown"
        };
    }
}

export function getLanguage() {
    return lang
}

function languageCodeToLanguage(language) {
    switch (language.toLowerCase()) {
        case "ru":
        case "ru-ru":
            return "ru"
        case "en":
        case "en-us":
        case "en-gb":
        default:
            return "en"
    };
}

export function getBrowserLanguage() {
    let browserLanguage = navigator.language || navigator.userLanguage
    return languageCodeToLanguage(browserLanguage)
}
