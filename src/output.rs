use crate::types::{CellKey, VerificationResult, VerificationStatus};
use rust_xlsxwriter::{Color, Format, FormatAlign, FormatBorder, Workbook, XlsxError};
use std::collections::HashMap;

fn xlsx_err(e: XlsxError) -> anyhow::Error {
    anyhow::anyhow!("Excel write error: {}", e)
}

pub struct OutputWriter;

impl OutputWriter {
    /// Minimal output: reproduces the **original columns only** (no status
    /// columns added).  Email cells that verified as Deliverable/Success/CatchAll are
    /// highlighted in blue (#00B0F0).  All column widths are auto-sized.
    pub fn write_minimal_output(
        filepath: &str,
        rows: &[Vec<String>],
        headers: &[String],
        // (email_col 1-based, status_col 1-based) — same pairs used in main
        status_columns: &[(usize, usize)],
        results: &HashMap<CellKey, VerificationResult>,
    ) -> anyhow::Result<()> {
        // Build a lookup: (row 1-based, email_col 1-based) → is_deliverable
        // Results are keyed by (row, status_col); we want (row, email_col).
        let deliverable_set: std::collections::HashSet<(usize, usize)> = results
            .iter()
            .filter_map(|((row, status_col), result)| {
                // Find the email_col whose status_col matches this result key.
                // Explicit deref needed: sc is &&usize, status_col is &usize.
                status_columns
                    .iter()
                    .find(|(_, sc)| *sc == *status_col)
                    .and_then(|(ec, _)| {
                        matches!(
                            result.status,
                            VerificationStatus::Deliverable | VerificationStatus::Success | VerificationStatus::CatchAll
                        )
                        .then_some((*row, *ec))
                    })
            })
            .collect();

        // ── Formats ───────────────────────────────────────────────────────────
        let header_fmt = Format::new()
            .set_bold()
            .set_align(FormatAlign::Center)
            .set_border(FormatBorder::Thin)
            .set_background_color(Color::RGB(0x1F_49_7D)) // dark navy
            .set_font_color(Color::White);

        let plain_fmt = Format::new().set_border(FormatBorder::Thin);

        // Blue highlight: #00B0F0 — only on verified email cells
        let blue_fmt = Format::new()
            .set_border(FormatBorder::Thin)
            .set_background_color(Color::RGB(0x00_B0_F0));

        // ── Workbook / worksheet ──────────────────────────────────────────────
        let mut workbook = Workbook::new();
        let ws = workbook.add_worksheet();
        ws.set_name("Verified Emails").map_err(xlsx_err)?;

        let col_count = headers.len();

        // ── Column width tracking ─────────────────────────────────────────────
        // Start from header length, then grow with each cell value
        let mut widths: Vec<usize> = headers.iter().map(|h| h.len() + 2).collect();

        // ── Row 0: header ─────────────────────────────────────────────────────
        for (col_idx, header) in headers.iter().enumerate() {
            ws.write_with_format(0, col_idx as u16, header.as_str(), &header_fmt)
                .map_err(xlsx_err)?;
        }

        // ── Rows 1+: data ─────────────────────────────────────────────────────
        for (row_idx, row_data) in rows.iter().enumerate() {
            let excel_row = row_idx + 2; // 1-based row index (row 1 = header)

            for col_idx in 0..col_count {
                let value = row_data.get(col_idx).map(|s| s.as_str()).unwrap_or("");

                // Is this cell an email column that verified as deliverable?
                let email_col_1based = col_idx + 1;
                let is_blue = status_columns
                    .iter()
                    .any(|(ec, _)| *ec == email_col_1based)
                    && deliverable_set.contains(&(excel_row, email_col_1based));

                let fmt = if is_blue { &blue_fmt } else { &plain_fmt };

                ws.write_with_format((row_idx + 1) as u32, col_idx as u16, value, fmt)
                    .map_err(xlsx_err)?;

                // Track max width
                if col_idx < widths.len() {
                    widths[col_idx] = widths[col_idx].max(value.len() + 2);
                }
            }
        }

        // ── Apply column widths ───────────────────────────────────────────────
        for (col_idx, &w) in widths.iter().enumerate() {
            // Clamp to a reasonable max (50 chars) and convert to pixels (~7px/char)
            let px = (w.min(60) * 7) as u16;
            ws.set_column_width_pixels(col_idx as u16, px)
                .map_err(xlsx_err)?;
        }

        workbook.save(filepath).map_err(xlsx_err)?;
        tracing::info!("Minimal output written to {}", filepath);
        Ok(())
    }

    /// Comprehensive output: writes a brand-new colour-coded Excel workbook.
    ///
    /// Column layout: all original columns are written, with a `<header>_status`
    /// column inserted immediately after every email column.
    ///
    /// Colour scheme (background):
    ///   • Blue   #00B0F0  → Deliverable / Success
    ///   • Red    #FFC7CE  → Undeliverable / MailboxNotExist / SyntaxError / DnsError
    ///   • Yellow #FFEB9C  → DomainCorrected (typo was fixed)
    ///   • Grey   #D3D3D3  → CatchAll / RoleBased / Disposable
    ///   • White  (default) → everything else
    pub fn write_comprehensive_output(
        filepath: &str,
        rows: &[Vec<String>],
        headers: &[String],
        status_columns: &[(usize, usize)], // (email_col 1-based, status_col 1-based)
        results: &HashMap<CellKey, VerificationResult>,
        auto_size: bool,
    ) -> anyhow::Result<()> {
        // ── Build merged header list ──────────────────────────────────────────
        let mut output_headers = headers.to_vec();
        for (email_col, status_col) in status_columns.iter().rev() {
            let email_header = headers
                .get(email_col.saturating_sub(1))
                .cloned()
                .unwrap_or_else(|| format!("Email{}", email_col));
            let insert_idx = status_col.saturating_sub(1).min(output_headers.len());
            output_headers.insert(insert_idx, format!("{}_status", email_header));
        }
        let col_count = output_headers.len();

        // ── Formats ───────────────────────────────────────────────────────────
        let header_fmt = Format::new()
            .set_bold()
            .set_align(FormatAlign::Center)
            .set_border(FormatBorder::Thin)
            .set_background_color(Color::RGB(0x1F_49_7D)) // dark navy
            .set_font_color(Color::White);

        let border_fmt = Format::new().set_border(FormatBorder::Thin);

        let blue_fmt = Format::new()
            .set_border(FormatBorder::Thin)
            .set_background_color(Color::RGB(0x00_B0_F0)); // #00B0F0

        let red_fmt = Format::new()
            .set_border(FormatBorder::Thin)
            .set_background_color(Color::RGB(0xFF_C7_CE)); // #FFC7CE

        let yellow_fmt = Format::new()
            .set_border(FormatBorder::Thin)
            .set_background_color(Color::RGB(0xFF_EB_9C)); // #FFEB9C

        let grey_fmt = Format::new()
            .set_border(FormatBorder::Thin)
            .set_background_color(Color::RGB(0xD3_D3_D3)); // #D3D3D3

        // ── Workbook / worksheet ──────────────────────────────────────────────
        let mut workbook = Workbook::new();
        let ws = workbook.add_worksheet();
        ws.set_name("Verified Emails").map_err(xlsx_err)?;

        // Row 0: header row
        for (col_idx, header) in output_headers.iter().enumerate() {
            ws.write_with_format(0, col_idx as u16, header.as_str(), &header_fmt)
                .map_err(xlsx_err)?;
        }

        // Rows 1+: data
        for (row_idx, row_data) in rows.iter().enumerate() {
            // Rebuild the row inserting status strings at the right positions
            let mut output_row: Vec<String> = row_data.clone();
            for (_email_col, status_col) in status_columns.iter().rev() {
                let key: CellKey = (row_idx + 2, *status_col);
                let status_str = results
                    .get(&key)
                    .map(|r| r.status.as_str().to_string())
                    .unwrap_or_default();
                let insert_idx = status_col.saturating_sub(1).min(output_row.len());
                output_row.insert(insert_idx, status_str);
            }

            for (col_idx, value) in output_row.iter().enumerate() {
                let fmt = Self::pick_format(
                    col_idx,
                    row_idx,
                    status_columns,
                    results,
                    &blue_fmt,
                    &red_fmt,
                    &yellow_fmt,
                    &grey_fmt,
                    &border_fmt,
                );
                ws.write_with_format(
                    (row_idx + 1) as u32,
                    col_idx as u16,
                    value.as_str(),
                    fmt,
                )
                .map_err(xlsx_err)?;
            }
        }

        // ── Optional column auto-sizing ───────────────────────────────────────
        if auto_size {
            let mut widths = vec![10usize; col_count];
            for (i, h) in output_headers.iter().enumerate() {
                widths[i] = widths[i].max(h.len() + 2);
            }
            for row_data in rows {
                for (i, v) in row_data.iter().enumerate() {
                    if i < col_count {
                        widths[i] = widths[i].max(v.len() + 2);
                    }
                }
            }
            for (col_idx, &w) in widths.iter().enumerate() {
                let px = (w.min(60) * 7) as u16;
                ws.set_column_width_pixels(col_idx as u16, px)
                    .map_err(xlsx_err)?;
            }
        }

        workbook.save(filepath).map_err(xlsx_err)?;
        tracing::info!("Comprehensive Excel output written to {}", filepath);
        Ok(())
    }

    /// Choose the cell background format.
    ///
    /// Status columns get colour-coded by the result; all other columns get
    /// the plain border format.
    #[allow(clippy::too_many_arguments)]
    fn pick_format<'a>(
        col_idx: usize,        // 0-based column in the output row
        row_idx: usize,        // 0-based data row
        status_columns: &[(usize, usize)],
        results: &HashMap<CellKey, VerificationResult>,
        blue: &'a Format,
        red: &'a Format,
        yellow: &'a Format,
        grey: &'a Format,
        plain: &'a Format,
    ) -> &'a Format {
        // status_col is 1-based; 0-based index = status_col - 1
        for (_email_col, status_col) in status_columns {
            if col_idx + 1 == *status_col {
                let key: CellKey = (row_idx + 2, *status_col);
                if let Some(result) = results.get(&key) {
                    return match result.status {
                        VerificationStatus::Deliverable | VerificationStatus::Success => blue,
                        VerificationStatus::Undeliverable
                        | VerificationStatus::MailboxNotExist
                        | VerificationStatus::SyntaxError
                        | VerificationStatus::DnsError => red,
                        VerificationStatus::DomainCorrected => yellow,
                        VerificationStatus::CatchAll
                        | VerificationStatus::RoleBased
                        | VerificationStatus::Disposable => grey,
                        _ => plain,
                    };
                }
            }
        }
        plain
    }
}