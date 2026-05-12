import { Book, Terminal } from 'lucide-react';

export default function HelpPage() {
  return (
    <div className="flex flex-col gap-6 dark:text-white pb-10">
      <div className="flex justify-between items-end">
        <div>
          <h1 className="text-3xl font-bold text-text-main dark:text-white mb-2">Help Center</h1>
          <p className="text-sm text-text-muted">Learn how to use Email Verifier effectively.</p>
        </div>
      </div>

      <div className="grid grid-cols-2 gap-6">
        <div className="card dark:bg-gray-800 dark:border-gray-700 flex flex-col gap-4">
          <div className="flex items-center gap-3 border-b border-gray-100 dark:border-gray-700 pb-4">
            <div className="w-10 h-10 rounded-full bg-brand/10 text-brand flex items-center justify-center">
              <Book className="w-5 h-5" />
            </div>
            <h2 className="text-lg font-bold">GUI Guide</h2>
          </div>
          <div className="text-sm text-text-main dark:text-gray-300 space-y-4">
            <p><strong>1. Selecting a File:</strong> Go to the Dashboard and click "Select File". You can load `.xlsx` or `.csv` files.</p>
            <p><strong>2. Sheet Options:</strong> If your Excel file has multiple sheets, a dropdown will appear allowing you to select which sheet to process.</p>
            <p><strong>3. Data Preview:</strong> A table will display the first 5 rows so you can ensure the correct data is loaded.</p>
            <p><strong>4. Verification:</strong> Click "Start Verification". Two files are generated: a <i>Minimal Report</i> (just emails and status) and a <i>Comprehensive Report</i> (original data + status columns).</p>
          </div>
        </div>

        <div className="card dark:bg-gray-800 dark:border-gray-700 flex flex-col gap-4">
          <div className="flex items-center gap-3 border-b border-gray-100 dark:border-gray-700 pb-4">
            <div className="w-10 h-10 rounded-full bg-brand/10 text-brand flex items-center justify-center">
              <Terminal className="w-5 h-5" />
            </div>
            <h2 className="text-lg font-bold">CLI Usage</h2>
          </div>
          <div className="text-sm text-text-main dark:text-gray-300 space-y-4">
            <p>You can also run verifications headlessly via the terminal.</p>
            <div className="bg-gray-50 dark:bg-gray-900 p-4 rounded-lg font-mono text-xs border border-gray-100 dark:border-gray-700 overflow-x-auto">
              cargo run -p email-verifier-cli -- --input leads.xlsx --workers 50
            </div>
            <ul className="list-disc pl-5 space-y-2 mt-2 text-text-muted">
              <li><code className="text-brand">--no-smtp</code>: Skip deep SMTP validation.</li>
              <li><code className="text-brand">--column-pattern</code>: Regex to find email headers.</li>
              <li><code className="text-brand">--quiet</code>: Suppress console progress bar.</li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  );
}
