# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

LifeTracker is a comprehensive life tracking application built with **Dioxus 0.6 + Rust**, migrated from a Tauri + React architecture. The application provides time tracking, financial management, diary/notes, habit tracking, and data analytics features.

**Migration Status**: Currently migrating from Tauri + React to Dioxus. Time tracking module is complete, accounting module is in progress. Legacy React components are archived in `_tauri_archive/`.

## Documentation

The Dioxus documentation is available in `docs\dioxus_0.6` directory.

## Development Commands

### Core Development
```bash
# Development mode (desktop app)
dx serve

# Development mode (web version)  
dx serve --platform web

# Build for production (desktop)
dx build --platform desktop --release

# Build for production (web)
dx build --platform web --release

# Code formatting
cargo fmt

# Code linting/checking
cargo clippy

# Run tests
cargo test
```

### Environment Setup
- **Rust 1.75+** required
- **CMake** required for dependencies
- **Dioxus CLI**: `cargo install dioxus-cli`

## Architecture Overview

### Application Structure
The application follows a **modular component architecture** where each major feature area has its own module:

```
src/
├── main.rs                 # Application entry point with dioxus::launch
├── lib.rs                  # Core library with global AppState management
├── components/             # Modular UI components
│   ├── app.rs              # Main router and application shell
│   ├── timing/             # Time tracking module (✅ complete)
│   ├── accounting/         # Financial management (🔄 in progress)
│   ├── diary/              # Notes/diary functionality (⏳ planned)
│   ├── habits.rs           # Habit tracking (⏳ planned)
│   └── common.rs           # Shared UI components
├── core/                   # Business logic layer
│   ├── timer.rs            # Timer/stopwatch logic
│   ├── task.rs             # Task management
│   ├── category.rs         # Category management
│   ├── accounting/         # Financial logic
│   └── analytics.rs        # Data analysis
├── storage/                # Data persistence layer
│   ├── database.rs         # SQLite operations with connection pooling
│   └── models.rs           # Data models and migrations
└── sync/                   # Data synchronization (WebDAV, etc.)
```

### Key Architectural Patterns

1. **Component-Based UI**: Each module has a main page component with tab navigation
   - `timing_page.rs` → `TimingPage` with tabs: dashboard, tasks, categories, statistics
   - `accounting_page.rs` → `AccountingPage` with tabs: overview, accounts, transactions, stats

2. **Global State Management**: 
   - Uses `AppState` with database connection and configuration
   - Synchronized initialization to avoid async runtime nesting
   - Global state accessible via `get_app_state_sync()` and `set_app_state()`

3. **Database Layer**: 
   - SQLite with WAL mode for better concurrency
   - Read/write connection separation using Arc<Mutex<Connection>>
   - Automatic migrations on startup

4. **Error Handling**:
   - Comprehensive `AppError` enum with specific error types
   - Error recovery strategies with retry logic
   - User-friendly error messages

### Data Flow
1. **UI Components** → Call core business logic functions
2. **Core Logic** → Performs validation and business rules  
3. **Storage Layer** → Handles database operations
4. **State Updates** → Components re-render via Dioxus signals

## Component Development Guidelines

### Dioxus Component Pattern
```rust
use dioxus::prelude::*;

#[component]
pub fn ExamplePage() -> Element {
    // State management
    let active_tab = use_signal(|| "dashboard");
    
    // Data loading
    let data = use_resource(move || async move {
        // Load data from core layer
        load_data().await
    });
    
    rsx! {
        div { class: "flex flex-col h-full",
            // Tab navigation
            div { class: "flex border-b border-gray-200 dark:border-gray-700",
                button {
                    class: if *active_tab.read() == "dashboard" { 
                        "px-4 py-2 text-theme-primary border-b-2 border-theme-primary" 
                    } else { 
                        "px-4 py-2 text-gray-500 hover:text-gray-700" 
                    },
                    onclick: move |_| active_tab.set("dashboard"),
                    "仪表板"
                }
            }
            
            // Content area with conditional rendering
            div { class: "flex-1 p-4",
                match active_tab.read().as_str() {
                    "dashboard" => rsx! { DashboardTab {} },
                    "tasks" => rsx! { TaskTab {} },
                    _ => rsx! { div { "未知页面" } }
                }
            }
        }
    }
}
```

### Styling Approach
- **Inline CSS classes** using Tailwind-like utilities
- **Dark mode support** via `dark:` prefixes
- **Responsive design** with `md:`, `lg:` breakpoints
- **Theme variables** for consistent colors

### State Management
- Use `use_signal` for local component state
- Use `use_resource` for async data loading
- Access global state via `get_app_state_sync()`
- Avoid nested async runtimes

## Module Organization

### Timing Module (Complete)
- **dashboard.rs**: Timer display, current task info, quick actions
- **task_management.rs**: CRUD operations for tasks
- **category_management.rs**: Category creation and management  
- **statistics.rs**: Time tracking analytics and reports

### Accounting Module (In Progress)
- **overview.rs**: Financial dashboard with account balances
- **accounts.rs**: Account management (checking, savings, credit cards)
- **transactions.rs**: Income/expense recording and categorization
- **stats.rs**: Financial analytics and budget tracking
- **trend_chart.rs**: Visual financial trends

### Core Business Logic
- **AppCore**: Central coordinator that manages timer, tasks, categories, analytics
- **Timer**: Handles start/stop/pause operations with precise timing
- **TaskManager**: CRUD operations and task lifecycle management
- **CategoryManager**: Category organization and validation
- **Analytics**: Report generation and trend analysis

## Database Schema
- **tasks**: Task records with timing data
- **categories**: Task categorization system  
- **accounts**: Financial account information
- **transactions**: Financial transaction records
- **time_logs**: Detailed time tracking entries
- **app_config**: Application configuration storage

## Important Development Notes

1. **Migration Context**: This project is actively migrating from Tauri+React to Dioxus+Rust. Some React components in `_tauri_archive/` serve as reference for feature parity.

2. **Cursor Rules**: Follow the development patterns defined in `.cursor/rules/my-cursor-rules.mdc` - prioritize modular architecture, use Chinese comments with English logs, and maintain single file responsibility.

3. **File Size Limits**: Keep component files under 500 lines. Use sub-modules for complex features.

4. **Error Handling**: Always use `Result<T, AppError>` for operations that can fail. Implement proper error recovery strategies.

5. **Testing**: Focus on core business logic testing. UI components should have minimal testing overhead.

6. **Async Patterns**: Use synchronous initialization patterns to avoid runtime nesting issues with Dioxus.

## Common Tasks

### Adding a New Feature Module
1. Create module directory in `src/components/`
2. Implement main page component with tab navigation
3. Add core business logic in `src/core/`
4. Update database schema if needed
5. Add route in `src/components/app.rs`

### Database Operations
```rust
// Reading data
let tasks = db.read(|conn| {
    // Use parameterized queries
    let mut stmt = conn.prepare("SELECT * FROM tasks WHERE category_id = ?1")?;
    // Process results...
    Ok(results)
})?;

// Writing data  
let result = db.write(|conn| {
    conn.execute("INSERT INTO tasks (name, category_id) VALUES (?1, ?2)", 
                 params![name, category_id])?;
    Ok(())
})?;
```

### Error Handling Pattern
```rust
use crate::errors::{AppError, Result};

pub fn example_operation() -> Result<String> {
    let data = some_operation()
        .map_err(|e| AppError::Business(format!("Operation failed: {}", e)))?;
    
    validate_data(&data)?;
    Ok(data)
}
```

This architecture ensures maintainable, testable code while providing a solid foundation for the comprehensive life tracking application.