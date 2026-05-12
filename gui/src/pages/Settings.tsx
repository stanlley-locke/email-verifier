import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Save, Moon, Sun, Settings as SettingsIcon } from 'lucide-react';

export default function SettingsPage() {
  const [settings, setSettings] = useState({
    theme: 'light',
    threads: 20,
    timeout: 10,
    no_smtp: false,
    auto_size_columns: true,
  });

  const [saving, setSaving] = useState(false);

  useEffect(() => {
    invoke<any>('get_settings').then(s => {
      setSettings(s);
      applyTheme(s.theme);
    }).catch(console.error);
  }, []);

  const applyTheme = (theme: string) => {
    if (theme === 'dark') {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
  };

  const handleSave = async () => {
    setSaving(true);
    try {
      await invoke('save_settings', { settings });
      applyTheme(settings.theme);
      // Optional: Add a toast notification here
    } catch (e) {
      console.error(e);
    } finally {
      setTimeout(() => setSaving(false), 500);
    }
  };

  return (
    <div className="flex flex-col gap-6 dark:text-white">
      <div className="flex justify-between items-end">
        <div>
          <h1 className="text-3xl font-bold text-text-main dark:text-white mb-2">Settings</h1>
          <p className="text-sm text-text-muted">Configure the verification engine and application appearance.</p>
        </div>
        <button 
          onClick={handleSave}
          className="px-6 py-3 bg-brand text-white rounded-full font-semibold text-sm hover:bg-brand/90 transition-all flex items-center gap-2 shadow-lg shadow-brand/20"
        >
          <Save className="w-4 h-4" />
          {saving ? 'Saved!' : 'Save Settings'}
        </button>
      </div>

      <div className="grid grid-cols-2 gap-6">
        {/* Theme Settings */}
        <div className="card dark:bg-gray-800 dark:border-gray-700 flex flex-col gap-6">
          <div className="flex items-center gap-3 border-b border-gray-100 dark:border-gray-700 pb-4">
            <div className="w-10 h-10 rounded-full bg-brand/10 text-brand flex items-center justify-center">
              <Sun className="w-5 h-5" />
            </div>
            <h2 className="text-lg font-bold">Appearance</h2>
          </div>
          
          <div>
            <label className="block text-sm font-semibold mb-3">Color Scheme</label>
            <div className="flex gap-4">
              <button 
                onClick={() => setSettings({ ...settings, theme: 'light' })}
                className={`flex-1 py-4 rounded-xl border-2 font-semibold flex flex-col items-center gap-2 transition-all ${settings.theme === 'light' ? 'border-brand bg-brand/5 text-brand' : 'border-gray-200 dark:border-gray-600 text-gray-500 hover:border-brand/30'}`}
              >
                <Sun className="w-6 h-6" /> Light (Donezo)
              </button>
              <button 
                onClick={() => setSettings({ ...settings, theme: 'dark' })}
                className={`flex-1 py-4 rounded-xl border-2 font-semibold flex flex-col items-center gap-2 transition-all ${settings.theme === 'dark' ? 'border-brand bg-brand/5 text-brand' : 'border-gray-200 dark:border-gray-600 text-gray-500 hover:border-brand/30'}`}
              >
                <Moon className="w-6 h-6" /> Dark (Obsidian)
              </button>
            </div>
          </div>
        </div>

        {/* Engine Settings */}
        <div className="card dark:bg-gray-800 dark:border-gray-700 flex flex-col gap-6">
          <div className="flex items-center gap-3 border-b border-gray-100 dark:border-gray-700 pb-4">
            <div className="w-10 h-10 rounded-full bg-brand/10 text-brand flex items-center justify-center">
              <SettingsIcon className="w-5 h-5" />
            </div>
            <h2 className="text-lg font-bold">Verification Engine</h2>
          </div>
          
          <div className="flex flex-col gap-5">
            <div>
              <div className="flex justify-between mb-2">
                <label className="text-sm font-semibold">Max Concurrent Threads</label>
                <span className="text-sm font-bold text-brand">{settings.threads}</span>
              </div>
              <input 
                type="range" 
                min="1" max="100" 
                value={settings.threads}
                onChange={e => setSettings({...settings, threads: parseInt(e.target.value)})}
                className="w-full accent-brand"
              />
            </div>

            <div>
              <div className="flex justify-between mb-2">
                <label className="text-sm font-semibold">SMTP Timeout (seconds)</label>
                <span className="text-sm font-bold text-brand">{settings.timeout}s</span>
              </div>
              <input 
                type="range" 
                min="1" max="30" 
                value={settings.timeout}
                onChange={e => setSettings({...settings, timeout: parseInt(e.target.value)})}
                className="w-full accent-brand"
              />
            </div>

            <div className="flex items-center justify-between mt-2 pt-4 border-t border-gray-100 dark:border-gray-700">
              <div>
                <p className="font-semibold text-sm">Disable SMTP Checks</p>
                <p className="text-xs text-text-muted mt-0.5">Skips connecting to mail servers (faster, less accurate).</p>
              </div>
              <label className="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" checked={settings.no_smtp} onChange={e => setSettings({...settings, no_smtp: e.target.checked})} className="sr-only peer" />
                <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all dark:border-gray-600 peer-checked:bg-brand"></div>
              </label>
            </div>

            <div className="flex items-center justify-between pt-2">
              <div>
                <p className="font-semibold text-sm">Auto-Size Columns</p>
                <p className="text-xs text-text-muted mt-0.5">Automatically adjust column widths in output Excel.</p>
              </div>
              <label className="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" checked={settings.auto_size_columns} onChange={e => setSettings({...settings, auto_size_columns: e.target.checked})} className="sr-only peer" />
                <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all dark:border-gray-600 peer-checked:bg-brand"></div>
              </label>
            </div>

          </div>
        </div>
      </div>
    </div>
  );
}
