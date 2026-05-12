import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Calendar as CalendarIcon, Clock, Plus } from 'lucide-react';

export default function CalendarPage() {
  const [reminders, setReminders] = useState<any[]>([]);

  useEffect(() => {
    invoke<any[]>('get_reminders').then(setReminders).catch(console.error);
  }, []);

  async function handleAddReminder() {
    const title = prompt("Enter reminder title:");
    if (!title) return;
    
    const newRem = {
      id: Date.now().toString(),
      title,
      date: new Date().toISOString().split('T')[0],
      time: "10:00 AM",
      status: "Pending"
    };

    await invoke('add_reminder', { item: newRem });
    setReminders([...reminders, newRem]);
  }

  return (
    <div className="flex flex-col gap-6">
      <div className="flex justify-between items-end">
        <div>
          <h1 className="text-3xl font-bold text-text-main mb-2">Calendar</h1>
          <p className="text-sm text-text-muted">Schedule local reminders to verify your email lists.</p>
        </div>
        <button onClick={handleAddReminder} className="px-6 py-3 bg-brand text-white rounded-full font-semibold text-sm hover:bg-brand/90 transition-all flex items-center gap-2 shadow-lg shadow-brand/20">
          <Plus className="w-4 h-4" />
          Add Reminder
        </button>
      </div>

      <div className="card">
        <div className="grid grid-cols-7 gap-4 mb-4">
          {['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'].map(day => (
            <div key={day} className="text-center font-bold text-text-muted text-sm pb-2 border-b border-gray-100">{day}</div>
          ))}
          {/* Simple mock calendar grid */}
          {Array.from({ length: 35 }).map((_, i) => (
            <div key={i} className={`h-24 p-2 border border-gray-50 rounded-xl ${i === 12 ? 'bg-brand/5 border-brand/20' : 'bg-white'}`}>
              <span className={`text-sm font-semibold ${i === 12 ? 'text-brand' : 'text-text-main'}`}>{i < 31 ? i + 1 : ''}</span>
              {i === 12 && (
                <div className="mt-2 bg-brand text-white text-[10px] p-1 rounded font-medium">Verify Q3 Leads</div>
              )}
            </div>
          ))}
        </div>
      </div>

      <h3 className="font-bold text-lg mt-4">Upcoming Reminders</h3>
      <div className="grid grid-cols-2 gap-4">
        {reminders.map(rem => (
          <div key={rem.id} className="card flex items-center justify-between hover:border-brand/30 transition-colors cursor-pointer">
            <div className="flex items-center gap-4">
              <div className="w-12 h-12 bg-brand/10 text-brand rounded-full flex items-center justify-center">
                <CalendarIcon className="w-5 h-5" />
              </div>
              <div>
                <h4 className="font-bold text-text-main">{rem.title}</h4>
                <div className="flex gap-3 text-xs text-text-muted mt-1 font-medium">
                  <span className="flex items-center gap-1"><CalendarIcon className="w-3 h-3" /> {rem.date}</span>
                  <span className="flex items-center gap-1"><Clock className="w-3 h-3" /> {rem.time}</span>
                </div>
              </div>
            </div>
            <div className="px-3 py-1 bg-gray-100 text-gray-500 text-xs font-bold rounded-md">
              {rem.status}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
