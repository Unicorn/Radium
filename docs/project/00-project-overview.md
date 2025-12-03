# Radium Project Overview

> **📌 For completed work, see [01-completed.md](./01-completed.md)**  
> **🎯 For prioritized roadmap, see [02-now-next-later.md](./02-now-next-later.md)**  
> **📋 For implementation plan, see [03-implementation-plan.md](./03-implementation-plan.md)**  
> 
> **🤖 AI agents: Read [AGENT_RULES.md](../AGENT_RULES.md) before starting work**

## Vision

Radium is a next-generation agentic orchestration tool designed for developers and power users. It provides a robust and extensible framework for creating, managing, and deploying autonomous agents that can perform complex tasks. Built with Rust, Radium leverages high performance and safety to create a highly reliable and efficient backend.

## Business Model

Radium will be an open-source project, licensed under the MIT license, to encourage community adoption and contributions. The architecture will also be designed to support a commercial SaaS offering, providing a managed, scalable platform for businesses.

- **Free Tier:** Aimed at individual developers and small teams, offering a generous amount of agent execution time and workflow runs.
- **Pro Tier:** Designed for businesses and power users, offering higher limits, advanced features (e.g., team collaboration, priority support), and enterprise-grade security.

## Core Features

- **Rust-based Backend:** A high-performance, concurrent backend capable of orchestrating a large number of agents.
- **Extensible Agent Framework:** A flexible framework that allows developers to create custom agents and integrate them into the Radium ecosystem.
- **Terminal-based GUI (TUI):** A rich, interactive TUI for managing agents, workflows, and tasks directly from the command line.
- **Native Desktop Application:** A cross-platform desktop application that provides a user-friendly graphical interface for Radium.
- **Web Application:** A feature-rich web interface providing access to Radium's core functionalities from any modern browser.
- **Flexible Model Support:** Support for major off-the-shelf AI models (e.g., Claude, Gemini, Codex) as well as custom-hosted models and MCP servers.
- **Workflow Engine:** A powerful workflow engine that allows users to define complex task chains and decision trees for agents to follow.
- **Configuration and Monitoring:** Comprehensive configuration options and real-time monitoring of agents and workflows.

## Architecture

Radium's architecture will be a modular monorepo managed by Nx. This structure will enable maximum code reuse between the web and desktop applications. The project will be organized into distinct packages and applications.

The core components will be:

- **Radium Core:** The Rust-based backend that provides the core agent orchestration, workflow execution, and data management services.
- **Radium CLI:** A command-line interface for interacting with Radium Core.
- **Radium TUI:** A terminal-based GUI that provides a rich, interactive user experience.
- **GUI Apps (`/apps`):**
    - **Desktop App:** A native desktop application built with Tauri.
    - **Web App:** A web application providing a similar user experience to the desktop app.
- **Shared Packages (`/packages`):**
    - **UI Library:** A shared library of UI components (e.g., React components) used by both the web and desktop apps.
    - **API Client:** A client library for communicating with the Radium Core backend API.
    - **Shared Types:** TypeScript types and interfaces shared across the monorepo.

These components will communicate with each other through a well-defined API, allowing for a clean separation of concerns and making it easy to extend and maintain the system.
