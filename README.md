# Email Verifier

A high-performance, concurrent Rust CLI application for bulk email verification. Designed to process large datasets (Excel and CSV formats) rapidly by leveraging async networking and a multi-layered verification pipeline.

## 🚀 Key Features

- **Multi-Layered Verification Pipeline:**
  1. **Syntax Check:** Validates format using strict regex.
  2. **Disposable & Role-Based Check:** Filters out temporary emails (e.g., mailinator) and generic role addresses (e.g., admin@, support@).
  3. **DNS Resolution:** Validates the existence of Mail Exchange (MX) records.
  4. **Catch-All Detection:** Probes domains with non-existent aliases to determine if the server accepts all traffic.
  5. **Active SMTP Verification:** Negotiates an SMTP handshake to ensure the specific mailbox exists without actually sending an email.
- **High Concurrency:** Built on `tokio`, allowing hundreds of network requests concurrently for rapid processing.
- **Excel & CSV Native:** Seamlessly reads from and writes to `.xlsx`, `.xlsm`, and `.csv` files.
- **Smart Output:** Generates two tailored reports:
  - **Minimal:** Original columns only. Deliverable and Catch-All emails are highlighted in bright blue for immediate use.
  - **Comprehensive:** A fully color-coded report with explicit status columns appended to each analyzed email column.

## 🛠 Installation

You must have [Rust and Cargo installed](https://rustup.rs/) on your system.

```bash
# Clone the repository (if applicable) and navigate to the directory
cd email-verifier

# Build the release executable
cargo build --release
```

After building, the standalone executable will be located at `target/release/email-verifier.exe` (on Windows) or `target/release/email-verifier` (on Mac/Linux).

## 📖 Usage Quickstart

You can run the tool directly using the built executable:

```cmd
# Basic execution - auto-detects "email" columns and verifies them
email-verifier.exe input.xlsx

# Specify a sheet and an output name prefix
email-verifier.exe input.xlsx -s "Mailing List" -o verified_campaign

# Run in fast-mode (skip active SMTP handshakes, check syntax/DNS only)
email-verifier.exe input.csv --no-smtp
```

> **For a complete list of commands, performance tuning parameters, and use-cases, please read our dedicated [Use Cases Guide](commands.md).**

## 📊 Output Formats

Every successful run generates three files automatically:

1. **`[name]_minimal.xlsx`**: Your original dataset untouched, except safe emails (Deliverable and Catch-all) are highlighted with a blue background. Perfect for immediately loading into your mailing software.
2. **`[name]_comprehensive.xlsx`**: Detailed output where every email column gets a sister `_status` column. Color-coded based on severity:
   - **Blue:** Deliverable / Success
   - **Grey:** Catch-all / Role-Based / Disposable
   - **Yellow:** Domain Corrected (typos automatically fixed)
   - **Red:** Undeliverable / Mailbox Not Exist / Syntax Error
3. **`[name].summary.json`**: A lightweight JSON file containing performance metrics, error rates, and total verification counts.

## ⚙️ Architecture

The codebase is heavily modularized to support easy feature addition:
- `src/main.rs`: Execution orchestration and Tokio task management.
- `src/verifier.rs`: The state machine executing the 6-layer logic.
- `src/smtp_verifier.rs`: Low-level network socket handling and SMTP protocol negotiations.
- `src/output.rs`: Advanced Excel file manipulation using `rust_xlsxwriter`.
- `src/dedup.rs`: In-memory caching and deduplication to prevent querying the same domain/email multiple times in one batch.
