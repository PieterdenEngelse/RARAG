use dioxus::prelude::*;
use std::borrow::Cow;

#[derive(Props, Clone, PartialEq)]
pub struct StatCardProps {
    pub title: Cow<'static, str>,
    pub value: Cow<'static, str>,
    #[props(default)]
    pub unit: Option<Cow<'static, str>>,
    #[props(default)]
    pub trend: Option<Cow<'static, str>>,
    #[props(default)]
    pub sparkline: Option<Vec<f64>>,
    #[props(default)]
    pub footer: Option<VNode>,
}

#[component]
pub fn StatCard(props: StatCardProps) -> Element {
    rsx! {
        div { class: "rounded p-4 bg-gray-800 border border-gray-700 relative",
            div { class: "text-xs text-gray-400", {props.title.clone()} }
            div { class: "text-2xl font-bold text-gray-100", {props.value.clone()} }
            if let Some(unit) = &props.unit {
                span { class: "text-sm text-gray-500", {unit.clone()} }
            }
            if let Some(trend) = &props.trend {
                div { class: "text-xs text-gray-500", {trend.clone()} }
            }
            if let Some(points) = &props.sparkline {
                div { class: "text-[10px] text-gray-600", "sparkline: {points.len()} pts" }
            }
            if let Some(footer) = &props.footer {
                div { class: "mt-2", {footer.clone()} }
            }
        }
    }
}
