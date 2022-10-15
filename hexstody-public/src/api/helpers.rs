use hexstody_api::domain::Currency;
use rocket_dyn_templates::handlebars::{handlebars_helper};

handlebars_helper!(is_eq_string: |a: String, b: String| a == b);
handlebars_helper!(currency_name: |a: Currency| a.symbol().symbol());