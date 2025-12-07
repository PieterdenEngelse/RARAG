use crate::components::SearchBar;
use dioxus::prelude::*;

#[component]
pub fn Home() -> Element {
    rsx! {
        div {
            class: "min-h-screen bg-gray-50 dark:bg-gray-900 py-8",

            div {
                class: "max-w-6xl mx-auto px-4",

                // Page header
                div {
                    class: "text-center mb-8",

                    h1 {
                        class: "text-4xl font-bold text-gray-900 dark:text-white mb-2",
                        "Rust Agentic RAG"
                    }

                    p {
                        class: "text-gray-600 dark:text-gray-400",
                        "Retrieval Augmented Generation powered by Rust"
                    }
                }

                // Search component
                SearchBar {}

                // Info section
                div {
                    class: "mt-12 grid grid-cols-1 md:grid-cols-3 gap-6",

                    div {
                        class: "p-6 bg-white dark:bg-gray-800 rounded-lg shadow",

                        div {
                            class: "text-3xl mb-2",
                            "📚"
                        }

                        h3 {
                            class: "text-lg font-semibold text-gray-900 dark:text-white mb-2",
                            "Document Search"
                        }

                        p {
                            class: "text-sm text-gray-600 dark:text-gray-400",
                            "Search through your indexed documents using semantic similarity"
                        }
                    }

                    div {
                        class: "p-6 bg-white dark:bg-gray-800 rounded-lg shadow",

                        div {
                            class: "text-3xl mb-2",
                            "🔍"
                        }

                        h3 {
                            class: "text-lg font-semibold text-gray-900 dark:text-white mb-2",
                            "Tantivy Index"
                        }

                        p {
                            class: "text-sm text-gray-600 dark:text-gray-400",
                            "Fast full-text search powered by Tantivy search engine"
                        }
                    }

                    div {
                        class: "p-6 bg-white dark:bg-gray-800 rounded-lg shadow",

                        div {
                            class: "text-3xl mb-2",
                            "🤖"
                        }

                        h3 {
                            class: "text-lg font-semibold text-gray-900 dark:text-white mb-2",
                            "LLM Integration"
                        }

                        p {
                            class: "text-sm text-gray-600 dark:text-gray-400",
                            "Local language model for summarization and reranking"
                        }
                    }
                }
            }
        }
    }
}
