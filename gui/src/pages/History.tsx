import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { 
  Search, ExternalLink, FileOutput, Filter
} from 'lucide-react';

export default function HistoryPage() {
  const [historyData, setHistoryData] = useState<any[]>([]);
  const [searchTerm, setSearchTerm] = useState('');

  useEffect(() => {
    loadHistory();
  }, []);

  const loadHistory = () => {
    invoke<any[]>('get_history').then(data => {
      // Sort by newest first
      setHistoryData(data.reverse());
    }).catch(console.error);
  };

  async function openFileSystem(path: string) {
    invoke('open_file_system', { path }).catch(console.error);
  }

  const filteredData = historyData.filter(item => item.name.toLowerCase().includes(searchTerm.toLowerCase()));

  return (
    <div className="flex flex-col gap-6 dark:text-white">
      <div className="flex justify-between items-end">
        <div>
          <h1 className="text-3xl font-bold text-text-main dark:text-white mb-2">History</h1>
          <p className="text-sm text-text-muted">Review past verifications and download generated reports.</p>
        </div>
      </div>

      <div className="card !p-0 overflow-hidden dark:bg-gray-800 dark:border-gray-700">
        <div className="p-4 border-b border-gray-100 dark:border-gray-700 flex justify-between items-center bg-gray-50/50 dark:bg-gray-800">
          <div className="relative w-64">
            <Search className="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-gray-400" />
            <input 
              type="text" 
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              placeholder="Filter by filename..." 
              className="w-full bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-700 rounded-lg py-2 pl-9 pr-4 text-sm focus:ring-2 focus:ring-brand/20 outline-none"
            />
          </div>
          <button className="px-4 py-2 border border-gray-200 dark:border-gray-700 rounded-lg text-sm font-medium flex items-center gap-2 hover:bg-gray-50 dark:hover:bg-gray-700">
            <Filter className="w-4 h-4" /> Filter
          </button>
        </div>

        <table className="w-full text-left border-collapse">
          <thead>
            <tr className="bg-white dark:bg-gray-900 border-b border-gray-100 dark:border-gray-700 text-xs uppercase tracking-wider text-text-muted">
              <th className="p-4 font-bold">File Name</th>
              <th className="p-4 font-bold">Processed Date</th>
              <th className="p-4 font-bold">Total Processed</th>
              <th className="p-4 font-bold">Deliverable</th>
              <th className="p-4 font-bold">Actions</th>
            </tr>
          </thead>
          <tbody>
            {filteredData.map(row => (
              <tr key={row.id} className="border-b border-gray-50 dark:border-gray-800 hover:bg-gray-50/50 dark:hover:bg-gray-700/50 transition-colors">
                <td className="p-4 font-semibold text-text-main dark:text-gray-200">{row.name}</td>
                <td className="p-4 text-sm text-text-muted">{row.date}</td>
                <td className="p-4 text-sm text-text-main dark:text-gray-300 font-medium">{row.total.toLocaleString()}</td>
                <td className="p-4 text-sm text-brand font-bold">{row.deliverable.toLocaleString()}</td>
                <td className="p-4 flex gap-2">
                  {row.minimal_path && (
                    <button onClick={() => openFileSystem(row.minimal_path)} title="Open Minimal Report" className="p-2 text-brand hover:bg-brand/10 rounded-lg transition-colors tooltip-trigger relative group">
                      <ExternalLink className="w-4 h-4" />
                    </button>
                  )}
                  {row.comprehensive_path && (
                    <button onClick={() => openFileSystem(row.comprehensive_path)} title="Open Comprehensive Report" className="p-2 text-brand hover:bg-brand/10 rounded-lg transition-colors tooltip-trigger relative group">
                      <FileOutput className="w-4 h-4" />
                    </button>
                  )}
                </td>
              </tr>
            ))}
            {historyData.length === 0 && (
              <tr>
                <td colSpan={5} className="p-8 text-center text-gray-500">No verifications in history yet.</td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
