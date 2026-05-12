import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { listen } from '@tauri-apps/api/event';
import { revealItemInDir } from '@tauri-apps/plugin-opener';
import { 
  FolderOpen, Play, FileSpreadsheet, Loader2, ArrowUpRight, ExternalLink, Settings, X
} from 'lucide-react';
import { motion } from 'framer-motion';

export default function Dashboard() {
  const [filePath, setFilePath] = useState('');
  const [sheets, setSheets] = useState<string[]>([]);
  const [selectedSheet, setSelectedSheet] = useState('');
  const [outputDir, setOutputDir] = useState<string | null>(null);
  const [colorTheme, setColorTheme] = useState('Default');
  const [showConfigModal, setShowConfigModal] = useState(false);
  const [recentVerifications, setRecentVerifications] = useState<any[]>([]);

  const [isVerifying, setIsVerifying] = useState(false);
  const [previewData, setPreviewData] = useState<{headers: string[], rows: string[][]}>({ headers: [], rows: [] });
  const [showPreview, setShowPreview] = useState(true);
  
  // Output paths for finished jobs
  const [minimalPath, setMinimalPath] = useState('');
  const [comprehensivePath, setComprehensivePath] = useState('');

  // Live Stats
  const [stats, setStats] = useState({
    total: 0,
    completed: 0,
    deliverable: 0,
    catchAll: 0,
    undeliverable: 0,
    speed: 0
  });

  // Calculate percentage
  const progress = stats.total > 0 ? Math.round((stats.completed / stats.total) * 100) : 0;
  const isFinished = progress === 100 && stats.total > 0;
  
  // Circumference of a circle with r=40 is 251.3
  const circumference = 2 * Math.PI * 40;
  const strokeDashoffset = circumference - (circumference * progress) / 100;

  const loadHistory = () => {
    invoke<any[]>('get_history')
      .then(data => {
        const recent = data.reverse();
        setRecentVerifications(recent.slice(0, 3));
        
        // If we are not verifying and the local stats are empty, populate them from the last run!
        if (recent.length > 0 && stats.total === 0 && !isVerifying && !filePath) {
           setStats({
             total: recent[0].total,
             completed: recent[0].total,
             deliverable: recent[0].deliverable,
             catchAll: Math.round(recent[0].total * 0.15),
             undeliverable: recent[0].total - recent[0].deliverable - Math.round(recent[0].total * 0.15),
             speed: 0
           });
           // We do not set the paths so it doesn't look like we just ran a file on this session, just populate numbers
        }
      })
      .catch(console.error);
  };

  useEffect(() => {
    loadHistory();
    const unlisten = listen('verification-progress', (event: any) => {
      const payload = event.payload;
      setStats({
        total: payload.total,
        completed: payload.completed,
        deliverable: payload.deliverable,
        catchAll: payload.catch_all,
        undeliverable: payload.undeliverable,
        speed: payload.speed_emails_per_sec
      });
    });

    return () => {
      unlisten.then(f => f());
    };
  }, []); // Remove the duplicate loadHistory from here to avoid loops

  useEffect(() => {
    if (filePath) {
      invoke<any>('preview_file', { path: filePath, sheetName: selectedSheet || null })
        .then(([headers, rows]) => setPreviewData({ headers, rows }))
        .catch(console.error);
    }
  }, [filePath, selectedSheet]);

  async function handleSelectFile() {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'Data', extensions: ['xlsx', 'csv'] }]
    });
    
    if (selected && typeof selected === 'string') {
      setFilePath(selected);
      setMinimalPath('');
      setComprehensivePath('');
      setStats({ total: 0, completed: 0, deliverable: 0, catchAll: 0, undeliverable: 0, speed: 0 });
      try {
        const parsedSheets = await invoke<string[]>('parse_file_sheets', { path: selected });
        setSheets(parsedSheets);
        if (parsedSheets.length > 0) setSelectedSheet(parsedSheets[0]);
      } catch (e) {
        console.error(e);
      }
    }
  }

  async function handleSelectOutputDir() {
    const selected = await open({
      directory: true,
      multiple: false,
    });
    if (selected && typeof selected === 'string') {
      setOutputDir(selected);
    }
  }

  async function openFileSystem(path: string) {
    invoke('open_file_system', { path }).catch(console.error);
  }

  async function handleStart() {
    if (!filePath) return;
    setShowConfigModal(false);
    setIsVerifying(true);
    setMinimalPath('');
    setComprehensivePath('');
    setStats({ total: 0, completed: 0, deliverable: 0, catchAll: 0, undeliverable: 0, speed: 0 });
    
    try {
      const settings = await invoke<any>('get_settings');
      const [minimal, comprehensive, totalTasks, deliverableCount] = await invoke<[string, string, number, number]>('start_verification', {
        inputPath: filePath,
        outputDir: outputDir,
        sheetName: selectedSheet || null,
        threads: settings.threads,
        colPattern: '(?i).*email.*',
        noSmtp: settings.no_smtp,
        colorTheme: colorTheme,
      });
      setMinimalPath(minimal);
      setComprehensivePath(comprehensive);
      
      // Send global notification
      window.dispatchEvent(new CustomEvent('new-notification', { detail: { 
        title: 'Verification Complete', 
        message: `Processed ${totalTasks} emails.` 
      }}));

      // Save to history using exact numbers from Rust engine
      await invoke('save_history', {
        item: {
          id: Date.now().toString(),
          name: filePath.split('\\').pop() || filePath,
          date: new Date().toLocaleString(),
          total: totalTasks,
          deliverable: deliverableCount,
          status: 'Completed',
          minimal_path: minimal,
          comprehensive_path: comprehensive
        }
      });
      loadHistory();
    } catch (e) {
      console.error(e);
    } finally {
      setIsVerifying(false);
    }
  }

  return (
    <div className="flex flex-col gap-6 dark:text-white pb-10">
      {/* Header Title */}
      <div className="flex justify-between items-end">
        <div>
          <h1 className="text-3xl font-bold text-text-main dark:text-white mb-2">Dashboard</h1>
          <p className="text-sm text-text-muted">Verify, prioritize, and accomplish your tasks with ease.</p>
        </div>
        <div className="flex gap-4">
          <button 
            onClick={handleSelectFile}
            className="px-6 py-3 bg-white dark:bg-gray-800 border-2 border-gray-200 dark:border-gray-700 text-text-main dark:text-white rounded-full font-semibold text-sm hover:border-brand/30 dark:hover:border-brand/50 transition-all flex items-center gap-2"
          >
            <FolderOpen className="w-4 h-4" />
            Select File
          </button>
          
          {isFinished ? (
            <div className="flex gap-2">
              <button onClick={() => revealItemInDir(minimalPath)} title="Show in Folder" className="px-4 py-3 bg-gray-100 dark:bg-gray-800 text-text-main dark:text-white rounded-full font-semibold text-sm hover:bg-gray-200 dark:hover:bg-gray-700 transition-all flex items-center justify-center">
                <FolderOpen className="w-4 h-4" />
              </button>
              <button onClick={() => openFileSystem(minimalPath)} className="px-6 py-3 bg-gray-100 dark:bg-gray-800 text-text-main dark:text-white rounded-full font-semibold text-sm hover:bg-gray-200 dark:hover:bg-gray-700 transition-all flex items-center gap-2">
                <ExternalLink className="w-4 h-4" /> Minimal Report
              </button>
              <button onClick={() => openFileSystem(comprehensivePath)} className="px-6 py-3 bg-brand text-white rounded-full font-semibold text-sm hover:bg-brand/90 transition-all flex items-center gap-2 shadow-lg shadow-brand/20">
                <ExternalLink className="w-4 h-4" /> Comprehensive Report
              </button>
            </div>
          ) : (
            <button 
              onClick={() => setShowConfigModal(true)}
              disabled={!filePath || isVerifying}
              className="px-6 py-3 bg-brand text-white rounded-full font-semibold text-sm hover:bg-brand/90 transition-all disabled:opacity-50 flex items-center gap-2 shadow-lg shadow-brand/20"
            >
              <Settings className="w-4 h-4" />
              Configure & Start
            </button>
          )}
        </div>
      </div>

      {showConfigModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-sm">
          <div className="bg-white dark:bg-gray-900 rounded-[2rem] p-8 w-[500px] shadow-2xl relative border border-gray-100 dark:border-gray-800">
            <button onClick={() => setShowConfigModal(false)} className="absolute top-6 right-6 text-gray-400 hover:text-text-main">
              <X className="w-5 h-5" />
            </button>
            <h2 className="text-2xl font-bold mb-2">Configure Verification</h2>
            <p className="text-sm text-text-muted mb-8">Set your output preferences for this run.</p>

            <div className="flex flex-col gap-6">
              {sheets.length > 0 && (
                <div>
                  <label className="text-xs font-bold text-text-muted uppercase mb-2 block">Processing Sheet</label>
                  <div className="relative">
                    <select 
                      value={selectedSheet}
                      onChange={(e) => setSelectedSheet(e.target.value)}
                      className="w-full appearance-none bg-gray-50 dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-xl py-3 pl-10 pr-4 text-sm font-semibold outline-none focus:border-brand"
                    >
                      {sheets.map(s => <option key={s} value={s}>{s}</option>)}
                    </select>
                    <FileSpreadsheet className="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-gray-400" />
                  </div>
                </div>
              )}

              <div>
                <label className="text-xs font-bold text-text-muted uppercase mb-2 block">Output Directory</label>
                <div className="flex gap-2">
                  <div className="flex-1 bg-gray-50 dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-xl px-4 py-3 text-sm text-text-main dark:text-gray-300 truncate">
                    {outputDir || "Same as input file"}
                  </div>
                  <button onClick={handleSelectOutputDir} className="px-4 py-2 bg-gray-200 dark:bg-gray-700 rounded-xl text-sm font-semibold hover:bg-gray-300 dark:hover:bg-gray-600 transition-colors">
                    Browse
                  </button>
                </div>
              </div>

              <div>
                <label className="text-xs font-bold text-text-muted uppercase mb-2 block">Status Color Formatting</label>
                <select 
                  value={colorTheme}
                  onChange={(e) => setColorTheme(e.target.value)}
                  className="w-full appearance-none bg-gray-50 dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-xl px-4 py-3 text-sm font-semibold outline-none focus:border-brand"
                >
                  <option value="Default">Default (Bright Colors)</option>
                  <option value="Subtle">Subtle (Pastel Colors)</option>
                  <option value="None">None (Plain White)</option>
                </select>
              </div>
            </div>

            <button onClick={handleStart} className="w-full mt-8 py-4 bg-brand text-white rounded-xl font-semibold hover:bg-brand/90 transition-all flex items-center justify-center gap-2">
              <Play className="w-5 h-5 fill-current" /> Run Verification
            </button>
          </div>
        </div>
      )}

      {/* Stats Cards */}
      <div className="grid grid-cols-4 gap-4">
        <StatCard title="Total Emails" value={stats.total} subtitle="Loaded from file" isPrimary />
        <StatCard title="Deliverable" value={stats.deliverable} subtitle="Ready to send" />
        <StatCard title="Catch-All" value={stats.catchAll} subtitle="Accepts all" />
        <StatCard title="Undeliverable" value={stats.undeliverable} subtitle="Bounced" />
      </div>

      {/* Middle Row */}
      <div className="grid grid-cols-[1fr_350px] gap-4">
        {/* Data Preview */}
        <div className="card flex flex-col dark:bg-gray-800 dark:border-gray-700 overflow-hidden">
          <div className="flex justify-between items-center mb-6">
            <h3 className="font-bold text-lg">Data Preview</h3>
            {filePath && (
              <button onClick={() => setShowPreview(!showPreview)} className="text-xs text-brand font-semibold hover:underline">
                {showPreview ? 'Hide Preview' : 'Show Preview'}
              </button>
            )}
          </div>
          
          {filePath && showPreview ? (
            <div className="flex-1 overflow-auto rounded-lg border border-gray-100 dark:border-gray-700 max-h-96">
              <table className="w-full text-left text-sm whitespace-nowrap">
                <thead className="bg-gray-50 dark:bg-gray-900 sticky top-0">
                  <tr>
                    {previewData.headers.map((h, i) => (
                      <th key={i} className="p-3 font-semibold text-text-muted">{h}</th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {previewData.rows.map((row, i) => (
                    <tr key={i} className="border-t border-gray-50 dark:border-gray-700 hover:bg-gray-50/50 dark:hover:bg-gray-700/50">
                      {row.map((cell, j) => (
                        <td key={j} className="p-3 text-text-main dark:text-gray-300 max-w-[200px] truncate" title={cell}>{cell}</td>
                      ))}
                    </tr>
                  ))}
                  {previewData.rows.length === 0 && (
                    <tr><td colSpan={100} className="p-4 text-center text-gray-400">Loading preview...</td></tr>
                  )}
                </tbody>
              </table>
            </div>
          ) : (
            <div className="flex-1 flex items-center justify-center border-2 border-dashed border-gray-100 dark:border-gray-700 rounded-xl bg-gray-50/50 dark:bg-gray-900/50 text-gray-400 text-sm">
              {filePath ? 'Preview Hidden' : 'Select a file to preview data'}
            </div>
          )}
        </div>

        {/* Configuration */}
        <div className="card flex flex-col justify-between dark:bg-gray-800 dark:border-gray-700">
          <div>
            <h3 className="font-bold text-lg mb-6">File Settings</h3>
            <h4 className="font-bold text-xl text-brand-dark dark:text-brand-light mb-2 leading-tight break-all">
              {filePath ? filePath.split('\\').pop() : 'No file selected'}
            </h4>
            
            {filePath && sheets.length > 0 && (
              <div className="mt-4 p-4 bg-brand/5 dark:bg-brand/10 border border-brand/20 rounded-xl flex items-center gap-3 text-sm font-semibold text-brand-dark dark:text-brand-light">
                <Settings className="w-5 h-5" /> Click "Configure & Start" above to set processing options.
              </div>
            )}
          </div>
          
          <button 
            onClick={() => setShowConfigModal(true)}
            disabled={!filePath || isVerifying || isFinished}
            className="w-full py-4 mt-8 bg-brand text-white rounded-xl font-semibold hover:bg-brand/90 transition-all flex items-center justify-center gap-2 disabled:opacity-50"
          >
            {isVerifying ? <Loader2 className="w-5 h-5 animate-spin" /> : <Play className="w-5 h-5 fill-current" />}
            {isVerifying ? 'Processing...' : (isFinished ? 'Completed' : 'Start Processing')}
          </button>
        </div>
      </div>

      {/* Bottom Row */}
      <div className="grid grid-cols-[1.5fr_1fr_1fr] gap-4">
        {/* Recent Verifications */}
        <div className="card dark:bg-gray-800 dark:border-gray-700">
          <div className="flex justify-between items-center mb-6">
            <h3 className="font-bold text-lg">Recent Verifications</h3>
          </div>
          <div className="flex flex-col gap-4">
             {minimalPath && (
               <RecentItem name={filePath.split('\\').pop() || ''} date="Just now" status="Completed" icon="green" minimalPath={minimalPath} comprehensivePath={comprehensivePath} onOpen={openFileSystem} />
             )}
             {recentVerifications.map(item => (
                <RecentItem key={item.id} name={item.name} date={item.date.split(',')[0]} status={item.status} icon="blue" minimalPath={item.minimal_path} comprehensivePath={item.comprehensive_path} onOpen={openFileSystem} />
             ))}
             {recentVerifications.length === 0 && !minimalPath && (
               <div className="text-sm text-gray-400 text-center py-4">No recent history</div>
             )}
          </div>
        </div>

        {/* Verification Progress */}
        <div className="card flex flex-col items-center dark:bg-gray-800 dark:border-gray-700">
          <h3 className="font-bold text-lg mb-8 w-full">Verification Progress</h3>
          <div className="relative w-48 h-48">
            <svg className="w-full h-full -rotate-90 transform translate-y-6" viewBox="0 0 100 100">
              <circle cx="50" cy="50" r="40" fill="none" stroke="currentColor" className="text-brand-light dark:text-gray-700" strokeWidth="12" strokeLinecap="round" />
              <motion.circle 
                cx="50" cy="50" r="40" 
                fill="none" stroke="#1E6446" strokeWidth="12" strokeLinecap="round" 
                strokeDasharray={circumference} 
                strokeDashoffset={strokeDashoffset}
                transition={{ duration: 0.3 }} 
              />
            </svg>
            <div className="absolute inset-0 flex flex-col items-center justify-center translate-y-6">
              <span className="text-4xl font-bold">{progress}%</span>
              <span className="text-xs text-text-muted mt-1">{isVerifying ? 'In Progress' : (isFinished ? 'Completed' : 'Pending')}</span>
            </div>
          </div>
          <div className="flex items-center gap-4 mt-auto w-full justify-center text-xs text-text-muted font-medium">
            <div className="flex items-center gap-1.5"><div className="w-2.5 h-2.5 rounded-full bg-brand" /> Completed</div>
            <div className="flex items-center gap-1.5"><div className="w-2.5 h-2.5 rounded-full bg-brand-dark" /> In Progress</div>
          </div>
        </div>

        {/* Processing Time / Speed */}
        <div className="card bg-brand-dark text-white border-none relative overflow-hidden flex flex-col justify-between">
          <div className="absolute inset-0 opacity-20 pointer-events-none" style={{ backgroundImage: `repeating-radial-gradient(circle at 100% 100%, transparent 0, transparent 10px, #fff 10px, #fff 11px)` }} />
          <div className="relative z-10">
            <h3 className="font-semibold text-white/90">Processing Speed</h3>
            <div className="mt-4 text-[42px] font-bold leading-none tracking-tight">
              {stats.speed.toFixed(1)} <span className="text-xl">emails/s</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

// Subcomponents
function StatCard({ title, value, subtitle, isPrimary }: any) {
  return (
    <div className={`rounded-2xl p-6 relative overflow-hidden ${isPrimary ? 'bg-brand text-white' : 'bg-white dark:bg-gray-800 border border-gray-100 dark:border-gray-700 text-text-main dark:text-gray-100 shadow-sm'}`}>
      <div className="flex justify-between items-start mb-4">
        <h3 className={`font-semibold text-sm ${isPrimary ? 'text-white/90' : 'text-text-muted'}`}>{title}</h3>
        <div className={`w-8 h-8 rounded-full border flex items-center justify-center ${isPrimary ? 'border-white/20 bg-white/10 text-white' : 'border-gray-200 dark:border-gray-600 text-text-main dark:text-gray-400'}`}>
          <ArrowUpRight className="w-4 h-4" />
        </div>
      </div>
      <div className="text-[40px] font-bold leading-none mb-4">{value}</div>
      <div className="flex items-center gap-2 text-xs font-medium">
        <div className={`px-2 py-1 rounded bg-black/5 flex items-center gap-1 ${isPrimary ? 'text-brand-light' : 'text-brand'}`}>
          <ArrowUpRight className="w-3 h-3" />
        </div>
        <span className={isPrimary ? 'text-white/70' : 'text-text-muted'}>{subtitle}</span>
      </div>
    </div>
  );
}

function RecentItem({ name, date, status, icon, minimalPath, comprehensivePath, onOpen }: any) {
  const iconColors: Record<string, string> = { blue: 'bg-blue-500', green: 'bg-brand', yellow: 'bg-amber-400', purple: 'bg-purple-500' };
  const statusColors: Record<string, string> = { 'Completed': 'text-brand bg-brand/10', 'In Progress': 'text-amber-500 bg-amber-500/10', 'Pending': 'text-red-500 bg-red-500/10' };
  return (
    <div className="flex items-center justify-between p-3 rounded-xl hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors group">
      <div className="flex items-center gap-4 min-w-0">
        <div className={`w-10 h-10 rounded-full flex items-center justify-center text-white font-bold relative shrink-0`}>
           <div className={`absolute inset-0 rounded-full ${iconColors[icon]} opacity-20`} />
           <div className={`w-4 h-4 rounded-full ${iconColors[icon]}`} />
        </div>
        <div className="min-w-0 truncate">
          <h4 className="text-sm font-bold text-text-main dark:text-gray-200 truncate pr-2">{name}</h4>
          <p className="text-[11px] text-text-muted mt-0.5">{date}</p>
        </div>
      </div>
      
      <div className="flex items-center gap-2 shrink-0 pl-2">
        <div className={`text-[10px] font-bold px-3 py-1 rounded-md ${statusColors[status]} text-center`}>{status}</div>
        
        {onOpen && (minimalPath || comprehensivePath) && (
          <div className="flex gap-1 ml-1 border-l border-gray-200 dark:border-gray-700 pl-2 opacity-0 group-hover:opacity-100 transition-opacity">
            {minimalPath && (
              <button onClick={() => onOpen(minimalPath)} title="Open Minimal Report" className="p-1 text-brand hover:bg-brand/10 rounded-md transition-colors">
                <ExternalLink className="w-3.5 h-3.5" />
              </button>
            )}
            {comprehensivePath && (
              <button onClick={() => onOpen(comprehensivePath)} title="Open Comprehensive Report" className="p-1 text-brand hover:bg-brand/10 rounded-md transition-colors">
                <FolderOpen className="w-3.5 h-3.5" />
              </button>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
