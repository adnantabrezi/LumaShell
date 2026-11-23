import React from 'react';
import {
  BatteryCharging,
  Bell,
  BellOff,
  Bluetooth,
  ChevronUp,
  ClipboardList,
  Layers,
  LayoutGrid,
  Music,
  Pause,
  Play,
  Power,
  Settings,
  Volume2,
  VolumeX,
  Wifi,
} from 'lucide-react';
import { AppletId, LumaConfig, PanelZone } from '../utils/lumaConfig';

export type ActivePopup =
  | null
  | 'launcher'
  | 'switcher'
  | 'tiling'
  | 'tray'
  | 'audio'
  | 'network'
  | 'bluetooth'
  | 'power'
  | 'calendar'
  | 'notifications';

export interface SystemTrayItem {
  id: 'discord' | 'nvidia' | 'steam' | 'onedrive';
  name: string;
  process: string;
  status: string;
  active: boolean;
}

interface CosmicTopPanelProps {
  config: LumaConfig;
  activeWorkspace: number;
  onSelectWorkspace: (ws: number) => void;
  activePopup: ActivePopup;
  onTogglePopup: (popup: ActivePopup) => void;
  onToggleSettings: () => void;
  managedWindowCount: number;
  trayItems: SystemTrayItem[];
  onSelectTrayItem: (id: SystemTrayItem['id']) => void;
  masterVolume: number;
  masterMuted: boolean;
  activeAudioSessions: number;
  wifiSsid: string;
  wifiConnected: boolean;
  bluetoothConnected: boolean;
  batteryPercent: number;
  doNotDisturb: boolean;
  onToggleDoNotDisturb: () => void;
  mediaPlaying: boolean;
  mediaTrack: string;
  onToggleMediaPlay: () => void;
}

export const TrayBrandGlyph: React.FC<{
  id: SystemTrayItem['id'];
  className?: string;
}> = ({ id, className = 'w-3.5 h-3.5' }) => {
  switch (id) {
    case 'discord':
      return (
        <svg viewBox="0 0 24 24" className={className} aria-hidden="true">
          <path
            d="M19.5 5.6A16.5 16.5 0 0 0 15.4 4.3l-.5 1a15.3 15.3 0 0 0-5.8 0l-.5-1a16.5 16.5 0 0 0-4.1 1.3C2 9.4 1.3 13.1 1.6 16.7a16.7 16.7 0 0 0 5 2.5l1.1-1.7a10.7 10.7 0 0 1-1.7-.8l.4-.3a11.8 11.8 0 0 0 11.2 0l.4.3c-.6.3-1.1.6-1.7.8l1.1 1.7a16.7 16.7 0 0 0 5-2.5c.4-4.2-.7-7.8-2.9-11.1Z"
            fill="#818cf8"
          />
          <circle cx="9" cy="13" r="1.4" fill="#0f172a" />
          <circle cx="15" cy="13" r="1.4" fill="#0f172a" />
        </svg>
      );
    case 'nvidia':
      return (
        <svg viewBox="0 0 24 24" className={className} aria-hidden="true">
          <rect x="2.5" y="4" width="19" height="16" rx="4" fill="#16a34a" />
          <path
            d="M6.5 12c2.2-3 8.8-3 11 0-2.2 3-8.8 3-11 0Z"
            fill="none"
            stroke="#f8fafc"
            strokeWidth="1.8"
            strokeLinecap="round"
            strokeLinejoin="round"
          />
          <circle cx="12" cy="12" r="2" fill="#f8fafc" />
        </svg>
      );
    case 'steam':
      return (
        <svg viewBox="0 0 24 24" className={className} aria-hidden="true">
          <circle cx="12" cy="12" r="9.5" fill="#1d4ed8" />
          <circle
            cx="15"
            cy="9.2"
            r="2.7"
            fill="none"
            stroke="#e2e8f0"
            strokeWidth="1.8"
          />
          <circle cx="9.2" cy="15.2" r="2.2" fill="#e2e8f0" />
          <path
            d="M10.8 13.7l2.5-2.6M4.2 12.8l4.2 1.8"
            stroke="#e2e8f0"
            strokeWidth="1.8"
            strokeLinecap="round"
          />
        </svg>
      );
    case 'onedrive':
      return (
        <svg viewBox="0 0 24 24" className={className} aria-hidden="true">
          <path
            d="M7.5 18h10a4 4 0 0 0 .8-7.9A5.5 5.5 0 0 0 7.7 8.4 4.2 4.2 0 0 0 7.5 18Z"
            fill="#38bdf8"
          />
        </svg>
      );
  }
};

export const CosmicTopPanel: React.FC<CosmicTopPanelProps> = ({
  config,
  activeWorkspace,
  onSelectWorkspace,
  activePopup,
  onTogglePopup,
  onToggleSettings,
  managedWindowCount,
  trayItems,
  onSelectTrayItem,
  masterVolume,
  masterMuted,
  activeAudioSessions,
  wifiSsid,
  wifiConnected,
  bluetoothConnected,
  batteryPercent,
  doNotDisturb,
  onToggleDoNotDisturb,
  mediaPlaying,
  mediaTrack,
  onToggleMediaPlay,
}) => {
  const isIsland = config.panelStyle === 'island';
  const isTransparent = config.panelStyle === 'transparent';
  const highContrast = config.highContrast;

  // Matches reference screenshot: "23 Nov, 11:13 am"
  const formatClockLabel = () => {
    const timeStr = config.clock24h ? '11:13' : '11:13 am';
    if (config.dateFormat === 'none') {
      return timeStr;
    }
    if (config.dateFormat === 'long') {
      return `Monday, 23 November · ${timeStr}`;
    }
    return `23 Nov, ${timeStr}`;
  };

  const renderAppletButton = (id: AppletId) => {
    if (!config.appletEnabled[id]) return null;

    const baseBtnClass = `flex items-center gap-1.5 px-2 py-1 rounded-lg text-xs font-medium transition-colors whitespace-nowrap shrink-0 ${
      highContrast
        ? 'bg-black text-white border border-yellow-400 hover:bg-neutral-900'
        : isTransparent
          ? 'text-white/95 hover:bg-white/15 drop-shadow-[0_1px_2px_rgba(0,0,0,0.85)]'
          : config.dark
            ? 'text-slate-100 hover:bg-white/10 active:bg-white/15'
            : 'text-slate-800 hover:bg-slate-900/10 active:bg-slate-900/15'
    }`;

    switch (id) {
      case 'clock':
        return (
          <button
            key="clock"
            type="button"
            onClick={() => onTogglePopup('calendar')}
            aria-label={`Clock applet: ${formatClockLabel()}`}
            className={`${baseBtnClass} tabular-nums font-semibold tracking-tight ${
              activePopup === 'calendar' ? 'cosmic-accent-soft border' : ''
            }`}
          >
            <span>{formatClockLabel()}</span>
          </button>
        );

      case 'media':
        return (
          <button
            key="media"
            type="button"
            onClick={onToggleMediaPlay}
            aria-label={`Media playback applet: ${mediaTrack}`}
            title={`${mediaTrack} (${mediaPlaying ? 'Playing' : 'Paused'})`}
            className={baseBtnClass}
          >
            <Music
              style={{
                color: isTransparent ? '#ffffff' : config.accentColor,
              }}
              className="w-3.5 h-3.5 shrink-0 opacity-90"
            />
            {!isTransparent && (
              <>
                <span className="hidden xl:inline max-w-[130px] truncate">
                  {mediaTrack}
                </span>
                {mediaPlaying ? (
                  <Pause className="w-3 h-3 shrink-0 opacity-85" />
                ) : (
                  <Play className="w-3 h-3 opacity-70 shrink-0" />
                )}
              </>
            )}
          </button>
        );

      case 'network':
        return (
          <button
            key="network"
            type="button"
            onClick={() => onTogglePopup('network')}
            aria-label={`Network applet: ${wifiConnected ? wifiSsid : 'Offline'}`}
            className={`${baseBtnClass} ${
              activePopup === 'network' ? 'cosmic-accent-soft border' : ''
            }`}
          >
            <Wifi
              style={{
                color: !wifiConnected
                  ? '#fb7185'
                  : isTransparent
                    ? '#ffffff'
                    : config.accentColor,
              }}
              className="w-3.5 h-3.5 shrink-0"
            />
            {!isTransparent && (
              <span className="hidden xl:inline">
                {wifiConnected ? wifiSsid : 'Offline'}
              </span>
            )}
          </button>
        );

      case 'volume':
        return (
          <button
            key="volume"
            type="button"
            onClick={() => onTogglePopup('audio')}
            aria-label={`Volume applet: ${masterVolume}%`}
            title={`Master Volume: ${masterVolume}% (${activeAudioSessions} active sessions)`}
            className={`${baseBtnClass} tabular-nums ${
              activePopup === 'audio' ? 'cosmic-accent-soft border' : ''
            }`}
          >
            {masterMuted ? (
              <VolumeX className="w-3.5 h-3.5 text-rose-400 shrink-0" />
            ) : (
              <Volume2
                style={{
                  color: isTransparent ? '#ffffff' : config.accentColor,
                }}
                className="w-3.5 h-3.5 shrink-0"
              />
            )}
            {!isTransparent && (
              <span>{masterMuted ? 'Muted' : `${masterVolume}%`}</span>
            )}
          </button>
        );

      case 'bluetooth':
        return (
          <button
            key="bluetooth"
            type="button"
            onClick={() => onTogglePopup('bluetooth')}
            aria-label="Bluetooth applet"
            className={`${baseBtnClass} ${
              activePopup === 'bluetooth' ? 'cosmic-accent-soft border' : ''
            }`}
          >
            <Bluetooth
              style={{
                color: !bluetoothConnected
                  ? '#94a3b8'
                  : isTransparent
                    ? '#ffffff'
                    : config.accentColor,
              }}
              className="w-3.5 h-3.5 shrink-0"
            />
          </button>
        );

      case 'battery':
        return (
          <button
            key="battery"
            type="button"
            onClick={onToggleSettings}
            aria-label={`Battery applet: ${batteryPercent}%`}
            title={`Battery: ${batteryPercent}%`}
            className={`${baseBtnClass} tabular-nums`}
          >
            <BatteryCharging
              className={`w-3.5 h-3.5 shrink-0 ${
                isTransparent ? 'text-white' : 'text-emerald-400'
              }`}
            />
            {!isTransparent && <span>{batteryPercent}%</span>}
          </button>
        );

      case 'notifications':
        return (
          <button
            key="notifications"
            type="button"
            onClick={onToggleDoNotDisturb}
            aria-label={`Notifications applet: ${
              doNotDisturb ? 'Quiet time enabled' : 'Notifications active'
            }`}
            className={baseBtnClass}
          >
            {doNotDisturb ? (
              <BellOff className="w-3.5 h-3.5 text-amber-400 shrink-0" />
            ) : (
              <Bell
                style={{
                  color: isTransparent ? '#ffffff' : config.accentColor,
                }}
                className="w-3.5 h-3.5 shrink-0"
              />
            )}
          </button>
        );
    }
  };

  const getZoneApplets = (zone: PanelZone) =>
    config.appletOrder
      .filter((id) => config.panelZones[id] === zone && config.appletEnabled[id])
      .map((id) => renderAppletButton(id));

  const panelSurfaceStyle = highContrast
    ? 'bg-black border-2 border-yellow-400 text-white'
    : isTransparent
      ? 'bg-transparent border-transparent text-white'
      : config.dark
        ? 'bg-slate-900/88 border border-white/15 text-slate-100 shadow-lg'
        : 'bg-white/92 border border-slate-300/80 text-slate-900 shadow-md';

  return (
    <header
      aria-label="Luma COSMIC Top Panel"
      style={{
        height: `${config.panelHeight}px`,
        marginTop: isIsland ? `${config.panelMargin}px` : '0px',
        marginLeft: isIsland ? `${config.panelMargin * 2}px` : '0px',
        marginRight: isIsland ? `${config.panelMargin * 2}px` : '0px',
        backdropFilter: isTransparent
          ? 'none'
          : `blur(${config.gpuBlurRadius}px)`,
      }}
      className={`relative z-40 grid grid-cols-[1fr_auto_1fr] items-center px-3.5 transition-all duration-150 ${panelSurfaceStyle} ${
        isIsland ? 'rounded-xl' : 'border-t-0 border-x-0'
      }`}
    >
      {/* ZONE 1: Panel Start / Left — Workspaces 1 2 3 (4) + Launcher/Switcher */}
      <div
        data-testid="panel-zone-left"
        className="justify-self-start flex items-center gap-2 min-w-0"
      >
        <div
          className="flex items-center gap-1"
          role="group"
          aria-label="Virtual Desktop Workspaces"
        >
          {[1, 2, 3, 4].map((ws) => {
            const isSelected = activeWorkspace === ws;
            return (
              <button
                key={ws}
                type="button"
                onClick={() => onSelectWorkspace(ws)}
                aria-label={`Switch to Workspace ${ws}`}
                style={
                  isSelected && !isTransparent
                    ? { backgroundColor: config.accentColor, color: '#020617' }
                    : undefined
                }
                className={`w-6 h-6 rounded-full text-xs font-bold tabular-nums transition-all flex items-center justify-center ${
                  isSelected
                    ? isTransparent
                      ? 'bg-white/30 text-white shadow-sm ring-1 ring-white/40'
                      : 'shadow-sm'
                    : isTransparent
                      ? 'text-white/85 hover:text-white hover:bg-white/10'
                      : 'text-slate-400 hover:text-slate-200'
                }`}
              >
                {ws}
              </button>
            );
          })}
        </div>

        <button
          type="button"
          onClick={() => onTogglePopup('launcher')}
          aria-label="Toggle Luma Application Launcher"
          title="Applications & Clipboard Overview"
          className={`p-1.5 rounded-lg text-xs transition-colors ${
            activePopup === 'launcher'
              ? 'bg-white/25 text-white'
              : 'text-white/85 hover:bg-white/15'
          }`}
        >
          <ClipboardList className="w-3.5 h-3.5" />
        </button>

        {/* Left Zone Configured Applets */}
        <div className="flex items-center gap-1 pl-1">
          {getZoneApplets('left')}
        </div>
      </div>

      {/* ZONE 2: Panel Center — True Viewport Horizontal Center ("23 Nov, 11:13 am") */}
      <div
        data-testid="panel-zone-center"
        className="justify-self-center flex items-center justify-center gap-1.5"
      >
        {getZoneApplets('center')}
      </div>

      {/* ZONE 3: Panel End / Right — Keyboard Layout + Tray + Applets + Settings + Power */}
      <div
        data-testid="panel-zone-right"
        className="justify-self-end flex items-center gap-1 shrink-0"
      >
        <button
          type="button"
          onClick={() => onTogglePopup('tiling')}
          data-testid="panel-tiling-button"
          aria-label={`Toggle COSMIC Tiling and Floating Menu (${config.tilingEnabled ? 'Auto-Tiling active' : 'Floating mode'})`}
          title={
            config.tilingEnabled
              ? 'COSMIC Auto-Tiling Active (Alt+Shift+T)'
              : 'Floating Window Mode (Alt+Shift+T)'
          }
          style={
            activePopup === 'tiling'
              ? {
                  backgroundColor: 'rgba(255,255,255,0.22)',
                  color: config.accentColor,
                }
              : undefined
          }
          className={`flex items-center gap-1 px-1.5 py-1 rounded-lg text-xs transition-colors drop-shadow-[0_1px_2px_rgba(0,0,0,0.85)] ${
            activePopup === 'tiling'
              ? ''
              : config.tilingEnabled
                ? 'text-white/95 hover:bg-white/15'
                : 'text-white/65 hover:bg-white/15 hover:text-white'
          }`}
        >
          {config.tilingEnabled ? (
            <LayoutGrid className="w-3.5 h-3.5" />
          ) : (
            <Layers className="w-3.5 h-3.5" />
          )}
          {managedWindowCount > 0 && (
            <span className="text-[10px] font-bold tabular-nums opacity-90">
              {managedWindowCount}
            </span>
          )}
        </button>

        <button
          type="button"
          onClick={() => onTogglePopup('switcher')}
          aria-label="Toggle Activities and Window Switcher"
          title="Keyboard Layout & Workspace Switcher"
          className="px-1.5 py-1 rounded-md text-xs font-semibold tracking-wider text-white/90 hover:bg-white/15 drop-shadow-[0_1px_2px_rgba(0,0,0,0.85)]"
        >
          US
        </button>

        {/* System Tray Overflow Chevron + Active Background Tray App Icons */}
        <div
          data-testid="panel-system-tray"
          role="group"
          aria-label="System Tray Icons"
          className="flex items-center gap-0.5 px-1"
        >
          <button
            type="button"
            onClick={() => onTogglePopup('tray')}
            aria-label="Toggle System Tray Overflow Menu"
            title="Show hidden system tray icons"
            className={`p-1 rounded-md transition-colors drop-shadow-[0_1px_2px_rgba(0,0,0,0.85)] ${
              activePopup === 'tray'
                ? 'bg-white/20 text-white'
                : 'text-white/80 hover:bg-white/15 hover:text-white'
            }`}
          >
            <ChevronUp
              className={`w-3 h-3 transition-transform duration-150 ${
                activePopup === 'tray' ? 'rotate-180' : ''
              }`}
            />
          </button>

          {trayItems
            .filter((item) => item.active)
            .map((item) => (
              <button
                key={item.id}
                type="button"
                onClick={() => onSelectTrayItem(item.id)}
                aria-label={`System tray: ${item.name} (${item.status})`}
                title={`${item.name} — ${item.status}`}
                className={`relative p-1 rounded-md transition-colors hover:bg-white/15 drop-shadow-[0_1px_2px_rgba(0,0,0,0.85)] ${
                  activePopup === 'tray' ? 'bg-white/10' : ''
                }`}
              >
                <TrayBrandGlyph id={item.id} className="w-3.5 h-3.5" />
              </button>
            ))}
        </div>

        {getZoneApplets('right')}

        <button
          type="button"
          onClick={onToggleSettings}
          aria-label="Open Luma COSMIC Settings"
          title="COSMIC Desktop Settings"
          className="p-1.5 rounded-lg text-white/90 hover:bg-white/15 transition-colors drop-shadow-[0_1px_2px_rgba(0,0,0,0.85)]"
        >
          <Settings className="w-3.5 h-3.5" />
        </button>

        <button
          type="button"
          onClick={() => onTogglePopup('power')}
          aria-label="Open Power and Session Menu"
          title="Power & Session Menu"
          className={`p-1.5 rounded-lg transition-colors drop-shadow-[0_1px_2px_rgba(0,0,0,0.85)] ${
            activePopup === 'power'
              ? 'bg-rose-500 text-white'
              : 'text-white/90 hover:bg-white/15'
          }`}
        >
          <Power className="w-3.5 h-3.5" />
        </button>
      </div>
    </header>
  );
};
