# Email Verifier Desktop App

A high-performance, production-ready desktop application designed to verify massive lists of email addresses efficiently and securely. Built with a lightning-fast Rust backend and a beautiful, glassy React frontend using Tauri.

## 🚀 Features

### Core Engine (Rust)
- **Multi-threaded Processing**: Uses asynchronous Tokio runtime to verify thousands of emails concurrently.
- **Advanced Verification**: 
  - Syntax checking via robust RegEx.
  - MX Record DNS lookups.
  - Catch-all and deliverability SMTP handshakes.
- **Smart Formatting**: Dynamically processes `.csv` and `.xlsx` files, and automatically detects/resolves issues like missing sheets or malformed cells.
- **Auto-sizing Excel Exports**: Writes comprehensive color-coded `.xlsx` reports that auto-resize to fit data beautifully.

### Frontend Dashboard (React + Tauri)
- **Live Verification UI**: View the exact live progress of your verification jobs via an animated, mathematically synchronized circular progress bar.
- **Configuration Modal**: Select target output folders, designate specific Excel sheets, and define custom color formatting (Default, Subtle, or None) before executing a job.
- **Data Persistence**: A custom SQLite/JSON database approach ensures your historical analytics, history logs, and user profile persist across app restarts.
- **Global Analytics**: High-level aggregated statistics on lifetime processed lists, total catch-alls, and average deliverability rates.

### OS Native Integrations
- **Reliable Output Launching**: Implements custom Rust shell handlers (`open_file_system`) to bypass browser-based permission errors and launch the finished `.xlsx` files instantly in your OS's native spreadsheet application.
- **Notifications**: System-wide event broadcasting for real-time alerts.

### User Customization
- **Theme Engine**: Toggle between the clean 'White' layout or a sleek 'Obsidian' Dark Mode.
- **Profile Manager**: Personalize the application with custom display names and preset avatars (👨‍💼/👩‍💼).

## 🛠 Architecture
The application is structured into a monorepo approach:
- `/core`: The core Rust library responsible for logic, multi-threading, and verification.
- `/cli`: A standalone Rust binary to run verification jobs directly from the terminal without a GUI.
- `/gui`: The Tauri wrapper wrapping a React + Vite frontend.
  - `src-tauri`: The Rust Tauri backend, defining the IPC (Inter-Process Communication) endpoints like `start_verification`, `get_history`, and `save_profile`.
  - `src`: The React components and Tailwind CSS styling.

## 💻 Getting Started

### Prerequisites
- Node.js (v18+)
- Rust (Cargo)
- NPM or Yarn

### Installation

1. Navigate to the GUI directory:
```bash
cd gui
```

2. Install the JavaScript dependencies:
```bash
npm install
```

3. Start the Development Server:
```bash
npm run tauri dev
```
*(This command will automatically spin up the Vite React server and compile the Rust application.)*

## 🎨 Design Philosophy
The UI was meticulously crafted to avoid the "spreadsheet app" feel. Instead, it utilizes a "Glassy Luxury" aesthetic—focusing on generous padding, subtle drop shadows, smooth transitions, and distinct typography to create a premium user experience.

## 📄 Output Types
- **Minimal Report**: A filtered `.csv` file containing *only* the emails that are fully deliverable and safe to send to.
- **Comprehensive Report**: A fully styled `.xlsx` file containing every original email, accompanied by appended columns indicating Status, Details, and processing time, complete with color-coded highlighting.
