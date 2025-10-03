# Vibe Kanban

## Project Overview

Vibe Kanban is a specialized kanban board designed for developers who work with AI coding agents. It streamlines the process of planning, reviewing, and orchestrating tasks by providing a centralized platform for managing coding tasks and integrating with various AI coding agents.

The project is a monorepo with a Rust backend and a React/TypeScript frontend.

- **Backend:** The backend is built with Rust using the Axum framework. It handles task management, Git integration, and communication with AI coding agents.
- **Frontend:** The frontend is a single-page application built with React, TypeScript, Vite, and Tailwind CSS. It provides an intuitive kanban board interface for managing tasks and projects.

## Building and Running

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [Node.js](https://nodejs.org/) (>=18)
- [pnpm](https://pnpm.io/) (>=8)

### Installation

1.  Install dependencies:
    ```bash
    pnpm i
    ```

### Development

-   Run the development server (backend and frontend):
    ```bash
    pnpm run dev
    ```

-   Build the frontend:
    ```bash
    cd frontend
    pnpm build
    ```

-   Build the `npx` package from source:
    ```bash
    ./local-build.sh
    ```

### Testing

-   Run the npm package test script:
    ```bash
    ./test-npm-package.sh
    ```

## Development Conventions

-   **Formatting:**
    -   Rust: `cargo fmt --all`
    -   Frontend: `npm run format` in the `frontend` directory.
-   **Linting:**
    -   Rust: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
    -   Frontend: `npm run lint` in the `frontend` directory.
-   **Type Checking:**
    -   Frontend: `npm run check` in the `frontend` directory.
    -   Rust: `cargo check`
-   **Contribution:** Before contributing, it is preferred to discuss ideas and changes with the core team by creating a GitHub issue.
