# Windows Build & Installer Guide

This document provides step-by-step instructions for compiling the **Email Verifier** application into a professional Windows installer.

## 🚀 One-Click Build Command
If your environment is already set up, simply run the following from the root directory:

```powershell
cd gui
npm run tauri build
```

---

## 📋 Detailed Steps

### 1. Environment Preparation
Ensure your Windows environment has the necessary tools:
- **Rust**: [rustup.rs](https://rustup.rs/)
- **Node.js**: [nodejs.org](https://nodejs.org/)
- **WiX Toolset v3**: Required for generating `.msi` installers. [Download here](https://wixtoolset.org/releases/).

### 2. Branding & Metadata
The application identity is defined in `gui/src-tauri/tauri.conf.json`. 
- `productName`: "Email Verifier"
- `identifier`: "com.emailverifier.dev"
- `title`: "Email Verifier"

### 3. Icon Generation
If you want to update the app icon in the future:
1. Place your logo (PNG) in the `gui` folder.
2. Run: `npx tauri icon logo.png`
This automatically updates all necessary system icons in `src-tauri/icons`.

### 4. Compiling for Production
The production build performs heavy optimizations to ensure the app is fast and the file size is minimal.

```powershell
# Navigate to GUI
cd gui

# Clean old locks (optional, use if build gets stuck)
cargo clean --manifest-path src-tauri/Cargo.toml

# Run the build
npm run tauri build
```

---

## 📦 Where is my Installer?
After the build finishes, your files will be available at:

| File Type | Location |
| :--- | :--- |
| **MSI Installer** | `gui/src-tauri/target/release/bundle/msi/Email Verifier_0.1.0_x64_en-US.msi` |
| **Standalone EXE** | `gui/src-tauri/target/release/Email Verifier.exe` |

---

## ⚠️ Common Issues

### "Blocking waiting for file lock"
This happens if `npm run tauri dev` is still running or a previous build crashed.
- **Fix**: Close all terminals and check Task Manager for any `cargo.exe` or `rustc.exe` processes and end them.

### TypeScript Errors during Build
The production build uses a strict TypeScript check (`tsc`). 
- **Fix**: Ensure there are no unused imports or variables in your `.tsx` files. I have already cleaned these up for the current version.

### Missing WiX Toolset
If the build fails at the "Bundling" stage.
- **Fix**: Install WiX v3 and ensure it is added to your system PATH.
