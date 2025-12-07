use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct DataTableProps {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

#[component]
pub fn DataTable(props: DataTableProps) -> Element {
    rsx! {
        div { class: "overflow-x-auto",
            table { class: "w-full text-left text-xs text-gray-300",
                thead { class: "bg-gray-800",
                    tr {
                        for head in &props.headers {
                            th { class: "px-3 py-2 font-semibold", {head.clone()} }
                        }
                    }
                }
                tbody {
                    for row in &props.rows {
                        tr { class: "odd:bg-gray-900 even:bg-gray-800 border-b border-gray-700",
                            for cell in row {
                                td { class: "px-3 py-2 text-gray-400", {cell.clone()} }
                            }
                        }
                    }
                }
            }
        }
    }
}
