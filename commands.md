# Email Verifier - Comprehensive Use Cases

This document outlines various ways to run the `email-verifier` CLI to suit different operational workflows.

> **Note on Compiled Release (.exe)**
> The examples below use `cargo run --` which is the standard way to run the tool through Rust.
> If you are using the compiled release version, you can run the standalone executable directly without installing Rust!
> 
> Simply replace `cargo run --` with `email-verifier.exe` in any of the examples below.
> 
> **Example (.exe usage):**
> ```cmd
> email-verifier.exe input.xlsx
> email-verifier.exe input.xlsx -s "Batch 3" -o verified_batch3
> email-verifier.exe input.csv --no-smtp --workers 50
> ```

## Basic Execution

The simplest way to run the tool. It will automatically detect columns with "email" in their header and process the active sheet of your Excel file (or the single sheet of a CSV).

```bash
cargo run -- input.xlsx
```
**Output generated:**
- `input_minimal.xlsx`: Clean sheet with original columns, Deliverable and Catch-All emails highlighted in blue.
- `input_comprehensive.xlsx`: Detailed sheet with full status columns added next to each email column.

---

## Targeted Sheets & Output Naming

If your Excel file has multiple sheets, or you want to explicitly name the output files.

```bash
# Target a specific sheet and name the output
cargo run -- input.xlsx -s "Batch 3" -o verified_batch3

# Resulting files:
# verified_batch3_minimal.xlsx
# verified_batch3_comprehensive.xlsx
```

---

## Custom Column Detection

If your column headers don't strictly contain the word "email", you can use a regular expression to match them.

```bash
# Match columns containing 'contact' or 'mail'
cargo run -- input.csv --column-pattern "(?i).*(contact|mail).*"
```

---

## Fast/Syntax-Only Mode

If you don't need deep SMTP/Catch-all verification and just want to rapidly check syntax and DNS records (MX checks), you can disable the SMTP step. This is much faster and doesn't send any network probes to mail servers.

```bash
cargo run -- input.xlsx --no-smtp
```

---

## Tuning Network Performance

For large lists or slow network conditions, you can increase the timeout, tweak retries, and adjust the concurrent worker threads.

```bash
# 15-second timeout, 5 retries, 50 concurrent workers
cargo run -- input.xlsx --timeout 15 --retries 5 --workers 50
```
> **Warning**: Increasing workers too high can trigger rate limits or IP bans from remote SMTP servers.

---

## Automation / Silent Mode

If you are running the tool in a cron job or pipeline and want to log the results instead of printing them to the terminal.

```bash
# Run silently, but write detailed logs to verify.log
cargo run -- input.csv --quiet --log verify.log
```

## Debugging

If you are running into issues or want to see the raw responses from SMTP servers for debugging purposes.

```bash
cargo run -- input.xlsx --verbose
```
