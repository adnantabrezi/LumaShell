import React, { useState } from 'react';
import {
  ArrowDown,
  ArrowUp,
  BatteryCharging,
  Bell,
  Bluetooth,
  Calendar,
  Check,
  Copy,
  Cpu,
  Download,
  FileCode2,
  Image as ImageIcon,
  LayoutGrid,
  Maximize2,
  Minimize2,
  Monitor,
  Music,
  Palette,
  PanelTop,
  Plus,
  RotateCcw,
  Sliders,
  Type,
  Upload,
  Volume2,
  Wifi,
  X,
} from 'lucide-react';
import {
  ALL_APPLETS,
  AppletId,
  COSMIC_ACCENTS,
  DateFormatMode,
  defaultLumaConfig,
  DockLauncherId,
  DockPosition,
  DockStyle,
  FontFamilyPreset,
  formatAppletOrder,
  formatDockOrder,
  GpuBackendMode,
  GpuShaderPreset,
  LumaConfig,
  moveAppletToZone,
  PanelStyle,
  PanelZone,
  parseSettingsIni,
  reorderAppletInZone,
  reorderDockItem,
  serializeSettingsIni,
  VectorWallpaperPreset,
  WallpaperMode,
} from '../utils/lumaConfig';
import {
  RUST_GPU_RS_SOURCE,
  RUST_ICED_WGPU_SOURCE,
} from '../utils/rustSources';

export type SettingsPageId =
  | 'dock'
  | 'applets'
  | 'appearance'
  | 'gpu'
  | 'config_sync';

interface CosmicSettingsWindowProps {
  config: LumaConfig;
  onChangeConfig: (updater: (prev: LumaConfig) => LumaConfig) => void;
  activePage: SettingsPageId;
  onSelectPage: (page: SettingsPageId) => void;
  onClose: () => void;
  animateShader: boolean;
  onToggleAnimateShader: () => void;
  onLaunchDockApp: (id: DockLauncherId) => void;
}

const APPLET_METADATA: Record<
  AppletId,
  {
    title: string;
    subtitle: string;
    icon: React.FC<{ className?: string; style?: React.CSSProperties }>;
  }
> = {
  clock: {
    title: 'Clock & Date',
    subtitle: 'Time display, 12h/24h format, and localized date string',
    icon: Calendar,
  },
  media: {
    title: 'Media Transport',
    subtitle: 'WinRT GlobalSystemMediaTransportControls play/pause session',
    icon: Music,
  },
  network: {
    title: 'Network & Wi-Fi',
    subtitle: 'WLAN SSID, signal bars, and Ethernet adapter status',
    icon: Wifi,
  },
  volume: {
    title: 'Audio & Session Mixer',
    subtitle: 'Core Audio master endpoint and per-process volume mixer',
    icon: Volume2,
  },
  bluetooth: {
    title: 'Bluetooth Radio',
    subtitle: 'Paired AudioSink / Handsfree radios and connection status',
    icon: Bluetooth,
  },
  battery: {
    title: 'Power & Battery',
    subtitle: 'AC line status and remaining charge percentage',
    icon: BatteryCharging,
  },
  notifications: {
    title: 'Notifications & Quiet Time',
    subtitle: 'SHQueryUserNotificationState and Do Not Disturb toggle',
    icon: Bell,
  },
};

const DOCK_LAUNCHER_NAMES: Record<
  DockLauncherId,
  { title: string; binary: string }
> = {
  home: { title: 'Application Launcher', binary: 'LumaLauncher (Win+Alt+L)' },
  files: { title: 'Files Explorer', binary: 'explorer.exe (Win+E)' },
  terminal: { title: 'COSMIC Terminal', binary: 'wt.exe / powershell.exe' },
  browser: { title: 'Web Browser', binary: 'msedge.exe / default https' },
  settings: { title: 'Luma COSMIC Settings', binary: 'LumaSettings' },
};

export const CosmicSettingsWindow: React.FC<CosmicSettingsWindowProps> = ({
  config,
  onChangeConfig,
  activePage,
  onSelectPage,
  onClose,
  animateShader,
  onToggleAnimateShader,
  onLaunchDockApp,
}) => {
  const [isMaximized, setIsMaximized] = useState<boolean>(false);
  const [copiedIni, setCopiedIni] = useState<boolean>(false);
  const [iniDraft, setIniDraft] = useState<string>(() =>
    serializeSettingsIni(config)
  );
  const [iniFeedback, setIniFeedback] = useState<string | null>(null);
  const [activeZoneTab, setActiveZoneTab] = useState<'all' | PanelZone>('all');

  const currentIniText = serializeSettingsIni(config);

  const handleCopyIni = () => {
    if (navigator.clipboard) {
      navigator.clipboard.writeText(currentIniText).catch(() => {});
    }
    setCopiedIni(true);
    setTimeout(() => setCopiedIni(false), 1800);
  };

  const handleDownloadIni = () => {
    const blob = new Blob([currentIniText], { type: 'text/plain;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'settings.ini';
    a.click();
    URL.revokeObjectURL(url);
  };

  const handleWallpaperUpload = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = () => {
      if (typeof reader.result === 'string') {
        onChangeConfig((c) => ({
          ...c,
          wallpaperMode: 'custom_image',
          customWallpaperUrl: reader.result as string,
        }));
      }
    };
    reader.readAsDataURL(file);
  };

  const applyPanelPreset = (
    preset: 'cosmic' | 'macos' | 'compact' | 'developer'
  ) => {
    onChangeConfig((prev) => {
      if (preset === 'cosmic') {
        return {
          ...prev,
          panelStyle: 'island',
          appletOrder: [
            'clock',
            'media',
            'network',
            'volume',
            'bluetooth',
            'battery',
            'notifications',
          ],
          panelZones: {
            clock: 'center',
            media: 'left',
            network: 'right',
            volume: 'right',
            bluetooth: 'right',
            battery: 'right',
            notifications: 'right',
          },
        };
      }
      if (preset === 'macos') {
        return {
          ...prev,
          panelStyle: 'full',
          appletOrder: [
            'media',
            'bluetooth',
            'network',
            'volume',
            'battery',
            'clock',
            'notifications',
          ],
          panelZones: {
            media: 'right',
            bluetooth: 'right',
            network: 'right',
            volume: 'right',
            battery: 'right',
            clock: 'right',
            notifications: 'right',
          },
        };
      }
      if (preset === 'developer') {
        return {
          ...prev,
          panelStyle: 'island',
          appletOrder: [
            'clock',
            'battery',
            'network',
            'media',
            'volume',
            'bluetooth',
            'notifications',
          ],
          panelZones: {
            clock: 'left',
            battery: 'left',
            network: 'center',
            media: 'center',
            volume: 'right',
            bluetooth: 'right',
            notifications: 'right',
          },
        };
      }
      return {
        ...prev,
        panelStyle: 'full',
        panelZones: {
          clock: 'center',
          media: 'right',
          network: 'right',
          volume: 'right',
          bluetooth: 'right',
          battery: 'right',
          notifications: 'right',
        },
      };
    });
  };

  const navItems: {
    id: SettingsPageId;
    label: string;
    subtitle: string;
    icon: React.FC<{ className?: string; style?: React.CSSProperties }>;
  }[] = [
    {
      id: 'dock',
      label: 'Dock',
      subtitle: 'Position, magnification & launchers',
      icon: LayoutGrid,
    },
    {
      id: 'applets',
      label: 'Panel & Applets',
      subtitle: 'Left, Center & Right panel zones',
      icon: PanelTop,
    },
    {
      id: 'appearance',
      label: 'Global Accent & Wallpaper',
      subtitle: 'Accent color, wallpaper & crisp fonts',
      icon: Palette,
    },
    {
      id: 'gpu',
      label: 'GPU Compositor',
      subtitle: 'WGPU shaders, SDF & glass blur',
      icon: Cpu,
    },
    {
      id: 'config_sync',
      label: 'settings.ini & Rust',
      subtitle: 'Config serialization & WGPU source',
      icon: FileCode2,
    },
  ];

  const surfaceBg = config.dark
    ? 'bg-slate-900/94 text-slate-100 border-white/15'
    : 'bg-white/96 text-slate-900 border-slate-300';

  const cardClass = config.dark
    ? 'bg-slate-950/60 border border-white/10 rounded-xl p-5'
    : 'bg-slate-50 border border-slate-200 rounded-xl p-5';

  const activePillStyle: React.CSSProperties = {
    backgroundColor: config.accentColor,
    color: '#020617',
  };

  return (
    <section
      aria-label="COSMIC Desktop Settings Window"
      data-testid="cosmic-settings-window"
      style={{
        backdropFilter: `blur(${config.gpuBlurRadius}px)`,
      }}
      className={`relative z-30 mx-auto transition-all duration-200 border shadow-2xl flex flex-col overflow-hidden ${surfaceBg} ${
        isMaximized
          ? 'w-[calc(100vw-24px)] h-[calc(100vh-128px)] mt-2 rounded-xl'
          : 'w-[min(1100px,95vw)] h-[min(690px,calc(100vh-138px))] mt-3 rounded-2xl'
      }`}
    >
      {/* Window Title Bar */}
      <div
        className={`flex items-center justify-between px-5 py-3 border-b ${
          config.dark
            ? 'border-white/10 bg-slate-950/45'
            : 'border-slate-200 bg-slate-100/80'
        }`}
      >
        <div className="flex items-center gap-3">
          <Sliders
            style={{ color: config.accentColor }}
            className="w-4 h-4 shrink-0"
          />
          <div className="flex items-center gap-2">
            <h1 className="text-sm font-semibold tracking-tight">
              COSMIC Settings — Luma Shell
            </h1>
            <span
              className={`text-xs ${
                config.dark ? 'text-slate-400' : 'text-slate-500'
              }`}
            >
              · Desktop / {navItems.find((n) => n.id === activePage)?.label}
            </span>
          </div>
        </div>

        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={() => {
              const defaults = defaultLumaConfig();
              onChangeConfig(() => defaults);
              setIniDraft(serializeSettingsIni(defaults));
            }}
            className={`flex items-center gap-1.5 px-2.5 py-1 rounded-lg text-xs font-medium transition-colors whitespace-nowrap ${
              config.dark
                ? 'bg-white/5 text-slate-200 hover:bg-white/10'
                : 'bg-slate-200/70 text-slate-700 hover:bg-slate-200'
            }`}
          >
            <RotateCcw className="w-3.5 h-3.5" />
            <span>Reset Defaults</span>
          </button>

          <button
            type="button"
            onClick={() => setIsMaximized((v) => !v)}
            aria-label={
              isMaximized
                ? 'Restore Settings window size'
                : 'Maximize Settings window'
            }
            className={`p-1.5 rounded-lg transition-colors ${
              config.dark
                ? 'hover:bg-white/10 text-slate-400'
                : 'hover:bg-slate-200 text-slate-600'
            }`}
          >
            {isMaximized ? (
              <Minimize2 className="w-3.5 h-3.5" />
            ) : (
              <Maximize2 className="w-3.5 h-3.5" />
            )}
          </button>

          <button
            type="button"
            onClick={onClose}
            aria-label="Close Settings window"
            className={`p-1.5 rounded-lg transition-colors ${
              config.dark
                ? 'hover:bg-rose-500/20 text-slate-400 hover:text-rose-300'
                : 'hover:bg-rose-100 text-slate-600 hover:text-rose-700'
            }`}
          >
            <X className="w-4 h-4" />
          </button>
        </div>
      </div>

      {/* Main Split Layout: COSMIC Sidebar + Active Settings Viewport */}
      <div className="flex flex-1 min-h-0 overflow-hidden">
        {/* Left Navigation Sidebar */}
        <aside
          aria-label="Settings Categories"
          className={`w-64 shrink-0 border-r p-3 flex flex-col justify-between overflow-y-auto ${
            config.dark
              ? 'border-white/10 bg-slate-950/35'
              : 'border-slate-200 bg-slate-50/90'
          }`}
        >
          <div className="space-y-1">
            <div className="px-3 py-1.5 text-xs font-semibold text-slate-400">
              COSMIC Desktop
            </div>
            {navItems.map((item) => {
              const Icon = item.icon;
              const isSelected = activePage === item.id;
              return (
                <button
                  key={item.id}
                  type="button"
                  onClick={() => onSelectPage(item.id)}
                  className={`w-full flex items-center gap-3 px-3 py-2.5 rounded-xl text-left transition-colors ${
                    isSelected
                      ? 'cosmic-accent-soft border'
                      : config.dark
                        ? 'text-slate-200 hover:bg-white/5'
                        : 'text-slate-700 hover:bg-slate-200/60'
                  }`}
                >
                  <Icon
                    style={
                      isSelected ? { color: config.accentColor } : undefined
                    }
                    className={`w-4 h-4 shrink-0 ${
                      isSelected ? '' : 'text-slate-400'
                    }`}
                  />
                  <div className="min-w-0">
                    <div className="text-xs font-semibold truncate">
                      {item.label}
                    </div>
                    <div
                      className={`text-xs truncate ${
                        config.dark ? 'text-slate-400' : 'text-slate-500'
                      }`}
                    >
                      {item.subtitle}
                    </div>
                  </div>
                </button>
              );
            })}
          </div>

          {/* Bottom Live Global Theme Summary */}
          <div
            className={`p-3 rounded-xl text-xs space-y-1.5 ${
              config.dark
                ? 'bg-white/5 text-slate-300'
                : 'bg-slate-200/70 text-slate-700'
            }`}
          >
            <div className="font-semibold flex items-center justify-between">
              <span className="flex items-center gap-1.5">
                <span
                  style={{ backgroundColor: config.accentColor }}
                  className="w-2.5 h-2.5 rounded-full"
                />
                <span>Global Sync Active</span>
              </span>
              <span
                style={{ color: config.accentColor }}
                className="font-mono text-xs"
              >
                {config.gpuBackend.toUpperCase()}
              </span>
            </div>
            <div className="text-xs text-slate-400 leading-relaxed">
              Accent, wallpaper, crisp typography, dock, and applets sync live
              with <code className="font-mono">settings.ini</code>.
            </div>
          </div>
        </aside>

        {/* Right Scrollable Page Viewport */}
        <main className="flex-1 overflow-y-auto p-6 space-y-6">
          {/* ========================================================= */}
          {/* PAGE 1: COSMIC DOCK SETTINGS                              */}
          {/* ========================================================= */}
          {activePage === 'dock' && (
            <div className="space-y-6" data-testid="settings-page-dock">
              <div className="flex flex-wrap items-center justify-between gap-4">
                <div>
                  <h2 className="text-lg font-bold tracking-tight">
                    COSMIC Dock
                  </h2>
                  <p className="text-xs text-slate-400 mt-0.5">
                    Configure dock style, screen edge alignment, smooth cosine
                    magnification, and pinned application launchers.
                  </p>
                </div>

                <div className="flex items-center gap-2 text-xs text-slate-300 font-mono tabular-nums">
                  <span>Icon {config.iconSize}px</span>
                  <span>·</span>
                  <span>Mag {config.magnification}%</span>
                  <span>·</span>
                  <span>Alpha {config.transparency}</span>
                </div>
              </div>

              {/* Card 1: Dock Style & Alignment */}
              <div className={cardClass}>
                <h3 className="text-sm font-semibold mb-4">
                  01. Dock Style & Screen Alignment
                </h3>

                <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                  <div className="space-y-2">
                    <span className="block text-xs font-medium text-slate-400">
                      Surface Geometry Mode
                    </span>
                    <div className="grid grid-cols-2 sm:grid-cols-4 gap-2">
                      {(
                        [
                          {
                            id: 'pill',
                            title: 'COSMIC Pill',
                            desc: 'Stadium capsule',
                          },
                          {
                            id: 'transparent',
                            title: 'Borderless',
                            desc: 'Floating icons',
                          },
                          {
                            id: 'island',
                            title: 'Glass Island',
                            desc: 'Rounded box',
                          },
                          {
                            id: 'panel',
                            title: 'Full Edge',
                            desc: 'Bottom bar',
                          },
                        ] as { id: DockStyle; title: string; desc: string }[]
                      ).map((mode) => (
                        <button
                          key={mode.id}
                          type="button"
                          onClick={() =>
                            onChangeConfig((c) => ({ ...c, dockStyle: mode.id }))
                          }
                          className={`p-2.5 rounded-xl border text-left transition-all ${
                            config.dockStyle === mode.id
                              ? 'cosmic-accent-soft border'
                              : config.dark
                                ? 'border-white/10 bg-white/5 text-slate-200 hover:bg-white/10'
                                : 'border-slate-200 bg-white text-slate-700 hover:bg-slate-100'
                          }`}
                        >
                          <div className="text-xs font-semibold whitespace-nowrap">
                            {mode.title}
                          </div>
                          <div className="text-xs text-slate-400 mt-0.5 truncate">
                            {mode.desc}
                          </div>
                        </button>
                      ))}
                    </div>
                  </div>

                  <div className="space-y-2">
                    <span className="block text-xs font-medium text-slate-400">
                      Horizontal Position on Monitor
                    </span>
                    <div
                      className={`grid grid-cols-3 gap-1.5 p-1.5 rounded-xl ${
                        config.dark ? 'bg-slate-900' : 'bg-slate-200/70'
                      }`}
                    >
                      {(['left', 'center', 'right'] as DockPosition[]).map(
                        (pos) => (
                          <button
                            key={pos}
                            type="button"
                            onClick={() =>
                              onChangeConfig((c) => ({ ...c, position: pos }))
                            }
                            aria-label={`Dock Position ${pos}`}
                            style={
                              config.position === pos
                                ? activePillStyle
                                : undefined
                            }
                            className={`py-2 px-3 rounded-lg text-xs font-semibold capitalize transition-colors whitespace-nowrap ${
                              config.position === pos
                                ? 'shadow-sm'
                                : config.dark
                                  ? 'text-slate-300 hover:text-white'
                                  : 'text-slate-700 hover:text-slate-950'
                            }`}
                          >
                            {pos}
                          </button>
                        )
                      )}
                    </div>
                    <p className="text-xs text-slate-400">
                      Applies across primary and secondary monitors via DPI-aware
                      work area bounds.
                    </p>
                  </div>
                </div>

                {/* Behavior Toggles */}
                <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3 mt-5 pt-4 border-t border-white/10">
                  <label className="flex items-center justify-between gap-2 p-2.5 rounded-lg bg-white/5 cursor-pointer">
                    <span className="text-xs font-medium">
                      Adaptive Transparency (Desktop)
                    </span>
                    <input
                      type="checkbox"
                      checked={config.adaptiveDockTransparency}
                      onChange={(e) =>
                        onChangeConfig((c) => ({
                          ...c,
                          adaptiveDockTransparency: e.target.checked,
                        }))
                      }
                      className="w-4 h-4 rounded"
                    />
                  </label>

                  <label className="flex items-center justify-between gap-2 p-2.5 rounded-lg bg-white/5 cursor-pointer">
                    <span className="text-xs font-medium">Auto-hide Dock</span>
                    <input
                      type="checkbox"
                      checked={config.autoHide}
                      onChange={(e) =>
                        onChangeConfig((c) => ({
                          ...c,
                          autoHide: e.target.checked,
                        }))
                      }
                      className="w-4 h-4 rounded"
                    />
                  </label>

                  <label className="flex items-center justify-between gap-2 p-2.5 rounded-lg bg-white/5 cursor-pointer">
                    <span className="text-xs font-medium">Show Pinned Apps</span>
                    <input
                      type="checkbox"
                      checked={config.showPinnedApps}
                      onChange={(e) =>
                        onChangeConfig((c) => ({
                          ...c,
                          showPinnedApps: e.target.checked,
                        }))
                      }
                      className="w-4 h-4 rounded"
                    />
                  </label>

                  <label className="flex items-center justify-between gap-2 p-2.5 rounded-lg bg-white/5 cursor-pointer">
                    <span className="text-xs font-medium">
                      Active Window Indicators
                    </span>
                    <input
                      type="checkbox"
                      checked={config.showRunningIndicators}
                      onChange={(e) =>
                        onChangeConfig((c) => ({
                          ...c,
                          showRunningIndicators: e.target.checked,
                        }))
                      }
                      className="w-4 h-4 rounded"
                    />
                  </label>
                </div>
              </div>

              {/* Card 2: Icon Size, Magnification, Transparency & Spacing */}
              <div className={cardClass}>
                <div className="flex flex-wrap items-center justify-between gap-2 mb-4">
                  <h3 className="text-sm font-semibold">
                    02. Icon Size, Cosine Magnification & Surface Metrics
                  </h3>

                  <div className="flex items-center gap-1 bg-white/5 p-1 rounded-lg">
                    {[
                      { label: 'Small (36px)', size: 36, spacing: 50 },
                      { label: 'Medium (42px)', size: 42, spacing: 56 },
                      { label: 'Large (52px)', size: 52, spacing: 68 },
                    ].map((preset) => (
                      <button
                        key={preset.size}
                        type="button"
                        onClick={() =>
                          onChangeConfig((c) => ({
                            ...c,
                            iconSize: preset.size,
                            spacing: preset.spacing,
                          }))
                        }
                        style={
                          config.iconSize === preset.size
                            ? activePillStyle
                            : undefined
                        }
                        className={`px-2.5 py-1 rounded-md text-xs font-medium transition-colors whitespace-nowrap ${
                          config.iconSize === preset.size
                            ? 'font-semibold'
                            : 'text-slate-300 hover:text-white'
                        }`}
                      >
                        {preset.label}
                      </button>
                    ))}
                  </div>
                </div>

                <div className="grid grid-cols-1 md:grid-cols-2 gap-5">
                  <div className="space-y-1.5">
                    <div className="flex justify-between text-xs">
                      <label
                        htmlFor="slider-icon-size"
                        className="text-slate-200 font-medium"
                      >
                        Base Icon Size
                      </label>
                      <span
                        style={{ color: config.accentColor }}
                        className="font-mono tabular-nums"
                      >
                        {config.iconSize} px
                      </span>
                    </div>
                    <input
                      id="slider-icon-size"
                      type="range"
                      min={24}
                      max={64}
                      value={config.iconSize}
                      onChange={(e) =>
                        onChangeConfig((c) => ({
                          ...c,
                          iconSize: Number(e.target.value),
                        }))
                      }
                      className="w-full"
                    />
                  </div>

                  <div className="space-y-1.5">
                    <div className="flex justify-between text-xs">
                      <label
                        htmlFor="slider-magnification"
                        className="text-slate-200 font-medium"
                      >
                        Hover Magnification Strength
                      </label>
                      <span
                        style={{ color: config.accentColor }}
                        className="font-mono tabular-nums"
                      >
                        {config.magnification}%
                      </span>
                    </div>
                    <input
                      id="slider-magnification"
                      type="range"
                      min={0}
                      max={100}
                      value={config.magnification}
                      onChange={(e) =>
                        onChangeConfig((c) => ({
                          ...c,
                          magnification: Number(e.target.value),
                        }))
                      }
                      className="w-full"
                    />
                  </div>

                  <div className="space-y-1.5">
                    <div className="flex justify-between text-xs">
                      <label
                        htmlFor="slider-transparency"
                        className="text-slate-200 font-medium"
                      >
                        Surface Alpha (Transparency)
                      </label>
                      <span
                        style={{ color: config.accentColor }}
                        className="font-mono tabular-nums"
                      >
                        {config.transparency} / 255 (
                        {Math.round((config.transparency / 255) * 100)}%)
                      </span>
                    </div>
                    <input
                      id="slider-transparency"
                      type="range"
                      min={40}
                      max={230}
                      value={config.transparency}
                      onChange={(e) =>
                        onChangeConfig((c) => ({
                          ...c,
                          transparency: Number(e.target.value),
                        }))
                      }
                      className="w-full"
                    />
                  </div>

                  <div className="space-y-1.5">
                    <div className="flex justify-between text-xs">
                      <label
                        htmlFor="slider-spacing"
                        className="text-slate-200 font-medium"
                      >
                        Launcher Slot Spacing
                      </label>
                      <span
                        style={{ color: config.accentColor }}
                        className="font-mono tabular-nums"
                      >
                        {config.spacing} px
                      </span>
                    </div>
                    <input
                      id="slider-spacing"
                      type="range"
                      min={36}
                      max={96}
                      value={config.spacing}
                      onChange={(e) =>
                        onChangeConfig((c) => ({
                          ...c,
                          spacing: Number(e.target.value),
                        }))
                      }
                      className="w-full"
                    />
                  </div>

                  <div className="space-y-1.5">
                    <div className="flex justify-between text-xs">
                      <label
                        htmlFor="slider-radius"
                        className="text-slate-200 font-medium"
                      >
                        SDF Squircle Corner Radius
                      </label>
                      <span
                        style={{ color: config.accentColor }}
                        className="font-mono tabular-nums"
                      >
                        {config.cornerRadius} px
                      </span>
                    </div>
                    <input
                      id="slider-radius"
                      type="range"
                      min={8}
                      max={28}
                      value={config.cornerRadius}
                      onChange={(e) =>
                        onChangeConfig((c) => ({
                          ...c,
                          cornerRadius: Number(e.target.value),
                        }))
                      }
                      className="w-full"
                    />
                  </div>

                  <div className="grid grid-cols-2 gap-3">
                    <div>
                      <label
                        htmlFor="input-offset-x"
                        className="block text-xs text-slate-400 mb-1"
                      >
                        Offset X (-200..200)
                      </label>
                      <input
                        id="input-offset-x"
                        type="number"
                        min={-200}
                        max={200}
                        value={config.offsetX}
                        onChange={(e) =>
                          onChangeConfig((c) => ({
                            ...c,
                            offsetX: Number(e.target.value) || 0,
                          }))
                        }
                        className="w-full px-3 py-1.5 rounded-lg bg-black/30 border border-white/10 text-xs font-mono tabular-nums"
                      />
                    </div>
                    <div>
                      <label
                        htmlFor="input-offset-y"
                        className="block text-xs text-slate-400 mb-1"
                      >
                        Offset Y (-100..100)
                      </label>
                      <input
                        id="input-offset-y"
                        type="number"
                        min={-100}
                        max={100}
                        value={config.offsetY}
                        onChange={(e) =>
                          onChangeConfig((c) => ({
                            ...c,
                            offsetY: Number(e.target.value) || 0,
                          }))
                        }
                        className="w-full px-3 py-1.5 rounded-lg bg-black/30 border border-white/10 text-xs font-mono tabular-nums"
                      />
                    </div>
                  </div>
                </div>
              </div>

              {/* Card 3: Dock Launcher Sequence & Reordering */}
              <div className={cardClass}>
                <div className="flex flex-wrap items-center justify-between gap-2 mb-3">
                  <div>
                    <h3 className="text-sm font-semibold">
                      03. Dock Launchers Order
                    </h3>
                    <p className="text-xs text-slate-400">
                      Reorder core launchers or drag icons directly on the bottom
                      dock. Persisted as{' '}
                      <code
                        style={{ color: config.accentColor }}
                        className="font-mono"
                      >
                        dock_order={formatDockOrder(config.dockOrder)}
                      </code>
                    </p>
                  </div>
                </div>

                <div className="divide-y divide-white/10">
                  {config.dockOrder.map((launcherId, index) => {
                    const info = DOCK_LAUNCHER_NAMES[launcherId];
                    return (
                      <div
                        key={launcherId}
                        className="flex items-center justify-between py-2.5 gap-4"
                      >
                        <div className="flex items-center gap-3 min-w-0">
                          <span className="w-6 text-xs font-mono tabular-nums text-slate-400">
                            0{index + 1}
                          </span>
                          <div className="min-w-0">
                            <div className="text-xs font-semibold">
                              {info.title}
                            </div>
                            <div className="text-xs font-mono text-slate-400 truncate">
                              {launcherId} · {info.binary}
                            </div>
                          </div>
                        </div>

                        <div className="flex items-center gap-1.5 shrink-0">
                          <button
                            type="button"
                            disabled={index === 0}
                            onClick={() =>
                              onChangeConfig((c) => ({
                                ...c,
                                dockOrder: reorderDockItem(
                                  c.dockOrder,
                                  index,
                                  index - 1
                                ),
                              }))
                            }
                            aria-label={`Move ${info.title} up`}
                            className="p-1.5 rounded-lg bg-white/5 hover:bg-white/10 disabled:opacity-30 transition-colors"
                          >
                            <ArrowUp className="w-3.5 h-3.5" />
                          </button>

                          <button
                            type="button"
                            disabled={index === config.dockOrder.length - 1}
                            onClick={() =>
                              onChangeConfig((c) => ({
                                ...c,
                                dockOrder: reorderDockItem(
                                  c.dockOrder,
                                  index,
                                  index + 1
                                ),
                              }))
                            }
                            aria-label={`Move ${info.title} down`}
                            className="p-1.5 rounded-lg bg-white/5 hover:bg-white/10 disabled:opacity-30 transition-colors"
                          >
                            <ArrowDown className="w-3.5 h-3.5" />
                          </button>

                          <button
                            type="button"
                            onClick={() => onLaunchDockApp(launcherId)}
                            className="px-2.5 py-1 rounded-lg cosmic-accent-soft border text-xs font-medium transition-colors whitespace-nowrap"
                          >
                            Test Launch
                          </button>
                        </div>
                      </div>
                    );
                  })}
                </div>
              </div>
            </div>
          )}

          {/* ========================================================= */}
          {/* PAGE 2: COSMIC PANEL & APPLETS SETTINGS                   */}
          {/* ========================================================= */}
          {activePage === 'applets' && (
            <div className="space-y-6" data-testid="settings-page-applets">
              <div className="flex flex-wrap items-center justify-between gap-4">
                <div>
                  <h2 className="text-lg font-bold tracking-tight">
                    COSMIC Panel & Applets
                  </h2>
                  <p className="text-xs text-slate-400 mt-0.5">
                    Assign applets across Left (Panel Start), Center, and Right
                    (Panel End) zones and customize their display order.
                  </p>
                </div>

                <div className="flex items-center gap-1.5 bg-white/5 p-1 rounded-xl">
                  <button
                    type="button"
                    onClick={() => applyPanelPreset('cosmic')}
                    className="px-2.5 py-1 rounded-lg text-xs font-medium cosmic-accent-soft border whitespace-nowrap"
                  >
                    COSMIC Default
                  </button>
                  <button
                    type="button"
                    onClick={() => applyPanelPreset('developer')}
                    className="px-2.5 py-1 rounded-lg text-xs font-medium text-slate-300 hover:bg-white/10 whitespace-nowrap"
                  >
                    Developer Split
                  </button>
                  <button
                    type="button"
                    onClick={() => applyPanelPreset('macos')}
                    className="px-2.5 py-1 rounded-lg text-xs font-medium text-slate-300 hover:bg-white/10 whitespace-nowrap"
                  >
                    All Right-Aligned
                  </button>
                </div>
              </div>

              {/* Card 1: Top Panel Geometry & Style */}
              <div className={cardClass}>
                <h3 className="text-sm font-semibold mb-4">
                  01. Top Panel Style & Dimensions
                </h3>
                <div className="grid grid-cols-1 md:grid-cols-3 gap-5">
                  <div>
                    <span className="block text-xs text-slate-400 mb-1.5">
                      Panel Container Mode
                    </span>
                    <div className="grid grid-cols-3 gap-1.5 p-1 rounded-xl bg-black/30">
                      {(
                        [
                          { id: 'transparent', label: 'Borderless' },
                          { id: 'island', label: 'Island' },
                          { id: 'full', label: 'Full Bar' },
                        ] as { id: PanelStyle; label: string }[]
                      ).map((st) => (
                        <button
                          key={st.id}
                          type="button"
                          onClick={() =>
                            onChangeConfig((c) => ({ ...c, panelStyle: st.id }))
                          }
                          style={
                            config.panelStyle === st.id
                              ? activePillStyle
                              : undefined
                          }
                          className={`py-1.5 px-2 rounded-lg text-xs font-semibold transition-colors whitespace-nowrap ${
                            config.panelStyle === st.id
                              ? ''
                              : 'text-slate-300 hover:text-white'
                          }`}
                        >
                          {st.label}
                        </button>
                      ))}
                    </div>
                  </div>

                  <div>
                    <div className="flex justify-between text-xs mb-1.5">
                      <label
                        htmlFor="slider-panel-height"
                        className="text-slate-200"
                      >
                        Panel Height
                      </label>
                      <span
                        style={{ color: config.accentColor }}
                        className="font-mono tabular-nums"
                      >
                        {config.panelHeight} px
                      </span>
                    </div>
                    <input
                      id="slider-panel-height"
                      type="range"
                      min={32}
                      max={48}
                      value={config.panelHeight}
                      onChange={(e) =>
                        onChangeConfig((c) => ({
                          ...c,
                          panelHeight: Number(e.target.value),
                        }))
                      }
                      className="w-full"
                    />
                  </div>

                  <div>
                    <div className="flex justify-between text-xs mb-1.5">
                      <label
                        htmlFor="slider-panel-margin"
                        className="text-slate-200"
                      >
                        Island Edge Margin
                      </label>
                      <span
                        style={{ color: config.accentColor }}
                        className="font-mono tabular-nums"
                      >
                        {config.panelMargin} px
                      </span>
                    </div>
                    <input
                      id="slider-panel-margin"
                      type="range"
                      min={0}
                      max={12}
                      value={config.panelMargin}
                      onChange={(e) =>
                        onChangeConfig((c) => ({
                          ...c,
                          panelMargin: Number(e.target.value),
                        }))
                      }
                      className="w-full"
                    />
                  </div>
                </div>
              </div>

              {/* Card 2: Interactive 3-Zone Visual Schematic Preview */}
              <div className={cardClass}>
                <div className="flex flex-wrap items-center justify-between gap-2 mb-3">
                  <h3 className="text-sm font-semibold">
                    02. Live 3-Zone Panel Segment Map
                  </h3>
                  <div className="flex items-center gap-1 bg-black/30 p-1 rounded-lg">
                    {(
                      [
                        { id: 'all', label: 'All Zones (7)' },
                        { id: 'left', label: 'Left / Start' },
                        { id: 'center', label: 'Center' },
                        { id: 'right', label: 'Right / End' },
                      ] as { id: 'all' | PanelZone; label: string }[]
                    ).map((tab) => (
                      <button
                        key={tab.id}
                        type="button"
                        onClick={() => setActiveZoneTab(tab.id)}
                        style={
                          activeZoneTab === tab.id ? activePillStyle : undefined
                        }
                        className={`px-2.5 py-1 rounded-md text-xs font-medium transition-colors whitespace-nowrap ${
                          activeZoneTab === tab.id
                            ? 'font-semibold'
                            : 'text-slate-400 hover:text-slate-200'
                        }`}
                      >
                        {tab.label}
                      </button>
                    ))}
                  </div>
                </div>

                {/* 3-Column Visual Zone Summary */}
                <div className="grid grid-cols-1 md:grid-cols-3 gap-3 mb-5">
                  {(['left', 'center', 'right'] as PanelZone[]).map((z) => {
                    const zoneApplets = config.appletOrder.filter(
                      (id) =>
                        config.panelZones[id] === z && config.appletEnabled[id]
                    );
                    const zoneTitle =
                      z === 'left'
                        ? 'Panel Start (Left)'
                        : z === 'center'
                          ? 'Panel Center'
                          : 'Panel End (Right)';
                    return (
                      <div
                        key={z}
                        className="p-3 rounded-xl bg-black/25 border border-white/10 space-y-2"
                      >
                        <div className="flex items-center justify-between text-xs">
                          <span className="font-semibold text-slate-200">
                            {zoneTitle}
                          </span>
                          <span
                            style={{ color: config.accentColor }}
                            className="font-mono tabular-nums"
                          >
                            {zoneApplets.length} active
                          </span>
                        </div>
                        <div className="flex flex-wrap gap-1.5 min-h-[28px] items-center">
                          {zoneApplets.length === 0 ? (
                            <span className="text-xs text-slate-500">
                              No applets assigned
                            </span>
                          ) : (
                            zoneApplets.map((id) => (
                              <span
                                key={id}
                                className="text-xs text-slate-200 font-mono"
                              >
                                {id}
                                {zoneApplets[zoneApplets.length - 1] !== id
                                  ? ' · '
                                  : ''}
                              </span>
                            ))
                          )}
                        </div>
                      </div>
                    );
                  })}
                </div>

                {/* Full Applet List with Zone Switcher & Reorder Controls */}
                <div className="divide-y divide-white/10">
                  {config.appletOrder
                    .filter((id) =>
                      activeZoneTab === 'all'
                        ? true
                        : config.panelZones[id] === activeZoneTab
                    )
                    .map((appletId) => {
                      const meta = APPLET_METADATA[appletId];
                      const Icon = meta.icon;
                      const enabled = config.appletEnabled[appletId];
                      const currentZone = config.panelZones[appletId];
                      const sameZoneList = config.appletOrder.filter(
                        (id) => config.panelZones[id] === currentZone
                      );
                      const zoneIndex = sameZoneList.indexOf(appletId);

                      return (
                        <div
                          key={appletId}
                          className="py-3 flex flex-wrap items-center justify-between gap-4"
                        >
                          <div className="flex items-center gap-3 min-w-0">
                            <input
                              type="checkbox"
                              checked={enabled}
                              aria-label={`Enable ${meta.title}`}
                              onChange={(e) =>
                                onChangeConfig((c) => ({
                                  ...c,
                                  appletEnabled: {
                                    ...c.appletEnabled,
                                    [appletId]: e.target.checked,
                                  },
                                }))
                              }
                              className="w-4 h-4 rounded shrink-0"
                            />
                            <div
                              style={{ color: config.accentColor }}
                              className="p-2 rounded-lg bg-white/5 shrink-0"
                            >
                              <Icon className="w-4 h-4" />
                            </div>
                            <div className="min-w-0">
                              <div className="text-xs font-semibold flex items-center gap-2">
                                <span>{meta.title}</span>
                                <span className="text-xs font-mono text-slate-400">
                                  · {appletId}:{currentZone}
                                </span>
                              </div>
                              <div className="text-xs text-slate-400 truncate">
                                {meta.subtitle}
                              </div>
                            </div>
                          </div>

                          <div className="flex items-center gap-2 shrink-0">
                            <div
                              className="flex items-center gap-1 p-1 rounded-lg bg-black/30"
                              role="group"
                              aria-label={`${meta.title} Zone Assignment`}
                            >
                              {(['left', 'center', 'right'] as PanelZone[]).map(
                                (z) => (
                                  <button
                                    key={z}
                                    type="button"
                                    onClick={() =>
                                      onChangeConfig((c) =>
                                        moveAppletToZone(c, appletId, z)
                                      )
                                    }
                                    aria-label={`Assign ${meta.title} to ${z} zone`}
                                    style={
                                      currentZone === z
                                        ? activePillStyle
                                        : undefined
                                    }
                                    className={`px-2.5 py-1 rounded text-xs font-semibold capitalize transition-colors whitespace-nowrap ${
                                      currentZone === z
                                        ? ''
                                        : 'text-slate-400 hover:text-slate-200'
                                    }`}
                                  >
                                    {z}
                                  </button>
                                )
                              )}
                            </div>

                            <button
                              type="button"
                              disabled={zoneIndex <= 0}
                              onClick={() =>
                                onChangeConfig((c) =>
                                  reorderAppletInZone(c, appletId, 'up')
                                )
                              }
                              aria-label={`Move ${meta.title} earlier in ${currentZone} zone`}
                              className="p-1.5 rounded-lg bg-white/5 hover:bg-white/10 disabled:opacity-30 transition-colors"
                            >
                              <ArrowUp className="w-3.5 h-3.5" />
                            </button>
                            <button
                              type="button"
                              disabled={
                                zoneIndex === -1 ||
                                zoneIndex >= sameZoneList.length - 1
                              }
                              onClick={() =>
                                onChangeConfig((c) =>
                                  reorderAppletInZone(c, appletId, 'down')
                                )
                              }
                              aria-label={`Move ${meta.title} later in ${currentZone} zone`}
                              className="p-1.5 rounded-lg bg-white/5 hover:bg-white/10 disabled:opacity-30 transition-colors"
                            >
                              <ArrowDown className="w-3.5 h-3.5" />
                            </button>
                          </div>
                        </div>
                      );
                    })}
                </div>
              </div>

              {/* Card 3: Clock Applet Formatting & Disabled Applets Drawer */}
              <div className="grid grid-cols-1 md:grid-cols-2 gap-5">
                <div className={cardClass}>
                  <h3 className="text-sm font-semibold mb-3">
                    03. Clock & Calendar Applet Format
                  </h3>
                  <div className="space-y-3">
                    <label className="flex items-center justify-between text-xs cursor-pointer">
                      <span>Use 24-Hour Military Time</span>
                      <input
                        type="checkbox"
                        checked={config.clock24h}
                        onChange={(e) =>
                          onChangeConfig((c) => ({
                            ...c,
                            clock24h: e.target.checked,
                          }))
                        }
                        className="w-4 h-4 rounded"
                      />
                    </label>

                    <div>
                      <span className="block text-xs text-slate-400 mb-1.5">
                        Date Display Format
                      </span>
                      <div className="grid grid-cols-3 gap-1.5 p-1 rounded-xl bg-black/30">
                        {(
                          [
                            { id: 'none', label: 'Time Only' },
                            { id: 'short', label: 'ISO Short' },
                            { id: 'long', label: 'Full Weekday' },
                          ] as { id: DateFormatMode; label: string }[]
                        ).map((df) => (
                          <button
                            key={df.id}
                            type="button"
                            onClick={() =>
                              onChangeConfig((c) => ({
                                ...c,
                                dateFormat: df.id,
                              }))
                            }
                            style={
                              config.dateFormat === df.id
                                ? activePillStyle
                                : undefined
                            }
                            className={`py-1.5 px-2 rounded-lg text-xs font-semibold transition-colors whitespace-nowrap ${
                              config.dateFormat === df.id
                                ? ''
                                : 'text-slate-300 hover:text-white'
                            }`}
                          >
                            {df.label}
                          </button>
                        ))}
                      </div>
                    </div>
                  </div>
                </div>

                <div className={cardClass}>
                  <h3 className="text-sm font-semibold mb-3">
                    04. Add / Restore Hidden Applets
                  </h3>
                  {ALL_APPLETS.filter((id) => !config.appletEnabled[id]).length ===
                  0 ? (
                    <div className="text-xs text-slate-400 py-3">
                      All 7 Luma applets are currently active on the top panel.
                      Uncheck any applet above to hide it to this drawer.
                    </div>
                  ) : (
                    <div className="flex flex-wrap gap-2">
                      {ALL_APPLETS.filter(
                        (id) => !config.appletEnabled[id]
                      ).map((id) => (
                        <button
                          key={id}
                          type="button"
                          onClick={() =>
                            onChangeConfig((c) => ({
                              ...c,
                              appletEnabled: { ...c.appletEnabled, [id]: true },
                            }))
                          }
                          className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg cosmic-accent-soft border text-xs font-medium"
                        >
                          <Plus className="w-3.5 h-3.5" />
                          <span>Add {APPLET_METADATA[id].title}</span>
                        </button>
                      ))}
                    </div>
                  )}
                </div>
              </div>
            </div>
          )}

          {/* ========================================================= */}
          {/* PAGE 3: GLOBAL ACCENT, WALLPAPER & TYPOGRAPHY             */}
          {/* ========================================================= */}
          {activePage === 'appearance' && (
            <div className="space-y-6" data-testid="settings-page-appearance">
              <div>
                <h2 className="text-lg font-bold tracking-tight">
                  Global Accent Color, Wallpaper Studio & Typography
                </h2>
                <p className="text-xs text-slate-400 mt-0.5">
                  Personalize global system accent color, desktop wallpaper
                  engine, crisp vector font family, and Winlogon shell mode.
                </p>
              </div>

              {/* Card 1: Global Theme & Global Accent Color */}
              <div className={cardClass}>
                <h3 className="text-sm font-semibold mb-4">
                  01. Global System Theme & Accent Color
                </h3>

                <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
                  {/* Dark vs Daylight Polar */}
                  <div className="lg:col-span-4 space-y-2">
                    <span className="block text-xs font-medium text-slate-400">
                      Surface Luminance Mode
                    </span>
                    <div className="grid grid-cols-2 gap-2.5">
                      <button
                        type="button"
                        onClick={() =>
                          onChangeConfig((c) => ({
                            ...c,
                            dark: true,
                            highContrast: false,
                          }))
                        }
                        className={`p-3 rounded-xl border text-left transition-all ${
                          config.dark && !config.highContrast
                            ? 'cosmic-accent-soft border'
                            : 'border-white/10 bg-white/5'
                        }`}
                      >
                        <div className="text-xs font-semibold">COSMIC Dark</div>
                        <div className="text-xs text-slate-400 mt-0.5">
                          Obsidian slate
                        </div>
                      </button>

                      <button
                        type="button"
                        onClick={() =>
                          onChangeConfig((c) => ({
                            ...c,
                            dark: false,
                            highContrast: false,
                          }))
                        }
                        className={`p-3 rounded-xl border text-left transition-all ${
                          !config.dark && !config.highContrast
                            ? 'cosmic-accent-soft border'
                            : 'border-white/10 bg-white/5'
                        }`}
                      >
                        <div className="text-xs font-semibold">
                          Daylight Polar
                        </div>
                        <div className="text-xs text-slate-400 mt-0.5">
                          Frost daylight
                        </div>
                      </button>
                    </div>
                  </div>

                  {/* Global Accent Swatches + Custom Hex Picker */}
                  <div className="lg:col-span-8 space-y-2">
                    <div className="flex flex-wrap items-center justify-between gap-2">
                      <span className="text-xs font-medium text-slate-400">
                        Global Accent Color (Applies to Panel, Dock, Sliders,
                        Popups & Wallpaper)
                      </span>
                      <label className="flex items-center gap-2 text-xs cursor-pointer">
                        <span className="text-slate-400 font-mono">
                          Custom Hex:
                        </span>
                        <input
                          type="color"
                          aria-label="Custom Global Accent Hex Color"
                          value={config.accentColor}
                          onChange={(e) =>
                            onChangeConfig((c) => ({
                              ...c,
                              accentColor: e.target.value,
                            }))
                          }
                          className="w-6 h-6 rounded cursor-pointer border-0 bg-transparent"
                        />
                        <span
                          style={{ color: config.accentColor }}
                          className="font-mono font-semibold"
                        >
                          {config.accentColor.toUpperCase()}
                        </span>
                      </label>
                    </div>

                    <div className="grid grid-cols-2 sm:grid-cols-4 gap-2">
                      {COSMIC_ACCENTS.map((acc) => {
                        const isSelected =
                          config.accentColor.toLowerCase() ===
                          acc.hex.toLowerCase();
                        return (
                          <button
                            key={acc.id}
                            type="button"
                            onClick={() =>
                              onChangeConfig((c) => ({
                                ...c,
                                accentColor: acc.hex,
                              }))
                            }
                            aria-label={`Set global accent to ${acc.name}`}
                            className={`flex items-center gap-2 px-2.5 py-2 rounded-xl border text-xs font-semibold whitespace-nowrap transition-all ${
                              isSelected
                                ? 'border-white bg-white/15 shadow-sm'
                                : 'border-white/10 bg-white/5 hover:bg-white/10'
                            }`}
                          >
                            <span
                              style={{ backgroundColor: acc.hex }}
                              className="w-3 h-3 rounded-full shrink-0"
                            />
                            <span>{acc.name}</span>
                          </button>
                        );
                      })}
                    </div>
                  </div>
                </div>

                <div className="mt-4 pt-4 border-t border-white/10 flex flex-wrap items-center justify-between gap-4">
                  <label className="flex items-center gap-2 text-xs cursor-pointer">
                    <input
                      type="checkbox"
                      checked={config.syncAccentToWallpaper}
                      onChange={(e) =>
                        onChangeConfig((c) => ({
                          ...c,
                          syncAccentToWallpaper: e.target.checked,
                        }))
                      }
                      className="w-4 h-4 rounded"
                    />
                    <span>
                      Harmonize Desktop Wallpaper & GPU Shaders with Global
                      Accent Color
                    </span>
                  </label>

                  <label className="flex items-center gap-2 text-xs cursor-pointer">
                    <input
                      type="checkbox"
                      checked={config.highContrast}
                      onChange={(e) =>
                        onChangeConfig((c) => ({
                          ...c,
                          highContrast: e.target.checked,
                        }))
                      }
                      className="w-4 h-4 rounded"
                    />
                    <span>Windows High-Contrast Accessibility Palette</span>
                  </label>
                </div>
              </div>

              {/* Card 2: Global Desktop Wallpaper Studio */}
              <div className={cardClass}>
                <div className="flex flex-wrap items-center justify-between gap-2 mb-4">
                  <div className="flex items-center gap-2">
                    <ImageIcon
                      style={{ color: config.accentColor }}
                      className="w-4 h-4"
                    />
                    <h3 className="text-sm font-semibold">
                      02. Global Desktop Wallpaper & Background Studio
                    </h3>
                  </div>

                  <label className="flex items-center gap-2 text-xs cursor-pointer">
                    <input
                      type="checkbox"
                      checked={config.desktopEnabled}
                      onChange={(e) =>
                        onChangeConfig((c) => ({
                          ...c,
                          desktopEnabled: e.target.checked,
                        }))
                      }
                      className="w-4 h-4 rounded"
                    />
                    <span>Enable Luma Desktop Surface</span>
                  </label>
                </div>

                {/* 4 Wallpaper Modes */}
                <div className="grid grid-cols-2 sm:grid-cols-4 gap-2.5 mb-5">
                  {(
                    [
                      {
                        id: 'gpu_shader',
                        title: 'GPU Shader Live',
                        desc: 'WebGL2 / WGPU procedural nebula',
                      },
                      {
                        id: 'vector_scene',
                        title: 'COSMIC Vector Art',
                        desc: 'Crisp high-DPI SVG artwork',
                      },
                      {
                        id: 'gradient',
                        title: 'Ambient Gradient',
                        desc: 'Architectural accent mesh',
                      },
                      {
                        id: 'custom_image',
                        title: 'Custom Wallpaper',
                        desc: 'Upload image or paste URL',
                      },
                    ] as { id: WallpaperMode; title: string; desc: string }[]
                  ).map((wm) => (
                    <button
                      key={wm.id}
                      type="button"
                      onClick={() =>
                        onChangeConfig((c) => ({ ...c, wallpaperMode: wm.id }))
                      }
                      className={`p-3 rounded-xl border text-left transition-all ${
                        config.wallpaperMode === wm.id
                          ? 'cosmic-accent-soft border'
                          : 'border-white/10 bg-white/5 hover:bg-white/10'
                      }`}
                    >
                      <div className="text-xs font-semibold">{wm.title}</div>
                      <div className="text-xs text-slate-400 mt-0.5">
                        {wm.desc}
                      </div>
                    </button>
                  ))}
                </div>

                {/* Sub-options based on selected wallpaper mode */}
                {config.wallpaperMode === 'gpu_shader' && (
                  <div className="grid grid-cols-2 sm:grid-cols-5 gap-2 mb-4">
                    {(
                      [
                        { id: 'orbital_earth', label: 'Orbital Earth Horizon' },
                        { id: 'cosmic_nebula', label: 'COSMIC Nebula' },
                        { id: 'system76_obsidian', label: 'System76 Obsidian' },
                        { id: 'solar_flare', label: 'Solar Flare' },
                        { id: 'teal_aurora', label: 'Teal Aurora' },
                      ] as { id: GpuShaderPreset; label: string }[]
                    ).map((sp) => (
                      <button
                        key={sp.id}
                        type="button"
                        onClick={() =>
                          onChangeConfig((c) => ({ ...c, gpuShader: sp.id }))
                        }
                        style={
                          config.gpuShader === sp.id
                            ? activePillStyle
                            : undefined
                        }
                        className={`py-2 px-3 rounded-lg text-xs font-semibold transition-colors ${
                          config.gpuShader === sp.id
                            ? ''
                            : 'bg-white/5 text-slate-300 hover:bg-white/10'
                        }`}
                      >
                        {sp.label}
                      </button>
                    ))}
                  </div>
                )}

                {config.wallpaperMode === 'vector_scene' && (
                  <div className="grid grid-cols-2 sm:grid-cols-4 gap-2 mb-4">
                    {(
                      [
                        { id: 'orbit_topography', label: 'System76 Orbit' },
                        { id: 'nordic_ridges', label: 'Nordic Mountain Ridges' },
                        { id: 'isometric_grid', label: 'Cybernetic Grid' },
                        { id: 'sunset_dunes', label: 'Solar Horizon Dunes' },
                      ] as { id: VectorWallpaperPreset; label: string }[]
                    ).map((vp) => (
                      <button
                        key={vp.id}
                        type="button"
                        onClick={() =>
                          onChangeConfig((c) => ({
                            ...c,
                            wallpaperVectorPreset: vp.id,
                          }))
                        }
                        style={
                          config.wallpaperVectorPreset === vp.id
                            ? activePillStyle
                            : undefined
                        }
                        className={`py-2 px-3 rounded-lg text-xs font-semibold transition-colors ${
                          config.wallpaperVectorPreset === vp.id
                            ? ''
                            : 'bg-white/5 text-slate-300 hover:bg-white/10'
                        }`}
                      >
                        {vp.label}
                      </button>
                    ))}
                  </div>
                )}

                {config.wallpaperMode === 'custom_image' && (
                  <div className="flex flex-wrap items-center gap-3 mb-4">
                    <input
                      type="text"
                      value={config.customWallpaperUrl}
                      onChange={(e) =>
                        onChangeConfig((c) => ({
                          ...c,
                          customWallpaperUrl: e.target.value,
                        }))
                      }
                      placeholder="Paste custom wallpaper image URL or upload local image..."
                      aria-label="Custom Wallpaper URL"
                      className="flex-1 min-w-[240px] px-3 py-2 rounded-xl bg-black/30 border border-white/10 text-xs"
                    />
                    <label className="flex items-center gap-1.5 px-3 py-2 rounded-xl cosmic-accent-soft border text-xs font-semibold cursor-pointer">
                      <Upload className="w-3.5 h-3.5" />
                      <span>Upload Wallpaper Image</span>
                      <input
                        type="file"
                        accept="image/*"
                        onChange={handleWallpaperUpload}
                        className="hidden"
                      />
                    </label>
                  </div>
                )}

                {/* Wallpaper Contrast Dimming Slider */}
                <div className="space-y-1.5">
                  <div className="flex justify-between text-xs">
                    <label
                      htmlFor="slider-wallpaper-dimming"
                      className="text-slate-200 font-medium"
                    >
                      Wallpaper Contrast Scrim / Dimming
                    </label>
                    <span
                      style={{ color: config.accentColor }}
                      className="font-mono tabular-nums"
                    >
                      {config.wallpaperDimming}%
                    </span>
                  </div>
                  <input
                    id="slider-wallpaper-dimming"
                    type="range"
                    min={0}
                    max={70}
                    value={config.wallpaperDimming}
                    onChange={(e) =>
                      onChangeConfig((c) => ({
                        ...c,
                        wallpaperDimming: Number(e.target.value),
                      }))
                    }
                    className="w-full"
                  />
                </div>
              </div>

              {/* Card 3: Global Crisp Typography & ClearType Font Scaling */}
              <div className={cardClass}>
                <div className="flex items-center gap-2 mb-4">
                  <Type
                    style={{ color: config.accentColor }}
                    className="w-4 h-4"
                  />
                  <h3 className="text-sm font-semibold">
                    03. Crisp High-DPI Typography & Global Font Scale
                  </h3>
                </div>

                <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                  <div>
                    <span className="block text-xs text-slate-400 mb-2">
                      Global Shell Font Family (ClearType Natural Anti-Aliasing)
                    </span>
                    <div className="grid grid-cols-2 gap-2">
                      {(
                        [
                          {
                            id: 'jakarta',
                            label: 'Plus Jakarta Sans',
                            desc: 'COSMIC geometric display',
                          },
                          {
                            id: 'dm_sans',
                            label: 'DM Sans',
                            desc: 'High-legibility UI prose',
                          },
                          {
                            id: 'system_segoe',
                            label: 'Segoe UI Variable',
                            desc: 'Native Windows 11 shell',
                          },
                          {
                            id: 'jetbrains_mono',
                            label: 'JetBrains Mono',
                            desc: 'Developer technical mono',
                          },
                        ] as {
                          id: FontFamilyPreset;
                          label: string;
                          desc: string;
                        }[]
                      ).map((ff) => (
                        <button
                          key={ff.id}
                          type="button"
                          onClick={() =>
                            onChangeConfig((c) => ({ ...c, fontFamily: ff.id }))
                          }
                          className={`p-2.5 rounded-xl border text-left transition-all ${
                            config.fontFamily === ff.id
                              ? 'cosmic-accent-soft border'
                              : 'border-white/10 bg-white/5 hover:bg-white/10'
                          }`}
                        >
                          <div className="text-xs font-semibold">
                            {ff.label}
                          </div>
                          <div className="text-xs text-slate-400">
                            {ff.desc}
                          </div>
                        </button>
                      ))}
                    </div>
                  </div>

                  <div className="space-y-3">
                    <div className="space-y-1.5">
                      <div className="flex justify-between text-xs">
                        <label
                          htmlFor="slider-font-scale"
                          className="text-slate-200 font-medium"
                        >
                          Global Text & UI DPI Scale
                        </label>
                        <span
                          style={{ color: config.accentColor }}
                          className="font-mono tabular-nums"
                        >
                          {config.fontScale}%
                        </span>
                      </div>
                      <input
                        id="slider-font-scale"
                        type="range"
                        min={90}
                        max={115}
                        value={config.fontScale}
                        onChange={(e) =>
                          onChangeConfig((c) => ({
                            ...c,
                            fontScale: Number(e.target.value),
                          }))
                        }
                        className="w-full"
                      />
                    </div>
                    <p className="text-xs text-slate-400 leading-relaxed">
                      Native Win32 controls now receive{' '}
                      <code className="font-mono">WM_SETFONT</code> with{' '}
                      <code className="font-mono">
                        CLEARTYPE_NATURAL_QUALITY
                      </code>{' '}
                      (<code className="font-mono">Segoe UI Variable Text</code>
                      ), eliminating bitmap font pixelation.
                    </p>
                  </div>
                </div>
              </div>

              {/* Card 4: Windows Shell Replacement */}
              <div className={cardClass}>
                <h3 className="text-sm font-semibold mb-3">
                  04. Windows Shell Replacement & Watchdog Recovery
                </h3>
                <div className="flex flex-wrap items-center justify-between gap-4">
                  <div className="space-y-1">
                    <div className="text-xs font-semibold flex items-center gap-2">
                      <Monitor
                        style={{ color: config.accentColor }}
                        className="w-4 h-4"
                      />
                      <span>
                        {config.lumaShellActive
                          ? 'Luma Shell is registered as default Winlogon Shell'
                          : 'Windows Explorer (explorer.exe) is active default shell'}
                      </span>
                    </div>
                    <p className="text-xs text-slate-400">
                      Manages{' '}
                      <code className="font-mono">
                        HKCU\Software\Microsoft\Windows NT\CurrentVersion\Winlogon\Shell
                      </code>{' '}
                      with automatic watchdog rollback protection.
                    </p>
                  </div>

                  <button
                    type="button"
                    onClick={() =>
                      onChangeConfig((c) => ({
                        ...c,
                        lumaShellActive: !c.lumaShellActive,
                      }))
                    }
                    style={
                      !config.lumaShellActive ? activePillStyle : undefined
                    }
                    className={`px-3.5 py-2 rounded-xl text-xs font-semibold transition-colors whitespace-nowrap ${
                      config.lumaShellActive
                        ? 'bg-amber-500/20 text-amber-300 border border-amber-400/30 hover:bg-amber-500/30'
                        : ''
                    }`}
                  >
                    {config.lumaShellActive
                      ? 'Restore Windows Explorer'
                      : 'Set Luma as Default Shell'}
                  </button>
                </div>
              </div>
            </div>
          )}

          {/* ========================================================= */}
          {/* PAGE 4: GPU RENDERING & COMPOSITOR                        */}
          {/* ========================================================= */}
          {activePage === 'gpu' && (
            <div className="space-y-6" data-testid="settings-page-gpu">
              <div>
                <h2 className="text-lg font-bold tracking-tight">
                  GPU Rendering & WGPU / SDF Compositor
                </h2>
                <p className="text-xs text-slate-400 mt-0.5">
                  Configure the WebGL2 / WGPU shader background, Signed Distance
                  Field (SDF) dock surface rasterizer, and glass blur uniforms.
                </p>
              </div>

              <div className={cardClass}>
                <h3 className="text-sm font-semibold mb-4">
                  01. Hardware Rendering Backend
                </h3>
                <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
                  {(
                    [
                      {
                        id: 'wgpu',
                        title: 'WGPU + Iced Pipeline',
                        desc: 'Vulkan / DX12 shader surfaces (luma_iced)',
                      },
                      {
                        id: 'direct2d',
                        title: 'Direct2D + SDF ARGB',
                        desc: 'Sub-pixel SDF squircle & DWM Mica composition',
                      },
                      {
                        id: 'software',
                        title: 'GDI+ DIB Fallback',
                        desc: 'Low-overhead CPU compatible rasterizer',
                      },
                    ] as { id: GpuBackendMode; title: string; desc: string }[]
                  ).map((b) => (
                    <button
                      key={b.id}
                      type="button"
                      onClick={() =>
                        onChangeConfig((c) => ({ ...c, gpuBackend: b.id }))
                      }
                      className={`p-3.5 rounded-xl border text-left transition-all ${
                        config.gpuBackend === b.id
                          ? 'cosmic-accent-soft border'
                          : 'border-white/10 bg-white/5 text-slate-200 hover:bg-white/10'
                      }`}
                    >
                      <div className="text-xs font-semibold">{b.title}</div>
                      <div className="text-xs text-slate-400 mt-1">
                        {b.desc}
                      </div>
                    </button>
                  ))}
                </div>
              </div>

              <div className={cardClass}>
                <div className="flex flex-wrap items-center justify-between gap-2 mb-4">
                  <h3 className="text-sm font-semibold">
                    02. Desktop GPU Fragment Shader & Glass Uniforms
                  </h3>
                  <button
                    type="button"
                    onClick={onToggleAnimateShader}
                    style={animateShader ? activePillStyle : undefined}
                    className={`px-3 py-1 rounded-lg text-xs font-medium transition-colors whitespace-nowrap ${
                      animateShader
                        ? 'font-semibold'
                        : 'bg-white/10 text-slate-200 hover:bg-white/15'
                    }`}
                  >
                    {animateShader
                      ? 'Live Shader Loop: Active'
                      : 'Animate Shader Uniforms'}
                  </button>
                </div>

                <div className="grid grid-cols-2 sm:grid-cols-5 gap-2.5 mb-5">
                  {(
                    [
                      { id: 'orbital_earth', label: 'Orbital Earth Horizon' },
                      { id: 'cosmic_nebula', label: 'COSMIC Nebula' },
                      { id: 'system76_obsidian', label: 'System76 Obsidian' },
                      { id: 'solar_flare', label: 'Solar Flare' },
                      { id: 'teal_aurora', label: 'Teal Aurora' },
                    ] as { id: GpuShaderPreset; label: string }[]
                  ).map((preset) => (
                    <button
                      key={preset.id}
                      type="button"
                      onClick={() =>
                        onChangeConfig((c) => ({
                          ...c,
                          wallpaperMode: 'gpu_shader',
                          gpuShader: preset.id,
                        }))
                      }
                      className={`py-2.5 px-3 rounded-xl border text-xs font-semibold transition-all ${
                        config.gpuShader === preset.id
                          ? 'cosmic-accent-soft border'
                          : 'border-white/10 bg-white/5 text-slate-200 hover:bg-white/10'
                      }`}
                    >
                      {preset.label}
                    </button>
                  ))}
                </div>

                <div className="grid grid-cols-1 md:grid-cols-2 gap-5">
                  <div className="space-y-1.5">
                    <div className="flex justify-between text-xs">
                      <label
                        htmlFor="slider-gpu-blur"
                        className="text-slate-200"
                      >
                        Glass Backdrop Blur Radius
                      </label>
                      <span
                        style={{ color: config.accentColor }}
                        className="font-mono tabular-nums"
                      >
                        {config.gpuBlurRadius} px
                      </span>
                    </div>
                    <input
                      id="slider-gpu-blur"
                      type="range"
                      min={0}
                      max={40}
                      value={config.gpuBlurRadius}
                      onChange={(e) =>
                        onChangeConfig((c) => ({
                          ...c,
                          gpuBlurRadius: Number(e.target.value),
                        }))
                      }
                      className="w-full"
                    />
                  </div>

                  <div className="space-y-1.5">
                    <div className="flex justify-between text-xs">
                      <label
                        htmlFor="slider-gpu-bloom"
                        className="text-slate-200"
                      >
                        Shader Nebula Bloom Intensity
                      </label>
                      <span
                        style={{ color: config.accentColor }}
                        className="font-mono tabular-nums"
                      >
                        {config.gpuBloomStrength}%
                      </span>
                    </div>
                    <input
                      id="slider-gpu-bloom"
                      type="range"
                      min={0}
                      max={100}
                      value={config.gpuBloomStrength}
                      onChange={(e) =>
                        onChangeConfig((c) => ({
                          ...c,
                          gpuBloomStrength: Number(e.target.value),
                        }))
                      }
                      className="w-full"
                    />
                  </div>
                </div>

                <div className="mt-4 pt-4 border-t border-white/10 flex flex-wrap gap-4">
                  <label className="flex items-center gap-2 text-xs cursor-pointer">
                    <input
                      type="checkbox"
                      checked={config.gpuSpecularBorder}
                      onChange={(e) =>
                        onChangeConfig((c) => ({
                          ...c,
                          gpuSpecularBorder: e.target.checked,
                        }))
                      }
                      className="w-4 h-4 rounded"
                    />
                    <span>Top-Edge SDF Specular Rim Lighting</span>
                  </label>
                </div>
              </div>
            </div>
          )}

          {/* ========================================================= */}
          {/* PAGE 5: SETTINGS.INI & RUST GPU SOURCE SYNC               */}
          {/* ========================================================= */}
          {activePage === 'config_sync' && (
            <div className="space-y-6" data-testid="settings-page-config">
              <div className="flex flex-wrap items-center justify-between gap-4">
                <div>
                  <h2 className="text-lg font-bold tracking-tight">
                    %APPDATA%\LumaShell\settings.ini & Rust WGPU Modules
                  </h2>
                  <p className="text-xs text-slate-400 mt-0.5">
                    Live serialized configuration and upgraded Rust source files (
                    <code className="font-mono">src/shell/gpu.rs</code> and{' '}
                    <code className="font-mono">src/bin/luma_iced.rs</code>).
                  </p>
                </div>

                <div className="flex items-center gap-2">
                  <button
                    type="button"
                    onClick={handleCopyIni}
                    className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-white/10 hover:bg-white/15 text-xs font-medium whitespace-nowrap"
                  >
                    {copiedIni ? (
                      <Check className="w-3.5 h-3.5 text-emerald-400" />
                    ) : (
                      <Copy className="w-3.5 h-3.5" />
                    )}
                    <span>{copiedIni ? 'Copied INI' : 'Copy settings.ini'}</span>
                  </button>

                  <button
                    type="button"
                    onClick={handleDownloadIni}
                    style={activePillStyle}
                    className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold whitespace-nowrap"
                  >
                    <Download className="w-3.5 h-3.5" />
                    <span>Download settings.ini</span>
                  </button>
                </div>
              </div>

              <div className="grid grid-cols-1 lg:grid-cols-2 gap-5">
                <div className={cardClass}>
                  <div className="flex items-center justify-between mb-2">
                    <h3 className="text-xs font-semibold">
                      Live Generated settings.ini
                    </h3>
                    <span className="text-xs font-mono text-slate-400">
                      order={formatAppletOrder(config.appletOrder).slice(0, 22)}
                      ...
                    </span>
                  </div>
                  <pre
                    style={{ color: config.accentColor }}
                    className="p-3 rounded-lg bg-black/50 border border-white/10 text-xs font-mono overflow-x-auto leading-relaxed"
                  >
                    {currentIniText}
                  </pre>
                </div>

                <div className={cardClass}>
                  <div className="flex items-center justify-between mb-2">
                    <label
                      htmlFor="ini-import-textarea"
                      className="text-xs font-semibold"
                    >
                      Import / Parse Custom settings.ini
                    </label>
                    <button
                      type="button"
                      onClick={() => {
                        if (!iniDraft.trim()) {
                          setIniFeedback(
                            'Please enter valid settings.ini key=value lines.'
                          );
                          return;
                        }
                        const parsed = parseSettingsIni(iniDraft);
                        onChangeConfig(() => parsed);
                        setIniFeedback(
                          'Parsed and applied settings.ini to shell.'
                        );
                      }}
                      className="px-2.5 py-1 rounded-md cosmic-accent-soft border text-xs font-semibold"
                    >
                      Apply settings.ini
                    </button>
                  </div>
                  <textarea
                    id="ini-import-textarea"
                    rows={10}
                    value={iniDraft}
                    onChange={(e) => {
                      setIniDraft(e.target.value);
                      setIniFeedback(null);
                    }}
                    className="w-full p-3 rounded-lg bg-black/40 border border-white/10 text-xs font-mono text-slate-200 focus:outline-none"
                  />
                  {iniFeedback && (
                    <p className="text-xs text-emerald-400 mt-1.5">
                      {iniFeedback}
                    </p>
                  )}
                </div>
              </div>

              <div className={cardClass}>
                <h3 className="text-xs font-semibold mb-2">
                  Upgraded Native Rust GPU Module (src/shell/gpu.rs &
                  src/bin/luma_iced.rs)
                </h3>
                <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
                  <pre className="p-3 rounded-lg bg-black/50 border border-white/10 text-xs font-mono text-slate-300 overflow-x-auto max-h-52">
                    {RUST_GPU_RS_SOURCE}
                  </pre>
                  <pre className="p-3 rounded-lg bg-black/50 border border-white/10 text-xs font-mono text-slate-300 overflow-x-auto max-h-52">
                    {RUST_ICED_WGPU_SOURCE}
                  </pre>
                </div>
              </div>
            </div>
          )}
        </main>
      </div>
    </section>
  );
};
