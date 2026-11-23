import React, { useState } from 'react';
import {
  ArrowLeftRight,
  Bluetooth,
  Calendar,
  Check,
  Columns2,
  FolderOpen,
  Layers,
  LayoutGrid,
  Lock,
  LogOut,
  Moon,
  Power,
  RefreshCw,
  Rows2,
  Search,
  Sparkles,
  Star,
  Terminal,
  Trash2,
  Volume2,
  VolumeX,
  Wifi,
  X,
} from 'lucide-react';
import { LumaConfig, TilingDirection } from '../utils/lumaConfig';
import {
  ActivePopup,
  SystemTrayItem,
  TrayBrandGlyph,
} from './CosmicTopPanel';

export interface AudioSessionRow {
  id: string;
  label: string;
  process: string;
  volume: number;
  muted: boolean;
}

export interface LauncherAppEntry {
  id: string;
  name: string;
  category: string;
  target: string;
  favorite: boolean;
}

interface CosmicPopupsProps {
  config: LumaConfig;
  activePopup: ActivePopup;
  onClosePopup: () => void;
  // Audio Mixer state
  masterVolume: number;
  masterMuted: boolean;
  onChangeMasterVolume: (vol: number) => void;
  onToggleMasterMute: () => void;
  audioSessions: AudioSessionRow[];
  onChangeSessionVolume: (id: string, vol: number) => void;
  onToggleSessionMute: (id: string) => void;
  // Network state
  wifiSsid: string;
  wifiConnected: boolean;
  onConnectWifi: (ssid: string) => void;
  // Bluetooth state
  bluetoothDevices: {
    id: string;
    name: string;
    service: string;
    connected: boolean;
  }[];
  onToggleBluetoothDevice: (id: string) => void;
  // Launcher state
  launcherApps: LauncherAppEntry[];
  onToggleFavoriteApp: (id: string) => void;
  onLaunchApp: (app: LauncherAppEntry) => void;
  // System Tray state
  trayItems: SystemTrayItem[];
  onToggleTrayItemActive: (id: SystemTrayItem['id']) => void;
  // Workspaces / Switcher
  activeWorkspace: number;
  onSelectWorkspace: (ws: number) => void;
  onOpenSettingsPage: (
    page: 'dock' | 'applets' | 'gpu' | 'appearance' | 'config_sync'
  ) => void;
  // COSMIC Tiling & Floating Window Manager controls
  onUpdateConfig: (updater: (prev: LumaConfig) => LumaConfig) => void;
  managedWindowCount: number;
  focusedWindowTitle: string | null;
  focusedWindowIsFloating: boolean;
  onToggleFocusedFloat: () => void;
  onFlipFocusedSplit: () => void;
  onSwapFocusedWindow: (dir: 'prev' | 'next') => void;
  onOpenThreeTilePreset: () => void;
  onSetAllWindowsMode: (mode: 'tiling' | 'floating') => void;
  onClearWorkspaceWindows: () => void;
}

export const CosmicPopups: React.FC<CosmicPopupsProps> = ({
  config,
  activePopup,
  onClosePopup,
  masterVolume,
  masterMuted,
  onChangeMasterVolume,
  onToggleMasterMute,
  audioSessions,
  onChangeSessionVolume,
  onToggleSessionMute,
  wifiSsid,
  wifiConnected,
  onConnectWifi,
  bluetoothDevices,
  onToggleBluetoothDevice,
  launcherApps,
  onToggleFavoriteApp,
  onLaunchApp,
  trayItems,
  onToggleTrayItemActive,
  activeWorkspace,
  onSelectWorkspace,
  onOpenSettingsPage,
  onUpdateConfig,
  managedWindowCount,
  focusedWindowTitle,
  focusedWindowIsFloating,
  onToggleFocusedFloat,
  onFlipFocusedSplit,
  onSwapFocusedWindow,
  onOpenThreeTilePreset,
  onSetAllWindowsMode,
  onClearWorkspaceWindows,
}) => {
  const [searchQuery, setSearchQuery] = useState<string>('');
  const [filterFavoritesOnly, setFilterFavoritesOnly] =
    useState<boolean>(false);

  if (!activePopup) return null;

  const popupSurface = config.dark
    ? 'bg-slate-900/95 text-slate-100 border border-white/15 shadow-2xl'
    : 'bg-white/95 text-slate-900 border border-slate-300 shadow-xl';

  const filteredApps = launcherApps.filter((app) => {
    if (filterFavoritesOnly && !app.favorite) return false;
    if (!searchQuery.trim()) return true;
    const q = searchQuery.toLowerCase();
    return (
      app.name.toLowerCase().includes(q) ||
      app.category.toLowerCase().includes(q) ||
      app.target.toLowerCase().includes(q)
    );
  });

  return (
    <>
      {/* 1. APPLICATION LAUNCHER POPUP */}
      {activePopup === 'launcher' && (
        <div
          role="dialog"
          aria-label="Luma COSMIC Application Launcher"
          data-testid="popup-launcher"
          style={{ backdropFilter: `blur(${config.gpuBlurRadius}px)` }}
          className={`fixed top-12 left-4 z-50 w-[min(460px,92vw)] rounded-2xl p-4 ${popupSurface}`}
        >
          <div className="flex items-center justify-between mb-3">
            <div>
              <h2 className="text-sm font-bold">
                Applications & System Destinations
              </h2>
              <p className="text-xs text-slate-400">
                Indexed Start Menu (.lnk / .exe) & curated COSMIC Settings
              </p>
            </div>
            <button
              type="button"
              onClick={onClosePopup}
              aria-label="Close Application Launcher"
              className="p-1 rounded-lg hover:bg-white/10 text-slate-400"
            >
              <X className="w-4 h-4" />
            </button>
          </div>

          <div className="flex items-center gap-2 mb-3">
            <div className="relative flex-1">
              <Search className="w-3.5 h-3.5 text-slate-400 absolute left-3 top-1/2 -translate-y-1/2" />
              <input
                type="text"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                placeholder="Search applications or files..."
                aria-label="Search applications"
                className="w-full pl-8 pr-3 py-2 rounded-xl bg-black/30 border border-white/10 text-xs focus:outline-none"
              />
            </div>
            <button
              type="button"
              onClick={() => setFilterFavoritesOnly((v) => !v)}
              className={`px-2.5 py-2 rounded-xl text-xs font-medium transition-colors whitespace-nowrap ${
                filterFavoritesOnly
                  ? 'bg-amber-400/20 text-amber-300 border border-amber-400/40'
                  : 'bg-white/5 text-slate-300 hover:bg-white/10'
              }`}
            >
              Favorites
            </button>
          </div>

          <div className="max-h-72 overflow-y-auto divide-y divide-white/10">
            {filteredApps.length === 0 ? (
              <div className="py-8 text-center text-xs text-slate-400">
                No matching applications found for "{searchQuery}".
              </div>
            ) : (
              filteredApps.map((app) => (
                <div
                  key={app.id}
                  className="py-2 px-2 flex items-center justify-between gap-3 rounded-lg hover:bg-white/5 transition-colors"
                >
                  <button
                    type="button"
                    onClick={() => {
                      onLaunchApp(app);
                      onClosePopup();
                    }}
                    className="flex-1 text-left min-w-0"
                  >
                    <div className="text-xs font-semibold truncate">
                      {app.name}
                    </div>
                    <div className="text-xs text-slate-400 font-mono truncate">
                      {app.category} · {app.target}
                    </div>
                  </button>

                  <button
                    type="button"
                    onClick={() => onToggleFavoriteApp(app.id)}
                    aria-label={
                      app.favorite
                        ? `Remove ${app.name} from favorites`
                        : `Add ${app.name} to favorites`
                    }
                    className={`p-1.5 rounded-lg transition-colors ${
                      app.favorite
                        ? 'text-amber-400 hover:bg-amber-400/10'
                        : 'text-slate-500 hover:text-slate-300'
                    }`}
                  >
                    <Star
                      className="w-3.5 h-3.5"
                      fill={app.favorite ? 'currentColor' : 'none'}
                    />
                  </button>
                </div>
              ))
            )}
          </div>
        </div>
      )}

      {/* 2. AUDIO MIXER POPUP */}
      {activePopup === 'audio' && (
        <div
          role="dialog"
          aria-label="Core Audio Session Mixer"
          data-testid="popup-audio-mixer"
          style={{ backdropFilter: `blur(${config.gpuBlurRadius}px)` }}
          className={`fixed top-12 right-12 z-50 w-88 rounded-2xl p-4 ${popupSurface}`}
        >
          <div className="flex items-center justify-between mb-3">
            <div>
              <h2 className="text-sm font-bold">Core Audio Mixer</h2>
              <p className="text-xs text-slate-400">
                IAudioEndpointVolume & per-process sessions
              </p>
            </div>
            <button
              type="button"
              onClick={onClosePopup}
              aria-label="Close Audio Mixer"
              className="p-1 rounded-lg hover:bg-white/10 text-slate-400"
            >
              <X className="w-4 h-4" />
            </button>
          </div>

          {/* Master Endpoint */}
          <div className="p-3 rounded-xl bg-black/25 border border-white/10 mb-3 space-y-2">
            <div className="flex items-center justify-between">
              <button
                type="button"
                onClick={onToggleMasterMute}
                className="flex items-center gap-2 text-xs font-semibold"
              >
                {masterMuted ? (
                  <VolumeX className="w-4 h-4 text-rose-400" />
                ) : (
                  <Volume2
                    style={{ color: config.accentColor }}
                    className="w-4 h-4"
                  />
                )}
                <span>Master Output</span>
              </button>
              <span
                style={{ color: config.accentColor }}
                className="text-xs font-mono tabular-nums"
              >
                {masterMuted ? 'Muted' : `${masterVolume}%`}
              </span>
            </div>
            <input
              type="range"
              min={0}
              max={100}
              value={masterVolume}
              aria-label="Master Volume"
              onChange={(e) => onChangeMasterVolume(Number(e.target.value))}
              className="w-full"
            />
          </div>

          {/* Per-Application Sessions */}
          <div className="space-y-2.5">
            {audioSessions.map((session) => (
              <div
                key={session.id}
                className="p-2.5 rounded-xl bg-white/5 space-y-1.5"
              >
                <div className="flex items-center justify-between text-xs">
                  <button
                    type="button"
                    onClick={() => onToggleSessionMute(session.id)}
                    className="flex items-center gap-2 font-medium truncate"
                  >
                    {session.muted ? (
                      <VolumeX className="w-3.5 h-3.5 text-rose-400 shrink-0" />
                    ) : (
                      <Volume2
                        style={{ color: config.accentColor }}
                        className="w-3.5 h-3.5 shrink-0"
                      />
                    )}
                    <span className="truncate">{session.label}</span>
                  </button>
                  <span className="font-mono tabular-nums text-slate-400">
                    {session.muted ? 'Muted' : `${session.volume}%`}
                  </span>
                </div>
                <input
                  type="range"
                  min={0}
                  max={100}
                  value={session.volume}
                  aria-label={`${session.label} Volume`}
                  onChange={(e) =>
                    onChangeSessionVolume(session.id, Number(e.target.value))
                  }
                  className="w-full"
                />
              </div>
            ))}
          </div>
        </div>
      )}

      {/* 3. NETWORK WI-FI POPUP */}
      {activePopup === 'network' && (
        <div
          role="dialog"
          aria-label="WLAN Network Connections"
          data-testid="popup-network"
          style={{ backdropFilter: `blur(${config.gpuBlurRadius}px)` }}
          className={`fixed top-12 right-24 z-50 w-80 rounded-2xl p-4 ${popupSurface}`}
        >
          <div className="flex items-center justify-between mb-3">
            <div>
              <h2 className="text-sm font-bold">Wi-Fi & WLAN Interfaces</h2>
              <p className="text-xs text-slate-400">
                {wifiConnected ? `Connected to ${wifiSsid}` : 'Disconnected'}
              </p>
            </div>
            <button
              type="button"
              onClick={onClosePopup}
              aria-label="Close Network Popup"
              className="p-1 rounded-lg hover:bg-white/10 text-slate-400"
            >
              <X className="w-4 h-4" />
            </button>
          </div>

          <div className="space-y-1.5">
            {[
              { ssid: 'System76-COSMIC-5G', signal: 98 },
              { ssid: 'Luma-Studio-Mesh', signal: 84 },
              { ssid: 'FiberLab-Guest', signal: 62 },
            ].map((net) => {
              const isCurrent = wifiConnected && wifiSsid === net.ssid;
              return (
                <button
                  key={net.ssid}
                  type="button"
                  onClick={() => onConnectWifi(net.ssid)}
                  className={`w-full flex items-center justify-between p-2.5 rounded-xl text-xs transition-colors ${
                    isCurrent
                      ? 'cosmic-accent-soft border'
                      : 'bg-white/5 hover:bg-white/10'
                  }`}
                >
                  <div className="flex items-center gap-2.5">
                    <Wifi
                      style={{ color: config.accentColor }}
                      className="w-3.5 h-3.5"
                    />
                    <span className="font-medium">{net.ssid}</span>
                  </div>
                  <div className="flex items-center gap-2 font-mono tabular-nums text-xs text-slate-400">
                    <span>{net.signal}%</span>
                    {isCurrent && (
                      <Check
                        style={{ color: config.accentColor }}
                        className="w-3.5 h-3.5"
                      />
                    )}
                  </div>
                </button>
              );
            })}
          </div>
        </div>
      )}

      {/* 4. BLUETOOTH AUDIO SINK POPUP */}
      {activePopup === 'bluetooth' && (
        <div
          role="dialog"
          aria-label="Bluetooth AudioSink Devices"
          data-testid="popup-bluetooth"
          style={{ backdropFilter: `blur(${config.gpuBlurRadius}px)` }}
          className={`fixed top-12 right-20 z-50 w-80 rounded-2xl p-4 ${popupSurface}`}
        >
          <div className="flex items-center justify-between mb-3">
            <div>
              <h2 className="text-sm font-bold">Bluetooth Devices</h2>
              <p className="text-xs text-slate-400">
                AudioSink (0x110B) & Handsfree (0x111E) endpoints
              </p>
            </div>
            <button
              type="button"
              onClick={onClosePopup}
              aria-label="Close Bluetooth Popup"
              className="p-1 rounded-lg hover:bg-white/10 text-slate-400"
            >
              <X className="w-4 h-4" />
            </button>
          </div>

          <div className="space-y-1.5">
            {bluetoothDevices.map((dev) => (
              <button
                key={dev.id}
                type="button"
                onClick={() => onToggleBluetoothDevice(dev.id)}
                className={`w-full flex items-center justify-between p-2.5 rounded-xl text-xs transition-colors ${
                  dev.connected
                    ? 'cosmic-accent-soft border'
                    : 'bg-white/5 hover:bg-white/10'
                }`}
              >
                <div className="flex items-center gap-2.5 text-left">
                  <Bluetooth
                    style={{ color: config.accentColor }}
                    className="w-3.5 h-3.5 shrink-0"
                  />
                  <div>
                    <div className="font-semibold">{dev.name}</div>
                    <div className="text-xs text-slate-400 font-mono">
                      {dev.service}
                    </div>
                  </div>
                </div>
                <span className="text-xs font-medium">
                  {dev.connected ? 'Connected' : 'Connect'}
                </span>
              </button>
            ))}
          </div>
        </div>
      )}

      {/* 5. CALENDAR / CLOCK POPUP */}
      {activePopup === 'calendar' && (
        <div
          role="dialog"
          aria-label="Clock and Calendar Applet Popup"
          style={{ backdropFilter: `blur(${config.gpuBlurRadius}px)` }}
          className={`fixed top-12 left-1/2 -translate-x-1/2 z-50 w-80 rounded-2xl p-4 ${popupSurface}`}
        >
          <div className="flex items-center justify-between mb-2">
            <div className="flex items-center gap-2">
              <Calendar
                style={{ color: config.accentColor }}
                className="w-4 h-4"
              />
              <h2 className="text-sm font-bold">23 November 2026</h2>
            </div>
            <button
              type="button"
              onClick={() => {
                onOpenSettingsPage('applets');
                onClosePopup();
              }}
              style={{ color: config.accentColor }}
              className="text-xs font-medium hover:underline"
            >
              Clock Format
            </button>
          </div>
          <p className="text-xs text-slate-400">
            Monday, 23 Nov · 11:13 am · Zone:{' '}
            <code
              style={{ color: config.accentColor }}
              className="font-mono font-semibold"
            >
              clock:{config.panelZones.clock}
            </code>
          </p>
        </div>
      )}

      {/* 6. WORKSPACE & WINDOW SWITCHER (Win + Tab) */}
      {activePopup === 'switcher' && (
        <div
          role="dialog"
          aria-label="Workspace and Window Switcher"
          style={{ backdropFilter: `blur(${config.gpuBlurRadius}px)` }}
          className={`fixed top-12 right-16 z-50 w-96 rounded-2xl p-4 ${popupSurface}`}
        >
          <div className="flex items-center justify-between mb-3">
            <h2 className="text-sm font-bold">
              IVirtualDesktopManager — Workspaces
            </h2>
            <button
              type="button"
              onClick={onClosePopup}
              aria-label="Close Workspace Switcher"
              className="p-1 rounded-lg hover:bg-white/10 text-slate-400"
            >
              <X className="w-4 h-4" />
            </button>
          </div>
          <div className="grid grid-cols-2 gap-2">
            {[1, 2, 3, 4].map((ws) => (
              <button
                key={ws}
                type="button"
                onClick={() => {
                  onSelectWorkspace(ws);
                  onClosePopup();
                }}
                className={`p-3 rounded-xl border text-left transition-all ${
                  activeWorkspace === ws
                    ? 'cosmic-accent-soft border'
                    : 'border-white/10 bg-white/5 hover:bg-white/10'
                }`}
              >
                <div className="text-xs font-bold font-mono tabular-nums">
                  Workspace {ws}
                </div>
                <div className="text-xs text-slate-400 mt-1">
                  {ws === 1
                    ? 'Brave Web Browser'
                    : ws === 2
                      ? 'Rust Analyzer & VS Code'
                      : ws === 3
                        ? 'WGPU Shader Pipeline'
                        : 'COSMIC Orbital Desktop'}
                </div>
              </button>
            ))}
          </div>
        </div>
      )}

      {/* 6B. SYSTEM TRAY POPUP (Shell_NotifyIcon / TrayNotifyWnd) */}
      {activePopup === 'tray' && (
        <div
          role="dialog"
          aria-label="System Tray Icons"
          data-testid="popup-tray"
          style={{ backdropFilter: `blur(${config.gpuBlurRadius}px)` }}
          className={`fixed top-12 right-28 z-50 w-76 rounded-2xl p-4 ${popupSurface}`}
        >
          <div className="flex items-center justify-between mb-3">
            <div>
              <h2 className="text-sm font-bold">System Tray</h2>
              <p className="text-xs text-slate-400">
                Background Shell_NotifyIcon services
              </p>
            </div>
            <button
              type="button"
              onClick={onClosePopup}
              aria-label="Close System Tray"
              className="p-1 rounded-lg hover:bg-white/10 text-slate-400"
            >
              <X className="w-4 h-4" />
            </button>
          </div>
          <div className="space-y-2">
            {trayItems.map((item) => (
              <div
                key={item.id}
                className="flex items-center justify-between gap-3 px-2.5 py-2 rounded-xl bg-white/5 border border-white/10"
              >
                <div className="flex items-center gap-2.5 min-w-0">
                  <div className="w-7 h-7 rounded-lg bg-white/10 flex items-center justify-center shrink-0">
                    <TrayBrandGlyph id={item.id} className="w-4 h-4" />
                  </div>
                  <div className="min-w-0">
                    <div className="text-xs font-semibold truncate">
                      {item.name}
                    </div>
                    <div className="text-xs text-slate-400 truncate">
                      {item.process} · {item.active ? item.status : 'Hidden'}
                    </div>
                  </div>
                </div>
                <button
                  type="button"
                  onClick={() => onToggleTrayItemActive(item.id)}
                  className={`px-2 py-1 rounded-lg text-xs font-medium shrink-0 transition-colors ${
                    item.active
                      ? 'bg-white/10 text-slate-200 hover:bg-white/20'
                      : 'cosmic-accent-soft border'
                  }`}
                >
                  {item.active ? 'Hide' : 'Show'}
                </button>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* 6C. COSMIC TILING & FLOATING POPOVER (GlazeWM SplitContainer Engine) */}
      {activePopup === 'tiling' && (
        <div
          role="dialog"
          aria-label="COSMIC Tiling and Floating Controls"
          data-testid="popup-tiling"
          style={{ backdropFilter: `blur(${config.gpuBlurRadius}px)` }}
          className={`fixed top-12 right-32 z-50 w-84 rounded-2xl p-4 space-y-3 ${popupSurface}`}
        >
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              {config.tilingEnabled ? (
                <LayoutGrid
                  style={{ color: config.accentColor }}
                  className="w-4 h-4"
                />
              ) : (
                <Layers className="w-4 h-4 text-amber-400" />
              )}
              <div>
                <h2 className="text-sm font-bold leading-tight">
                  COSMIC Tiling
                </h2>
                <p className="text-[11px] text-slate-400">
                  GlazeWM SplitContainer · Workspace {activeWorkspace}
                </p>
              </div>
            </div>
            <button
              type="button"
              onClick={onClosePopup}
              aria-label="Close Tiling Controls"
              className="p-1 rounded-lg hover:bg-white/10 text-slate-400"
            >
              <X className="w-4 h-4" />
            </button>
          </div>

          {/* Primary Auto-Tiling vs Floating Mode Switch */}
          <div className="p-3 rounded-xl bg-white/[0.05] border border-white/10 flex items-center justify-between gap-3">
            <div>
              <div className="text-xs font-semibold">Tile Windows</div>
              <div className="text-[11px] text-slate-400">
                {config.tilingEnabled
                  ? 'Auto-tiling enabled (Alt+Shift+T)'
                  : 'Floating mode active (Alt+Shift+T)'}
              </div>
            </div>
            <button
              type="button"
              role="switch"
              aria-checked={config.tilingEnabled}
              aria-label="Toggle Auto-Tiling Mode"
              onClick={() =>
                onUpdateConfig((prev) => ({
                  ...prev,
                  tilingEnabled: !prev.tilingEnabled,
                }))
              }
              style={
                config.tilingEnabled
                  ? { backgroundColor: config.accentColor }
                  : undefined
              }
              className={`relative w-10 h-5 rounded-full transition-colors shrink-0 ${
                config.tilingEnabled ? '' : 'bg-slate-600'
              }`}
            >
              <span
                className={`block w-4 h-4 rounded-full bg-white shadow transition-transform duration-150 ${
                  config.tilingEnabled ? 'translate-x-5' : 'translate-x-0.5'
                }`}
              />
            </button>
          </div>

          {/* Split Direction Selector */}
          <div className="space-y-1.5">
            <div className="flex items-center justify-between text-[11px] text-slate-400">
              <span>New Split Direction</span>
              <span className="font-mono">Alt+V</span>
            </div>
            <div className="grid grid-cols-2 gap-1.5">
              {(
                [
                  { id: 'horizontal', label: 'Horizontal', icon: Columns2 },
                  { id: 'vertical', label: 'Vertical', icon: Rows2 },
                ] as {
                  id: TilingDirection;
                  label: string;
                  icon: React.FC<{ className?: string }>;
                }[]
              ).map((dir) => {
                const Icon = dir.icon;
                const active = config.tilingDirection === dir.id;
                return (
                  <button
                    key={dir.id}
                    type="button"
                    onClick={() =>
                      onUpdateConfig((prev) => ({
                        ...prev,
                        tilingDirection: dir.id,
                      }))
                    }
                    className={`flex items-center justify-center gap-1.5 py-1.5 rounded-xl text-xs font-medium border transition-colors ${
                      active
                        ? 'cosmic-accent-soft'
                        : 'border-white/10 bg-white/5 text-slate-300 hover:bg-white/10'
                    }`}
                  >
                    <Icon className="w-3.5 h-3.5" />
                    <span>{dir.label}</span>
                  </button>
                );
              })}
            </div>
          </div>

          {/* Focused Window Quick Actions */}
          {managedWindowCount > 0 && focusedWindowTitle && (
            <div className="p-2.5 rounded-xl bg-black/25 border border-white/10 space-y-2">
              <div className="flex items-center justify-between text-[11px]">
                <span className="text-slate-400 truncate">
                  Active:{' '}
                  <strong className="text-slate-100">{focusedWindowTitle}</strong>
                </span>
                <span className="font-mono text-slate-400">
                  {managedWindowCount} open
                </span>
              </div>
              <div className="grid grid-cols-3 gap-1.5">
                <button
                  type="button"
                  onClick={onToggleFocusedFloat}
                  className="px-2 py-1.5 rounded-lg bg-white/10 hover:bg-white/15 text-[11px] font-medium flex items-center justify-center gap-1 transition-colors"
                >
                  {focusedWindowIsFloating ? (
                    <>
                      <LayoutGrid className="w-3 h-3" />
                      <span>Tile</span>
                    </>
                  ) : (
                    <>
                      <Layers className="w-3 h-3" />
                      <span>Float</span>
                    </>
                  )}
                </button>
                <button
                  type="button"
                  onClick={onFlipFocusedSplit}
                  className="px-2 py-1.5 rounded-lg bg-white/10 hover:bg-white/15 text-[11px] font-medium flex items-center justify-center gap-1 transition-colors"
                >
                  <Columns2 className="w-3 h-3" />
                  <span>Flip Split</span>
                </button>
                <button
                  type="button"
                  onClick={() => onSwapFocusedWindow('next')}
                  className="px-2 py-1.5 rounded-lg bg-white/10 hover:bg-white/15 text-[11px] font-medium flex items-center justify-center gap-1 transition-colors"
                >
                  <ArrowLeftRight className="w-3 h-3" />
                  <span>Swap</span>
                </button>
              </div>
            </div>
          )}

          {/* Gaps & COSMIC Active Hint */}
          <div className="space-y-2.5 pt-0.5">
            <div className="space-y-1">
              <div className="flex items-center justify-between text-xs">
                <span className="text-slate-300">Inner Window Gap</span>
                <span className="font-mono tabular-nums text-slate-400">
                  {config.tilingInnerGap}px
                </span>
              </div>
              <input
                type="range"
                min={0}
                max={24}
                value={config.tilingInnerGap}
                aria-label="Inner Window Gap"
                onChange={(e) =>
                  onUpdateConfig((prev) => ({
                    ...prev,
                    tilingInnerGap: Number(e.target.value),
                  }))
                }
                className="w-full"
              />
            </div>

            <div className="space-y-1">
              <div className="flex items-center justify-between text-xs">
                <span className="text-slate-300">Outer Screen Gap</span>
                <span className="font-mono tabular-nums text-slate-400">
                  {config.tilingOuterGap}px
                </span>
              </div>
              <input
                type="range"
                min={0}
                max={24}
                value={config.tilingOuterGap}
                aria-label="Outer Screen Gap"
                onChange={(e) =>
                  onUpdateConfig((prev) => ({
                    ...prev,
                    tilingOuterGap: Number(e.target.value),
                  }))
                }
                className="w-full"
              />
            </div>

            <div className="flex items-center justify-between pt-0.5">
              <span className="text-xs text-slate-300">
                COSMIC Active Focus Hint (2px)
              </span>
              <button
                type="button"
                role="switch"
                aria-checked={config.tilingActiveHint}
                aria-label="Toggle COSMIC Active Focus Hint"
                onClick={() =>
                  onUpdateConfig((prev) => ({
                    ...prev,
                    tilingActiveHint: !prev.tilingActiveHint,
                  }))
                }
                style={
                  config.tilingActiveHint
                    ? { backgroundColor: config.accentColor }
                    : undefined
                }
                className={`relative w-9 h-5 rounded-full transition-colors ${
                  config.tilingActiveHint ? '' : 'bg-slate-600'
                }`}
              >
                <span
                  className={`block w-3.5 h-3.5 rounded-full bg-white shadow transition-transform duration-150 ${
                    config.tilingActiveHint ? 'translate-x-4.5' : 'translate-x-0.5'
                  }`}
                />
              </button>
            </div>
          </div>

          {/* 1-Click Workspace Actions */}
          <div className="pt-2 border-t border-white/10 space-y-1.5">
            <button
              type="button"
              onClick={() => {
                onOpenThreeTilePreset();
                onClosePopup();
              }}
              style={{
                backgroundColor: config.accentColor,
                color: '#020617',
              }}
              className="w-full py-2 px-3 rounded-xl text-xs font-bold flex items-center justify-center gap-1.5 hover:opacity-95 transition-opacity"
            >
              <Sparkles className="w-3.5 h-3.5" />
              <span>Open 3-Tile Workspace</span>
            </button>

            {managedWindowCount > 0 && (
              <div className="grid grid-cols-3 gap-1.5">
                <button
                  type="button"
                  onClick={() => onSetAllWindowsMode('tiling')}
                  className="py-1.5 px-2 rounded-lg bg-white/5 hover:bg-white/10 text-[11px] font-medium text-slate-200 transition-colors"
                >
                  Tile All
                </button>
                <button
                  type="button"
                  onClick={() => onSetAllWindowsMode('floating')}
                  className="py-1.5 px-2 rounded-lg bg-white/5 hover:bg-white/10 text-[11px] font-medium text-slate-200 transition-colors"
                >
                  Float All
                </button>
                <button
                  type="button"
                  onClick={onClearWorkspaceWindows}
                  aria-label="Close All Workspace Windows"
                  className="py-1.5 px-2 rounded-lg bg-rose-500/15 hover:bg-rose-500/25 text-[11px] font-medium text-rose-300 flex items-center justify-center gap-1 transition-colors"
                >
                  <Trash2 className="w-3 h-3" />
                  <span>Clear</span>
                </button>
              </div>
            )}
          </div>
        </div>
      )}

      {/* 7. POWER & SESSION MENU */}
      {activePopup === 'power' && (
        <div
          role="dialog"
          aria-label="Power and Session Menu"
          style={{ backdropFilter: `blur(${config.gpuBlurRadius}px)` }}
          className={`fixed top-12 right-4 z-50 w-56 rounded-2xl p-2 ${popupSurface}`}
        >
          {[
            { label: 'Lock Workstation', icon: Lock },
            { label: 'Sleep Session', icon: Moon },
            { label: 'Sign Out (EWX_LOGOFF)', icon: LogOut },
            { label: 'Restart Windows', icon: RefreshCw },
            { label: 'Shut Down', icon: Power },
          ].map((item) => {
            const Icon = item.icon;
            return (
              <button
                key={item.label}
                type="button"
                onClick={onClosePopup}
                className="w-full flex items-center gap-2.5 px-3 py-2 rounded-xl text-xs font-medium hover:bg-white/10 text-left transition-colors"
              >
                <Icon className="w-3.5 h-3.5 text-slate-400" />
                <span>{item.label}</span>
              </button>
            );
          })}
        </div>
      )}
    </>
  );
};

interface DesktopSubWindowProps {
  title: string;
  kind: 'terminal' | 'files';
  config: LumaConfig;
  onClose: () => void;
}

export const DesktopSubWindow: React.FC<DesktopSubWindowProps> = ({
  title,
  kind,
  config,
  onClose,
}) => {
  return (
    <div
      role="dialog"
      aria-label={title}
      style={{ backdropFilter: `blur(${config.gpuBlurRadius}px)` }}
      className={`fixed bottom-24 left-8 z-30 w-[min(500px,90vw)] rounded-2xl border shadow-2xl overflow-hidden ${
        config.dark
          ? 'bg-slate-950/90 border-white/15 text-slate-100'
          : 'bg-white/95 border-slate-300 text-slate-900'
      }`}
    >
      <div className="flex items-center justify-between px-4 py-2.5 border-b border-white/10 bg-black/30">
        <div className="flex items-center gap-2 text-xs font-semibold">
          {kind === 'terminal' ? (
            <Terminal className="w-3.5 h-3.5 text-emerald-400" />
          ) : (
            <FolderOpen className="w-3.5 h-3.5 text-amber-400" />
          )}
          <span>{title}</span>
        </div>
        <button
          type="button"
          onClick={onClose}
          aria-label={`Close ${title}`}
          className="p-1 rounded-lg hover:bg-white/10 text-slate-400"
        >
          <X className="w-3.5 h-3.5" />
        </button>
      </div>

      {kind === 'terminal' ? (
        <div className="p-4 font-mono text-xs space-y-1.5 text-slate-300">
          <div className="text-emerald-400">
            PS C:\Users\Luma\rust_shell&gt; cargo run --bin luma_iced
          </div>
          <div>
            Compiling rust_shell v0.1.0 (src/shell/gpu.rs, src/bin/luma_iced.rs)
          </div>
          <div style={{ color: config.accentColor }}>
            WGPU Backend: {config.gpuBackend.toUpperCase()} · Shader:{' '}
            {config.gpuShader} · Blur: {config.gpuBlurRadius}px
          </div>
          <div>
            Dock: {config.dockStyle} ({config.position}) · Icon {config.iconSize}
            px · Mag {config.magnification}%
          </div>
        </div>
      ) : (
        <div className="p-4 text-xs space-y-2">
          <div className="font-mono text-xs text-slate-400">
            %APPDATA%\LumaShell\
          </div>
          <div className="grid grid-cols-2 gap-2">
            {[
              'settings.ini',
              'settings.ini.bak',
              'favorites.txt',
              'luma.log',
            ].map((file) => (
              <div
                key={file}
                className="p-2.5 rounded-xl bg-white/5 border border-white/10 flex items-center gap-2 font-mono"
              >
                <FolderOpen className="w-3.5 h-3.5 text-amber-400" />
                <span>{file}</span>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};
