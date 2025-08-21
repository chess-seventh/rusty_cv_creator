# 🦀 Rusty CV Creator

<div align="center">

[![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)](https://rustlang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=for-the-badge)](LICENSE)
[![Build Status](https://img.shields.io/github/actions/workflow/status/chess-seventh/rusty_cv_creator/.github/workflow/build.yml?branch=main&style=for-the-badge)](https://github.com/chess-seventh/rusty_cv_creator/actions)
[![codecov](https://img.shields.io/codecov/c/github/chess-seventh/rusty_cv_creator?style=for-the-badge&logo=codecov)](https://codecov.io/github/chess-seventh/rusty_cv_creator)

*✨ The blazingly fast, memory-safe CV generator that turns your LaTeX dreams into reality ✨*

[Features](#-features) • [Installation](#-installation) • [Quick Start](#-quick-start) • [Configuration](#-configuration) • [Usage](#-usage) • [Contributing](#-contributing)

</div>

---

## 📖 Overview

**Rusty CV Creator** is a powerful command-line tool written in Rust that automates the process of generating personalized CVs for job applications. Say goodbye to manually editing LaTeX templates for every application! 🎯

Instead of spending hours tweaking your CV for each position, Rusty CV Creator:
- 📋 **Templates** your CV with placeholders
- 🔄 **Replaces** job-specific information automatically
- 🎨 **Compiles** LaTeX to beautiful PDFs
- 💾 **Stores** application history in a database
- 📁 **Organizes** everything with intelligent file management

Perfect for job seekers who want to maintain consistency while customizing their applications efficiently!

## ✨ Features

### 🚀 Core Functionality
- **🎯 Smart CV Generation**: Automatically customize CVs for specific companies and positions
- **📝 LaTeX Integration**: Full LaTeX/XeLaTeX compilation support for professional-quality output  
- **🗄️ Database Management**: Track all your applications with SQLite/PostgreSQL support
- **📂 Intelligent Organization**: Automatic directory structure and file naming
- **🔍 Application Search**: Filter and find previous applications with advanced queries

### 🛠️ Technical Excellence  
- **⚡ Blazingly Fast**: Written in Rust for maximum performance
- **🛡️ Memory Safe**: Zero-cost abstractions with compile-time guarantees
- **🔧 Configurable**: INI-based configuration system
- **📊 Database Agnostic**: Support for both SQLite and PostgreSQL
- **🎨 Template-driven**: Flexible placeholder-based template system

### 🎛️ Command-Line Interface
- **🔧 CRUD Operations**: Create, read, update, and delete CV records
- **📋 Interactive Selection**: Fuzzy-finding with skim integration
- **👁️ Preview Support**: Optional PDF viewer integration
- **🏃 Dry Run Mode**: Test operations without side effects
- **📱 Cross Platform**: Works on Linux, macOS, and Windows

## 🚀 Installation

### Prerequisites

```bash
# Rust (latest stable)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# XeLaTeX (for PDF compilation)
sudo apt install texlive-xetex  # Ubuntu/Debian
brew install --cask mactex      # macOS
```

### Install from Source

```bash
git clone https://github.com/chess-seventh/rusty_cv_creator.git
cd rusty_cv_creator
cargo build --release
```

### Install from Crates.io *(coming soon)*

```bash
cargo install rusty_cv_creator
```

## ⚡ Quick Start

### 1. Initialize Configuration

Create your configuration file at `~/.config/rusty-cv-creator/rusty-cv-config.ini`:

```ini
[destination]
cv_path = "~/Documents/CVs"

[cv]  
cv_template_path = "~/Documents/cv-template"
cv_template_file = "cv.tex"

[to_replace]
position_line_to_change = "POSITION_PLACEHOLDER"
quote_line_to_change = "QUOTE_PLACEHOLDER"

[db]
db_path = "~/.config/rusty-cv-creator"
db_file = "applications.db"
engine = "sqlite"

[optional]
pdf_viewer = "evince"
```

### 2. Prepare Your Template

Create a LaTeX template with placeholders:

```latex
\documentclass{article}
\begin{document}

\section*{POSITION_PLACEHOLDER}
\textit{QUOTE_PLACEHOLDER}

% Your CV content here...

\end{document}
```

### 3. Generate Your First CV

```bash
# Create a new customized CV
rusty_cv_creator insert "ACME Corporation" "Senior Rust Developer" "Passionate about systems programming and performance"

# View the generated CV
rusty_cv_creator insert "TechCorp" "Software Engineer" "Love building reliable software" --view-generated-cv

# Dry run (test without creating files)  
rusty_cv_creator insert "StartupCo" "Backend Engineer" "Excited about scaling challenges" --dry-run
```

## 🛠️ Configuration

### Database Setup

Set your database URL in your environment or `.env` file:

```bash
# SQLite (recommended for single user)
echo "DATABASE_URL=sqlite://$HOME/.config/rusty-cv-creator/applications.db" > .env

# PostgreSQL (for advanced users)  
echo "DATABASE_URL=postgresql://user:password@localhost/cv_db" > .env
```

Initialize the database:
```bash
diesel setup
diesel migration run
```

### Template Structure

Your CV template directory should contain:
```
cv-template/
├── cv.tex              # Main template file
├── assets/              # Images, fonts, etc.
└── bibliography.bib     # References (optional)
```

## 📚 Usage

### Basic Commands

```bash
# Insert new CV application
rusty_cv_creator insert <company> <job_title> <quote>

# List all applications
rusty_cv_creator list

# Filter applications
rusty_cv_creator list --company "ACME" --job "Engineer" --date "2024"

# Remove an application (interactive selection)
rusty_cv_creator remove --company "OldCorp"

# Update application details  
rusty_cv_creator update --job "Senior Developer"
```

### Advanced Options

```bash
# Specify custom config file
rusty_cv_creator --config-ini ./custom-config.ini insert ...

# Use PostgreSQL instead of SQLite
rusty_cv_creator --engine postgres insert ...

# Don't save to database
rusty_cv_creator insert ... --save-to-database false

# View PDF after generation
rusty_cv_creator insert ... --view-generated-cv true
```

### Examples

```bash
# Generate CV for a startup position
rusty_cv_creator insert "Rocket Industries" "Rust Engineer" "Excited to build the future of space technology"

# Quick application without database storage
rusty_cv_creator insert "QuickApply Co" "Developer" "Fast application" --save-to-database false --dry-run

# Browse your application history
rusty_cv_creator list --date "2024-01"
```

## 🗂️ File Organization

Rusty CV Creator automatically organizes your files:

```
~/Documents/CVs/
├── 2024/
│   ├── 2024-01-15_ACME-Corp_Senior-Rust-Developer.pdf
│   ├── 2024-01-20_TechCorp_Software-Engineer.pdf
│   └── 2024-01-25_StartupCo_Backend-Engineer.pdf
└── 2023/
    └── 2023-12-01_OldCorp_Junior-Developer.pdf
```

## 🧪 Testing

Run the comprehensive test suite:

```bash
# Unit tests
cargo test

# Integration tests  
cargo test --test integration_tests

# With coverage
cargo tarpaulin --out Html
```

## 🏗️ Architecture

```
src/
├── main.rs              # Application entry point
├── cli_structure.rs     # Command-line interface definitions
├── config_parse.rs      # Configuration file parsing
├── database.rs          # Database operations (Diesel)
├── file_handlers.rs     # File system operations
├── global_conf.rs       # Global state management
├── helpers.rs           # Utility functions
├── user_action.rs       # Core business logic
├── models.rs           # Database models
└── schema.rs           # Database schema
```

## 🤝 Contributing

We love contributions! Here's how you can help:

### 🐛 Found a Bug?
Open an [issue](https://github.com/chess-seventh/rusty_cv_creator/issues) with:
- Clear description of the problem
- Steps to reproduce
- Your environment details

### 💡 Have an Idea?
- Check existing [issues](https://github.com/chess-seventh/rusty_cv_creator/issues) first
- Open a new issue tagged with `enhancement`
- Describe your use case and proposed solution

### 🔧 Want to Code?

```bash
# Fork the repository
git clone https://github.com/your-username/rusty_cv_creator.git
cd rusty_cv_creator

# Create a feature branch
git checkout -b feature/amazing-feature

# Make your changes
# Add tests for your changes!
cargo test

# Ensure code quality
cargo fmt
cargo clippy

# Commit and push
git commit -m "Add amazing feature"
git push origin feature/amazing-feature
```

Then open a Pull Request! 🎉

### 📋 Development Setup

```bash
# Install development dependencies
cargo install diesel_cli --no-default-features --features sqlite
cargo install cargo-tarpaulin  # For code coverage

# Set up the development database
diesel setup
diesel migration run

# Run tests
cargo test
```

## 🌟 Roadmap

- [ ] **Web Interface**: Browser-based CV management 
- [ ] **Template Gallery**: Community-contributed templates
- [ ] **Cloud Storage**: Google Drive/Dropbox integration
- [ ] **Multiple Formats**: Support for Word, HTML, Markdown output
- [ ] **AI Integration**: Smart content suggestions
- [ ] **Application Tracking**: Status updates and reminders
- [ ] **Analytics**: Application success metrics

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **[Diesel](https://diesel.rs)** - Safe, extensible ORM for Rust
- **[Clap](https://clap.rs)** - Command line argument parser  
- **[Skim](https://github.com/lotabout/skim)** - Fuzzy finder in Rust
- **[Chrono](https://github.com/chronotope/chrono)** - Date and time library
- **LaTeX Community** - For the amazing typesetting system

---

<div align="center">

**Made with ❤️ and ☕ by [chess-seventh](https://github.com/chess-seventh)**

*Star ⭐ this repo if you found it helpful!*

[⬆ Back to Top](#-rusty-cv-creator)

</div>
