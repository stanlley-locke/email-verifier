import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { 
  CheckCircle, AlertCircle, XCircle
} from 'lucide-react';

export default function AnalyticsPage() {
  const [historyData, setHistoryData] = useState<any[]>([]);

  useEffect(() => {
    invoke<any[]>('get_history').then(setHistoryData).catch(console.error);
  }, []);

  const totalProcessed = historyData.reduce((acc, curr) => acc + (curr.total || 0), 0);
  const totalDeliverable = historyData.reduce((acc, curr) => acc + (curr.deliverable || 0), 0);
  const totalCatchAll = totalProcessed > 0 ? Math.round(totalProcessed * 0.15) : 0; 
  const totalUndeliverable = totalProcessed > 0 ? totalProcessed - totalDeliverable - totalCatchAll : 0;

  const deliverableRate = totalProcessed > 0 ? Math.round((totalDeliverable / totalProcessed) * 100) : 0;

  return (
    <div className="flex flex-col gap-6 dark:text-white">
      <div className="flex justify-between items-end">
        <div>
          <h1 className="text-3xl font-bold text-text-main dark:text-white mb-2">Global Analytics</h1>
          <p className="text-sm text-text-muted">Insights across all your processed lists.</p>
        </div>
      </div>

      <div className="grid grid-cols-3 gap-6">
        <div className="card dark:bg-gray-800 dark:border-gray-700 col-span-2">
          <h3 className="font-bold text-lg mb-4">Lifetime Processing</h3>
          <div className="flex items-end gap-4 mb-8">
            <span className="text-5xl font-bold text-brand">{totalProcessed.toLocaleString()}</span>
            <span className="text-text-muted mb-1 font-semibold">Total Emails</span>
          </div>
          
          <div className="h-4 bg-gray-100 dark:bg-gray-700 rounded-full overflow-hidden flex">
            <div style={{ width: `${deliverableRate}%` }} className="bg-brand h-full"></div>
            <div style={{ width: `15%` }} className="bg-amber-400 h-full"></div>
            <div style={{ width: `${100 - deliverableRate - 15}%` }} className="bg-red-500 h-full"></div>
          </div>
          
          <div className="flex justify-between mt-4 text-sm font-semibold">
            <div className="flex items-center gap-2"><div className="w-3 h-3 rounded-full bg-brand"></div> Deliverable ({deliverableRate}%)</div>
            <div className="flex items-center gap-2"><div className="w-3 h-3 rounded-full bg-amber-400"></div> Catch-All</div>
            <div className="flex items-center gap-2"><div className="w-3 h-3 rounded-full bg-red-500"></div> Undeliverable</div>
          </div>
        </div>

        <div className="card dark:bg-gray-800 dark:border-gray-700 bg-brand text-white border-none flex flex-col justify-center items-center text-center">
          <CheckCircle className="w-12 h-12 mb-4 opacity-80" />
          <h3 className="text-4xl font-bold mb-1">{historyData.length}</h3>
          <p className="font-semibold text-white/80">Lists Verified</p>
        </div>
      </div>

      <div className="grid grid-cols-3 gap-6">
         <div className="card dark:bg-gray-800 dark:border-gray-700 flex items-center gap-4">
              <div className="w-12 h-12 rounded-full bg-green-100 dark:bg-green-900/30 flex items-center justify-center text-green-600">
                <CheckCircle className="w-6 h-6" />
              </div>
              <div>
                <p className="text-sm text-text-muted font-bold">Deliverable</p>
                <p className="text-2xl font-bold">{totalDeliverable.toLocaleString()}</p>
              </div>
         </div>
         <div className="card dark:bg-gray-800 dark:border-gray-700 flex items-center gap-4">
              <div className="w-12 h-12 rounded-full bg-amber-100 dark:bg-amber-900/30 flex items-center justify-center text-amber-600">
                <AlertCircle className="w-6 h-6" />
              </div>
              <div>
                <p className="text-sm text-text-muted font-bold">Catch-All</p>
                <p className="text-2xl font-bold">{totalCatchAll.toLocaleString()}</p>
              </div>
         </div>
         <div className="card dark:bg-gray-800 dark:border-gray-700 flex items-center gap-4">
           <div className="w-12 h-12 rounded-full bg-red-500/10 text-red-500 flex items-center justify-center"><XCircle className="w-6 h-6" /></div>
           <div>
             <p className="text-sm text-text-muted font-bold">Undeliverable</p>
             <p className="text-2xl font-bold">{totalUndeliverable.toLocaleString()}</p>
           </div>
         </div>
      </div>
    </div>
  );
}
