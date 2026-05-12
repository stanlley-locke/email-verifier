import { useState, useEffect } from 'react';
import { NavLink, Outlet } from 'react-router-dom';
import { invoke } from '@tauri-apps/api/core';
import { 
  Search, Bell, LayoutDashboard, CheckSquare, Calendar, 
  BarChart2, Settings, HelpCircle, X, Save
} from 'lucide-react';

export default function Layout() {
  const [showNotifications, setShowNotifications] = useState(false);
  const [notifications, setNotifications] = useState<any[]>([]);
  const [activeToast, setActiveToast] = useState<{title: string, message: string} | null>(null);
  
  const [showProfileModal, setShowProfileModal] = useState(false);
  const [profile, setProfile] = useState({ name: 'Stanley M.', email: 'stanley@email.com', avatar: 'male' });
  const [editProfile, setEditProfile] = useState({ ...profile });

  useEffect(() => {
    invoke<any>('get_profile').then(p => {
      setProfile(p);
      setEditProfile(p);
    }).catch(console.error);

    const handleNotification = (e: any) => {
      const newNotif = { id: Date.now(), ...e.detail, read: false };
      setNotifications(prev => [newNotif, ...prev].slice(0, 5));
      setActiveToast({ title: newNotif.title, message: newNotif.message });
      setTimeout(() => setActiveToast(null), 5000); // hide toast after 5s
    };

    window.addEventListener('new-notification', handleNotification);
    return () => window.removeEventListener('new-notification', handleNotification);
  }, []);

  const handleSaveProfile = async () => {
    try {
      await invoke('save_profile', { profile: editProfile });
      setProfile(editProfile);
      setShowProfileModal(false);
    } catch (e) {
      console.error(e);
    }
  };

  const unreadCount = notifications.filter(n => !n.read).length;

  const markAllRead = () => {
    setNotifications(prev => prev.map(n => ({ ...n, read: true })));
    setShowNotifications(false);
  };

  return (
    <div className="min-h-screen bg-bg-sidebar dark:bg-gray-900 flex p-4 gap-6 text-text-main dark:text-gray-200">
      {/* Sidebar */}
      <aside className="w-[240px] flex flex-col justify-between shrink-0 h-full max-h-[calc(100vh-2rem)] rounded-3xl bg-bg-sidebar dark:bg-gray-800">
        <div>
          <div className="flex items-center gap-3 px-4 py-6">
            <img 
              src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEAAAABACAYAAACqaXHeAAAACXBIWXMAAAsTAAALEwEAmpwYAAADOklEQVR4nO2aTWgTQRTH1yp+K3jx4MGPkx9VFIVapO2bDS0EKSrIvBdrRRH1JogKIgbakyJ4EQ9eBMGDBwU9KIJo8l7aqkUrVLCXqsXvgyAtKKIWjUzT0KRWzTYfu4nzhwdLdnfe//1m2J2ZjeNYWVlZWVlZWVlZTahw9875wBgFoR4Q/KyEkkGOlEd8pIROGO9OPlLxyEbF+N7voiYNg/FdKKFrJlU88I4VIDTkdxEFiEE3ppd7732meADMF2YkCN71VHyoQ6/y23Shw9MocIUO+G244KOAcV/OAIAxWoEAoh4AUHvlAaB2CyBXgQVAFoAKgOlgAmD6oIROAmPM76LGAm8A43EQelt0AMC4zVyjr+ipSui6/8XTZSfpTDGeVJy2lgCAbk1fV3tfz1KMHT4Wf0c/1dMzfO8qPgChIWC9Ln1t0+3WOSAkJS+esQNYzx3zjKuB6WPRAajRZSYwLc3aMxDqKlXx5vnT3NM8O52/PqGXGU8lfQuA4PPGTr04fY8xBII3iw8Ar4ZvhWek87qxliWKacCf1yDTQOZIAIZpSvBiEYs/5yTbqrJ6XvClv/MAxtduLFKdfT8eBKZvBSuc6Ysrem9mjgbRa5Tgm6BMhAZVXENmG25cbwDGF/kXj/1uDNeO86dGcgZpJghCX4FpT2Y7dZ0tC4DxAjD+8Nxe6p7z4zc2TQ6TK8BTYbyU+YROjwYl9MCD2cfAWJvtafdMYDxbFmsBEOo122pZjSbbqlzBzaOzx+EJ7htWjNeAI+HMB90IwFikGhiflNdiiPG7EjxlJkrj22+617pQxXE7MB01YY7Nb79dZyZZTKdTbZXpahDMaypBNL5X/yazzgDGCDC9qpzlMGO/Ejrc2BVZ9Ke85pwSPKKEnhU6v/8AJA2CfgJTn3lYAtMZE6PHfeZcsfIGB4D4ExYAWwDtFkCuAguALAAVANMWgFgASZ8AYPQ/B6D3Vx4APJQzgAahlX4bLnQ0CNXnDMAoWN/+8ux9oV4vy/ERmT8Ved14DGh8CiVwvTMZhRK65l9fXIIcqS9Yus7JR5u6tswzn54V00NDsxx6HAS7ldAx4z2v4q2srKysrKysrJzK1S83FUikV/Vp4QAAAABJRU5ErkJggg==" 
              alt="Logo" 
              className="w-8 h-8 rounded-md" 
            />
            <span className="text-lg font-bold">Email Verifier</span>
          </div>

          <div className="mt-4 px-2">
            <p className="text-xs font-semibold text-text-muted px-4 mb-4">MENU</p>
            <nav className="flex flex-col gap-1">
              <SidebarItem to="/" icon={LayoutDashboard} label="Dashboard" />
              <SidebarItem to="/history" icon={CheckSquare} label="History" />
              <SidebarItem to="/calendar" icon={Calendar} label="Calendar" />
              <SidebarItem to="/analytics" icon={BarChart2} label="Analytics" />
            </nav>

            <p className="text-xs font-semibold text-text-muted px-4 mb-4 mt-8">GENERAL</p>
            <nav className="flex flex-col gap-1">
              <SidebarItem to="/settings" icon={Settings} label="Settings" />
              <SidebarItem to="/help" icon={HelpCircle} label="Help" />
            </nav>
          </div>
        </div>

        <div className="px-4 py-6">
          <div className="bg-brand-light/30 dark:bg-gray-700/50 rounded-2xl p-4 border border-brand/10 dark:border-gray-700 relative overflow-hidden">
            <div className="absolute top-0 right-0 w-16 h-16 bg-brand/10 rounded-full blur-xl transform translate-x-1/2 -translate-y-1/2" />
            <h4 className="font-bold text-sm text-text-main dark:text-white mb-1">System Status</h4>
            <p className="text-xs text-text-muted mb-4 leading-relaxed">Engine is online and fully updated to v0.1.0.</p>
            <button className="w-full py-2 bg-white dark:bg-gray-800 text-text-main dark:text-white rounded-lg text-xs font-semibold shadow-sm border border-gray-100 dark:border-gray-600 hover:border-brand/30 transition-colors">
              Check Updates
            </button>
          </div>
        </div>
      </aside>

      {/* Main Content */}
      <main className="flex-1 flex flex-col h-full max-h-[calc(100vh-2rem)] overflow-y-auto bg-white dark:bg-gray-900 rounded-[2.5rem] shadow-[0_0_40px_rgba(0,0,0,0.02)] border border-gray-50 dark:border-gray-800">
        
        {/* Topbar */}
        <header className="px-8 py-6 flex items-center justify-between sticky top-0 bg-white/80 dark:bg-gray-900/80 backdrop-blur-md z-20">
          <div className="relative w-96">
            <Search className="w-5 h-5 absolute left-4 top-1/2 -translate-y-1/2 text-gray-400" />
            <input 
              type="text" 
              placeholder="Search history..." 
              className="w-full bg-bg-main dark:bg-gray-800 border-none rounded-full py-3 pl-12 pr-4 text-sm focus:ring-2 focus:ring-brand/20 outline-none dark:text-white"
            />
            <div className="absolute right-4 top-1/2 -translate-y-1/2 flex items-center gap-1 text-xs text-gray-400 dark:text-gray-500 bg-white dark:bg-gray-700 px-2 py-1 rounded shadow-sm border border-gray-100 dark:border-gray-600 font-medium">
              ⌘F
            </div>
          </div>

          <div className="flex items-center gap-6">
            <div className="relative flex items-center gap-4 text-gray-400">
              <button 
                onClick={() => setShowNotifications(!showNotifications)}
                className="p-2 text-gray-400 hover:text-text-main dark:hover:text-white transition-colors relative"
              >
                <Bell className="w-6 h-6" />
                {unreadCount > 0 && (
                  <span className="absolute top-1 right-1 w-3 h-3 border-2 border-white dark:border-gray-900 bg-red-500 rounded-full animate-pulse" />
                )}
              </button>

              {showNotifications && (
                <div className="absolute right-0 top-12 mt-2 w-80 bg-white dark:bg-gray-800 rounded-2xl shadow-xl border border-gray-100 dark:border-gray-700 z-50 overflow-hidden text-text-main dark:text-white">
                  <div className="flex justify-between items-center p-4 border-b border-gray-100 dark:border-gray-700">
                    <h3 className="font-bold text-sm">Notifications</h3>
                    <button onClick={markAllRead} className="text-xs text-brand hover:underline">Mark all read</button>
                  </div>
                  <div className="max-h-64 overflow-y-auto">
                    {notifications.length === 0 ? (
                      <div className="p-4 text-center text-sm text-text-muted">No recent notifications</div>
                    ) : (
                      notifications.map(n => (
                        <div key={n.id} className={`p-4 border-b border-gray-50 dark:border-gray-700/50 last:border-0 ${n.read ? 'opacity-60' : 'bg-brand/5'}`}>
                          <h4 className="text-sm font-semibold">{n.title}</h4>
                          <p className="text-xs text-text-muted mt-1">{n.message}</p>
                        </div>
                      ))
                    )}
                  </div>
                </div>
              )}
            </div>
            
            <button 
              onClick={() => setShowProfileModal(true)}
              className="flex items-center gap-3 pl-6 border-l border-gray-100 dark:border-gray-700 hover:opacity-80 transition-opacity text-left"
            >
              <div className="w-10 h-10 rounded-full bg-brand-light dark:bg-brand-dark border-2 border-brand/20 shadow-sm flex items-center justify-center overflow-hidden text-xl">
                {profile.avatar === 'female' ? '👩‍💼' : '👨‍💼'}
              </div>
              <div>
                <p className="text-sm font-semibold text-text-main dark:text-white">{profile.name}</p>
                <p className="text-xs text-text-muted">{profile.email}</p>
              </div>
            </button>
          </div>
        </header>

        {/* Page Outlet */}
        <div className="px-8 pb-8 flex-1">
          <Outlet />
        </div>
      </main>

      {/* Profile Modal */}
      {showProfileModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-sm">
          <div className="bg-white dark:bg-gray-900 rounded-[2rem] p-8 w-[400px] shadow-2xl relative border border-gray-100 dark:border-gray-800">
            <button onClick={() => setShowProfileModal(false)} className="absolute top-6 right-6 text-gray-400 hover:text-text-main dark:hover:text-white">
              <X className="w-5 h-5" />
            </button>
            <h2 className="text-2xl font-bold mb-2 text-text-main dark:text-white">Edit Profile</h2>
            <p className="text-sm text-text-muted mb-8">Update your personal details.</p>

            <div className="flex flex-col gap-5">
              <div>
                <label className="text-xs font-bold text-text-muted uppercase mb-2 block">Display Name</label>
                <input 
                  type="text" 
                  value={editProfile.name}
                  onChange={e => setEditProfile({ ...editProfile, name: e.target.value })}
                  className="w-full bg-gray-50 dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-xl px-4 py-3 text-sm font-semibold outline-none focus:border-brand text-text-main dark:text-white"
                />
              </div>
              
              <div>
                <label className="text-xs font-bold text-text-muted uppercase mb-2 block">Email Address</label>
                <input 
                  type="text" 
                  value={editProfile.email}
                  onChange={e => setEditProfile({ ...editProfile, email: e.target.value })}
                  className="w-full bg-gray-50 dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-xl px-4 py-3 text-sm font-semibold outline-none focus:border-brand text-text-main dark:text-white"
                />
              </div>

              <div>
                <label className="text-xs font-bold text-text-muted uppercase mb-2 block">Avatar Style</label>
                <div className="flex gap-2 text-text-main dark:text-white">
                  <button 
                    onClick={() => setEditProfile({ ...editProfile, avatar: 'male' })}
                    className={`flex-1 py-3 rounded-xl border flex flex-col items-center gap-2 transition-all ${editProfile.avatar === 'male' ? 'border-brand bg-brand/5 text-brand dark:text-brand-light' : 'border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800 hover:border-brand/30'}`}
                  >
                    <span className="text-2xl">👨‍💼</span>
                    <span className="text-xs font-semibold">Male</span>
                  </button>
                  <button 
                    onClick={() => setEditProfile({ ...editProfile, avatar: 'female' })}
                    className={`flex-1 py-3 rounded-xl border flex flex-col items-center gap-2 transition-all ${editProfile.avatar === 'female' ? 'border-brand bg-brand/5 text-brand dark:text-brand-light' : 'border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800 hover:border-brand/30'}`}
                  >
                    <span className="text-2xl">👩‍💼</span>
                    <span className="text-xs font-semibold">Female</span>
                  </button>
                </div>
              </div>
            </div>

            <button onClick={handleSaveProfile} className="w-full mt-8 py-4 bg-brand text-white rounded-xl font-semibold hover:bg-brand/90 transition-all flex items-center justify-center gap-2 shadow-lg shadow-brand/20">
              <Save className="w-5 h-5" /> Save Changes
            </button>
          </div>
        </div>
      )}

      {/* Toast Notification */}
      {activeToast && (
        <div className="fixed bottom-6 right-6 z-[100] bg-white dark:bg-gray-800 rounded-2xl p-4 shadow-2xl border border-gray-100 dark:border-gray-700 flex items-start gap-4 animate-in slide-in-from-bottom-5 fade-in duration-300 w-80">
          <div className="w-10 h-10 rounded-full bg-brand/10 flex items-center justify-center shrink-0">
            <CheckSquare className="w-5 h-5 text-brand" />
          </div>
          <div className="flex-1">
            <h4 className="font-bold text-sm text-text-main dark:text-gray-200">{activeToast.title}</h4>
            <p className="text-xs text-text-muted mt-1">{activeToast.message}</p>
          </div>
          <button onClick={() => setActiveToast(null)} className="text-gray-400 hover:text-text-main">
            <X className="w-4 h-4" />
          </button>
        </div>
      )}
    </div>
  );
}

function SidebarItem({ to, icon: Icon, label }: any) {
  return (
    <NavLink 
      to={to}
      className={({ isActive }) => 
        `flex items-center justify-between px-4 py-3 rounded-xl cursor-pointer transition-colors ${isActive ? 'bg-brand/10 text-brand font-semibold dark:bg-brand/20 dark:text-brand-light' : 'text-text-muted hover:bg-gray-50 dark:hover:bg-gray-700 hover:text-text-main dark:hover:text-white font-medium'}`
      }
    >
      {({ isActive }) => (
        <div className="flex items-center gap-3">
          <Icon className={`w-5 h-5 ${isActive ? 'text-brand' : ''}`} />
          <span className="text-sm">{label}</span>
        </div>
      )}
    </NavLink>
  );
}
