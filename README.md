# RustGPT 🦀✨

Welcome to the RustGPT repository! Here, you'll find a web ChatGPT clone entirely crafted using Rust and HTMX, where technology meets simplicity and performance. 🚀

- [Try the RustGPT hosted demo](https://rustgpt.bitswired.com)
- [Read the blog article](https://www.bitswired.com/en/blog/post/rustgpt-journey-rust-htmx-web-dev)

## Introduction

RustGPT is my latest experiment in cloning the abilities of OpenAI's ChatGPT. It represents the fourth iteration in a series of clones, each built with different tech stacks to evaluate their functionality in creating a ChatGPT-like application.

In this repository, you will find a Rust-based server leveraging the Axum framework combined with HTMX, providing a Rusty web development experience. From database operations to streaming responses, this project covers a broad spectrum of backend functionalities and real-time web interactions.

So, for Rust enthusiasts and web developers alike, dive in to explore a world where web development is redefined with the power of Rust!

## Features 🌟

- **Rust with Axum Framework**: A fast and reliable server that's all about performance and simplicity.
- **SQLite**: A lightweight yet powerful database for all your data persistence needs.
- **Server Sent Events (SSE)**: Real-time streaming made easy to bring life to the ChatGPT interactions.
- **HTMX**: No hefty JavaScript frameworks needed—HTMX keeps interactions snappy with simple HTML attributes.

## Tech Stack 🛠️

- [`sqlx`](https://github.com/launchbadge/sqlx): Direct and type-safe SQL queries and migrations.
- [`tera`](https://github.com/Keats/tera): A templating engine inspired by Jinja2, for rendering the HTML views.
- [`axum`](https://github.com/tokio-rs/axum): A web application framework that's easy to use and incredibly fast.

For those eyeing some client-side WASM magic, you might also want to check out [Yew](https://github.com/yewstack/yew) or [Leptos](https://github.com/LeptosProject/leptos) for more complex applications.

## Quickstart 🏁

Jump right into it by following these steps:

1. Clone the repository.
2. Create a .env

```
MIGRATIONS_PATH=db/migrations
TEMPLATES_PATH=templates
DATABASE_URL=sqlite:db/db.db
DATABASE_PATH=db/db.db
OPENAI_API_KEY=<api-key> (only necessary for tests, users will add their own keys)
```

3. Install TailwindCSS Standalone in this repository: https://tailwindcss.com/blog/standalone-cli.
4. `cargo install just`: install Just
5. `just init`: install additional tools and migrate the db
6. `just dev`: concurrently run tailwind and cargo run in watch mode
7. Open your browser and enjoy chatting with your Rust-powered ChatGPT clone (port 3000 by default)

## Contributing 🤝

Contributions are what make the open-source community an incredible place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

If you have a suggestion that would make RustGPT better, please fork the repo and create a pull request. You can also simply open an issue. Don't forget to give the project a star! Thank you again!

## Acknowledgments 🎓

Hats off to the wonderful crates and libraries that made RustGPT possible!

---

Created with 💚 by a Rustacean who believes in the power of Rust for the web! Follow the journey on [Bitswired](https://www.bitswired.com).
