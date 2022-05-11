Handlebars.registerHelper('print_withdrawal_request', function () {
    const html =
        "<td>" + this.request_id + "</td>" +
        "<td>" + this.user_id + "</td>" +
        "<td>" + this.address + "</td>" +
        "<td>" + this.created_at + "</td>" +
        "<td>" + this.amount + "</td>" +
        "<td>" + this.status + "</td>";
    return html
})
