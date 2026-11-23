import React, { useCallback, useEffect, useMemo, useState } from 'react';
import {
  Cpu,
  FolderOpen,
  LayoutGrid,
  Palette,
  PanelTop,
  Sliders,
  Sparkles,
  Terminal,
} from 'lucide-react';
import {
  defaultLumaConfig,
  DockLauncherId,
  FontFamilyPreset,
  hexToRgbTuple,
  LumaConfig,
} from './utils/lumaConfig';
import {
  collectWindowIds,
  computeTiledLayout,
  createThreePanePresetTree,
  insertWindowIntoTree,
  ManagedAppKind,
  ManagedWindow,
  removeWindowFromTree,
  resizeWindowInTree,
  staggeredFloatingRect,
  swapWindowWithNeighbor,
  TilingNode,
  toggleSplitDirectionAt,
  WindowRect,
} from './utils/tilingEngine';
import { GpuCompositorCanvas } from './components/GpuCompositorCanvas';
import {
  ActivePopup,
  CosmicTopPanel,
  SystemTrayItem,
} from './components/CosmicTopPanel';
import { CosmicDock, DockAppItem } from './components/CosmicDock';
import {
  CosmicSettingsWindow,
  SettingsPageId,
} from './components/CosmicSettingsWindow';
import {
  AudioSessionRow,
  CosmicPopups,
  LauncherAppEntry,
} from './components/CosmicPopups';
import { CosmicWindowLayer } from './components/CosmicWindowLayer';

const STORAGE_KEY = 'luma_cosmic_orbital_config_v14';

const APP_WINDOW_META: Record<
  ManagedAppKind,
  { title: string; subtitle: string }
> = {
  vscode: {
    title: 'Visual Studio Code',
    subtitle: 'rust_shell — src/shell/tiling.rs',
  },
  terminal: {
    title: 'COSMIC Terminal',
    subtitle: 'PowerShell · cargo build --release',
  },
  files: {
    title: 'Luma Files',
    subtitle: '%APPDATA%\\LumaShell',
  },
  brave: {
    title: 'Brave Browser',
    subtitle: 'GlazeWM × COSMIC Spec',
  },
  notes: {
    title: 'Workspace Notes',
    subtitle: 'COSMIC Scratchpad',
  },
};

function resolveFontStack(preset: FontFamilyPreset): string {
  switch (preset) {
    case 'jakarta':
      return "'Plus Jakarta Sans', 'Segoe UI Variable Text', system-ui, sans-serif";
    case 'dm_sans':
      return "'DM Sans', 'Segoe UI Variable Text', system-ui, sans-serif";
    case 'system_segoe':
      return "'Segoe UI Variable Text', 'Segoe UI', system-ui, sans-serif";
    case 'jetbrains_mono':
      return "'JetBrains Mono', monospace";
  }
}

export function App() {
  const [config, setConfig] = useState<LumaConfig>(() => {
    try {
      const saved = localStorage.getItem(STORAGE_KEY);
      if (saved) {
        return { ...defaultLumaConfig(), ...JSON.parse(saved) };
      }
    } catch {
      // ignore storage errors
    }
    return defaultLumaConfig();
  });

  // Start with unobstructed COSMIC Orbital Desktop matching the reference screenshot
  const [settingsOpen, setSettingsOpen] = useState<boolean>(false);
  const [settingsPage, setSettingsPage] = useState<SettingsPageId>('dock');
  const [activeWorkspace, setActiveWorkspace] = useState<number>(4);
  const [activePopup, setActivePopup] = useState<ActivePopup>(null);
  const [animateShader, setAnimateShader] = useState<boolean>(false);
  const [focusedAppId, setFocusedAppId] = useState<string>('vscode');
  const [contextMenuPos, setContextMenuPos] = useState<{
    x: number;
    y: number;
  } | null>(null);

  // GlazeWM SplitContainer + COSMIC Tiling/Floating Window Manager State
  const [viewport, setViewport] = useState<{ w: number; h: number }>(() => ({
    w: typeof window !== 'undefined' ? window.innerWidth : 1440,
    h: typeof window !== 'undefined' ? window.innerHeight : 900,
  }));
  const [managedWindows, setManagedWindows] = useState<ManagedWindow[]>([]);
  const [treesByWorkspace, setTreesByWorkspace] = useState<
    Record<number, TilingNode | null>
  >({});
  const [focusedWindowId, setFocusedWindowId] = useState<string | null>(null);
  const [topZIndex, setTopZIndex] = useState<number>(10);

  useEffect(() => {
    const onResize = () => {
      setViewport({ w: window.innerWidth, h: window.innerHeight });
    };
    window.addEventListener('resize', onResize);
    return () => window.removeEventListener('resize', onResize);
  }, []);

  const workArea: WindowRect = useMemo(() => {
    const topMargin = config.panelHeight + (config.panelStyle === 'island' ? config.panelMargin : 0);
    const bottomDockClearance = config.autoHide ? 16 : 84;
    return {
      x: 0,
      y: topMargin,
      width: Math.max(480, viewport.w),
      height: Math.max(320, viewport.h - topMargin - bottomDockClearance),
    };
  }, [viewport.w, viewport.h, config.panelHeight, config.panelStyle, config.panelMargin, config.autoHide]);

  const workspaceWindows = useMemo(
    () => managedWindows.filter((w) => w.workspace === activeWorkspace),
    [managedWindows, activeWorkspace]
  );

  const activeWorkspaceTree = treesByWorkspace[activeWorkspace] ?? null;

  const tiledRects = useMemo(
    () =>
      computeTiledLayout(activeWorkspaceTree, workArea, {
        innerGap: config.tilingInnerGap,
        outerGap: config.tilingOuterGap,
      }),
    [activeWorkspaceTree, workArea, config.tilingInnerGap, config.tilingOuterGap]
  );

  const focusWindow = useCallback((id: string) => {
    setFocusedWindowId(id);
    setTopZIndex((z) => {
      const nextZ = z + 1;
      setManagedWindows((prev) =>
        prev.map((w) => (w.id === id ? { ...w, zIndex: nextZ } : w))
      );
      return nextZ;
    });
  }, []);

  const openManagedAppWindow = useCallback(
    (appKind: ManagedAppKind) => {
      const existing = managedWindows.find(
        (w) => w.workspace === activeWorkspace && w.appKind === appKind
      );
      if (existing) {
        focusWindow(existing.id);
        setFocusedAppId(appKind);
        return;
      }
      const id = `win-${appKind}-${activeWorkspace}-${Date.now()}`;
      const meta = APP_WINDOW_META[appKind];
      const nextZ = topZIndex + 1;
      const mode = config.tilingEnabled ? 'tiling' : 'floating';
      const floatingRect = staggeredFloatingRect(
        workArea,
        workspaceWindows.length
      );
      const newWin: ManagedWindow = {
        id,
        appKind,
        title: meta.title,
        subtitle: meta.subtitle,
        workspace: activeWorkspace,
        mode,
        floatingRect,
        zIndex: nextZ,
      };
      setTopZIndex(nextZ);
      setManagedWindows((prev) => [...prev, newWin]);
      setFocusedWindowId(id);
      setFocusedAppId(appKind);
      if (mode === 'tiling') {
        setTreesByWorkspace((prev) => ({
          ...prev,
          [activeWorkspace]: insertWindowIntoTree(
            prev[activeWorkspace] ?? null,
            id,
            focusedWindowId,
            config.tilingDirection
          ),
        }));
      }
    },
    [
      managedWindows,
      activeWorkspace,
      focusWindow,
      topZIndex,
      config.tilingEnabled,
      config.tilingDirection,
      workArea,
      workspaceWindows.length,
      focusedWindowId,
    ]
  );

  const closeManagedWindow = useCallback((id: string) => {
    const closing = managedWindows.find((w) => w.id === id);
    if (!closing) return;
    setManagedWindows((prev) => prev.filter((w) => w.id !== id));
    setTreesByWorkspace((prev) => {
      const nextTree = removeWindowFromTree(prev[closing.workspace] ?? null, id);
      return { ...prev, [closing.workspace]: nextTree };
    });
    setFocusedWindowId((prev) => {
      if (prev !== id) return prev;
      const remaining = managedWindows.filter(
        (w) => w.workspace === closing.workspace && w.id !== id
      );
      return remaining.at(-1)?.id ?? null;
    });
  }, [managedWindows]);

  const toggleWindowMode = useCallback(
    (id: string) => {
      const target = managedWindows.find((w) => w.id === id);
      if (!target) return;

      const currentlyTiled = config.tilingEnabled && target.mode === 'tiling';

      if (currentlyTiled) {
        // Pop out into Floating Mode (smoothly shrinks from its current tiled rect or staggered rect)
        const currentRect = tiledRects[id];
        const nextFloatRect = currentRect
          ? {
              x: Math.round(
                currentRect.x + Math.min(32, currentRect.width * 0.08)
              ),
              y: Math.round(
                currentRect.y + Math.min(24, currentRect.height * 0.06)
              ),
              width: Math.max(
                420,
                Math.min(700, Math.round(currentRect.width * 0.86))
              ),
              height: Math.max(
                280,
                Math.min(480, Math.round(currentRect.height * 0.84))
              ),
            }
          : target.floatingRect;

        setTopZIndex((z) => {
          const nextZ = z + 1;
          setManagedWindows((prev) =>
            prev.map((w) =>
              w.id === id
                ? {
                    ...w,
                    mode: 'floating',
                    floatingRect: nextFloatRect,
                    zIndex: nextZ,
                  }
                : w
            )
          );
          return nextZ;
        });
        setTreesByWorkspace((prev) => ({
          ...prev,
          [target.workspace]: removeWindowFromTree(
            prev[target.workspace] ?? null,
            id
          ),
        }));
      } else {
        // Snap back into GlazeWM Tiling Tree (enabling global tiling if it was off)
        if (!config.tilingEnabled) {
          setConfig((c) => ({ ...c, tilingEnabled: true }));
        }
        setManagedWindows((prev) =>
          prev.map((w) => (w.id === id ? { ...w, mode: 'tiling' } : w))
        );
        setTreesByWorkspace((prev) => ({
          ...prev,
          [target.workspace]: insertWindowIntoTree(
            prev[target.workspace] ?? null,
            id,
            focusedWindowId && focusedWindowId !== id ? focusedWindowId : null,
            config.tilingDirection
          ),
        }));
      }
      setFocusedWindowId(id);
    },
    [
      managedWindows,
      config.tilingEnabled,
      config.tilingDirection,
      tiledRects,
      focusedWindowId,
    ]
  );

  const flipSplitDirectionForWindow = useCallback(
    (id: string | null) => {
      setConfig((prev) => ({
        ...prev,
        tilingDirection:
          prev.tilingDirection === 'horizontal' ? 'vertical' : 'horizontal',
      }));
      setTreesByWorkspace((prev) => {
        const currentTree = prev[activeWorkspace] ?? null;
        if (!currentTree) return prev;
        return {
          ...prev,
          [activeWorkspace]: toggleSplitDirectionAt(currentTree, id),
        };
      });
    },
    [activeWorkspace]
  );

  const swapTiledWindow = useCallback(
    (id: string, dir: 'prev' | 'next') => {
      setTreesByWorkspace((prev) => {
        const currentTree = prev[activeWorkspace] ?? null;
        if (!currentTree) return prev;
        return {
          ...prev,
          [activeWorkspace]: swapWindowWithNeighbor(currentTree, id, dir),
        };
      });
    },
    [activeWorkspace]
  );

  const resizeTiledWindow = useCallback(
    (id: string, delta: number) => {
      setTreesByWorkspace((prev) => {
        const currentTree = prev[activeWorkspace] ?? null;
        if (!currentTree) return prev;
        return {
          ...prev,
          [activeWorkspace]: resizeWindowInTree(currentTree, id, delta),
        };
      });
    },
    [activeWorkspace]
  );

  const updateFloatingRect = useCallback((id: string, rect: WindowRect) => {
    setManagedWindows((prev) =>
      prev.map((w) => (w.id === id ? { ...w, floatingRect: rect } : w))
    );
  }, []);

  const openThreeTileWorkspace = useCallback(() => {
    setSettingsOpen(false);
    setConfig((prev) => ({ ...prev, tilingEnabled: true }));
    const ts = Date.now();
    const idVscode = `win-vscode-${activeWorkspace}-${ts}`;
    const idTerm = `win-terminal-${activeWorkspace}-${ts + 1}`;
    const idFiles = `win-files-${activeWorkspace}-${ts + 2}`;

    const created: ManagedWindow[] = [
      {
        id: idVscode,
        appKind: 'vscode',
        title: APP_WINDOW_META.vscode.title,
        subtitle: APP_WINDOW_META.vscode.subtitle,
        workspace: activeWorkspace,
        mode: 'tiling',
        floatingRect: staggeredFloatingRect(workArea, 0),
        zIndex: topZIndex + 1,
      },
      {
        id: idTerm,
        appKind: 'terminal',
        title: APP_WINDOW_META.terminal.title,
        subtitle: APP_WINDOW_META.terminal.subtitle,
        workspace: activeWorkspace,
        mode: 'tiling',
        floatingRect: staggeredFloatingRect(workArea, 1),
        zIndex: topZIndex + 2,
      },
      {
        id: idFiles,
        appKind: 'files',
        title: APP_WINDOW_META.files.title,
        subtitle: APP_WINDOW_META.files.subtitle,
        workspace: activeWorkspace,
        mode: 'tiling',
        floatingRect: staggeredFloatingRect(workArea, 2),
        zIndex: topZIndex + 3,
      },
    ];

    setTopZIndex((z) => z + 4);
    setManagedWindows((prev) => [
      ...prev.filter((w) => w.workspace !== activeWorkspace),
      ...created,
    ]);
    setTreesByWorkspace((prev) => ({
      ...prev,
      [activeWorkspace]: createThreePanePresetTree(idVscode, idTerm, idFiles),
    }));
    setFocusedWindowId(idVscode);
    setFocusedAppId('vscode');
  }, [activeWorkspace, workArea, topZIndex]);

  const setAllWorkspaceWindowsMode = useCallback(
    (mode: 'tiling' | 'floating') => {
      setConfig((prev) => ({ ...prev, tilingEnabled: mode === 'tiling' }));
      setManagedWindows((prev) =>
        prev.map((w) =>
          w.workspace === activeWorkspace ? { ...w, mode } : w
        )
      );
      if (mode === 'tiling') {
        setTreesByWorkspace((prev) => {
          let tree = prev[activeWorkspace] ?? null;
          const wsWins = managedWindows.filter(
            (w) => w.workspace === activeWorkspace
          );
          const existingIds = new Set(collectWindowIds(tree));
          for (const w of wsWins) {
            if (!existingIds.has(w.id)) {
              tree = insertWindowIntoTree(
                tree,
                w.id,
                focusedWindowId,
                config.tilingDirection
              );
            }
          }
          return { ...prev, [activeWorkspace]: tree };
        });
      }
    },
    [activeWorkspace, managedWindows, focusedWindowId, config.tilingDirection]
  );

  const clearWorkspaceWindows = useCallback(() => {
    setManagedWindows((prev) =>
      prev.filter((w) => w.workspace !== activeWorkspace)
    );
    setTreesByWorkspace((prev) => ({ ...prev, [activeWorkspace]: null }));
    setFocusedWindowId(null);
  }, [activeWorkspace]);

  // GlazeWM / COSMIC Global Keyboard Shortcuts
  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      const target = e.target as HTMLElement | null;
      if (
        target &&
        (target.tagName === 'INPUT' ||
          target.tagName === 'TEXTAREA' ||
          target.isContentEditable)
      ) {
        return;
      }
      if (e.altKey && e.shiftKey && e.key.toLowerCase() === 't') {
        e.preventDefault();
        setConfig((prev) => ({ ...prev, tilingEnabled: !prev.tilingEnabled }));
      } else if (e.altKey && e.code === 'Space' && focusedWindowId) {
        e.preventDefault();
        toggleWindowMode(focusedWindowId);
      } else if (e.altKey && !e.shiftKey && e.key.toLowerCase() === 'v') {
        e.preventDefault();
        flipSplitDirectionForWindow(focusedWindowId);
      } else if (
        e.altKey &&
        !e.shiftKey &&
        e.key.toLowerCase() === 'h' &&
        focusedWindowId
      ) {
        e.preventDefault();
        swapTiledWindow(focusedWindowId, 'prev');
      } else if (
        e.altKey &&
        !e.shiftKey &&
        e.key.toLowerCase() === 'l' &&
        focusedWindowId
      ) {
        e.preventDefault();
        swapTiledWindow(focusedWindowId, 'next');
      } else if (e.key === 'Escape') {
        setActivePopup(null);
        setContextMenuPos(null);
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [
    focusedWindowId,
    toggleWindowMode,
    flipSplitDirectionForWindow,
    swapTiledWindow,
  ]);

  // Live simulated hardware & applet states
  const [masterVolume, setMasterVolume] = useState<number>(78);
  const [masterMuted, setMasterMuted] = useState<boolean>(false);
  const [audioSessions, setAudioSessions] = useState<AudioSessionRow[]>([
    {
      id: 'cosmic-term',
      label: 'COSMIC Terminal (wt.exe)',
      process: 'wt.exe',
      volume: 85,
      muted: false,
    },
    {
      id: 'rust-rover',
      label: 'Visual Studio Code (Code.exe)',
      process: 'Code.exe',
      volume: 65,
      muted: false,
    },
    {
      id: 'brave-media',
      label: 'Brave Browser WebAudio',
      process: 'brave.exe',
      volume: 92,
      muted: false,
    },
  ]);

  const [wifiSsid, setWifiSsid] = useState<string>('System76-COSMIC-5G');
  const [wifiConnected, setWifiConnected] = useState<boolean>(true);
  const [bluetoothDevices, setBluetoothDevices] = useState([
    {
      id: 'bt-1',
      name: 'Sony WH-1000XM5 Earbuds',
      service: 'AudioSink (0x110B)',
      connected: true,
    },
    {
      id: 'bt-2',
      name: 'Keychron Q1 Pro Mechanical',
      service: 'HID Keyboard',
      connected: true,
    },
    {
      id: 'bt-3',
      name: 'Galaxy Buds3 Pro',
      service: 'Handsfree (0x111E)',
      connected: false,
    },
  ]);

  const [doNotDisturb, setDoNotDisturb] = useState<boolean>(false);
  const [mediaPlaying, setMediaPlaying] = useState<boolean>(true);
  const [mediaTrack] = useState<string>('Solaris Synth · Spotify');
  const [trayItems, setTrayItems] = useState<SystemTrayItem[]>([
    {
      id: 'discord',
      name: 'Discord',
      process: 'Discord.exe',
      status: 'Online · Voice Ready',
      active: true,
    },
    {
      id: 'nvidia',
      name: 'NVIDIA GPU Monitor',
      process: 'nvcontainer.exe',
      status: '42°C · WGPU Ready',
      active: true,
    },
    {
      id: 'steam',
      name: 'Steam Client',
      process: 'steam.exe',
      status: 'Running in background',
      active: true,
    },
    {
      id: 'onedrive',
      name: 'OneDrive Sync',
      process: 'OneDrive.exe',
      status: 'Up to date',
      active: true,
    },
  ]);

  const [launcherApps, setLauncherApps] = useState<LauncherAppEntry[]>([
    {
      id: 'app-dock',
      name: 'COSMIC Dock Settings',
      category: 'Luma Settings',
      target: 'ms-luma:settings/dock',
      favorite: true,
    },
    {
      id: 'app-applets',
      name: 'COSMIC Panel Applets & Zones',
      category: 'Luma Settings',
      target: 'ms-luma:settings/applets',
      favorite: true,
    },
    {
      id: 'app-appearance',
      name: 'Global Accent & Wallpaper Studio',
      category: 'Luma Settings',
      target: 'ms-luma:settings/appearance',
      favorite: true,
    },
    {
      id: 'app-gpu',
      name: 'WGPU & Orbital Earth Shader',
      category: 'GPU Pipeline',
      target: 'cargo run --bin luma_iced',
      favorite: true,
    },
    {
      id: 'app-term',
      name: 'Windows Terminal (wt.exe)',
      category: 'Developer Tools',
      target: 'wt.exe',
      favorite: true,
    },
    {
      id: 'app-files',
      name: 'File Explorer',
      category: 'System',
      target: 'explorer.exe',
      favorite: false,
    },
  ]);

  // Exact pinned dock apps matching the user's COSMIC Dock screenshot
  const [pinnedAppOrder, setPinnedAppOrder] = useState<DockAppItem[]>([
    {
      id: 'brave',
      label: 'Brave Browser',
      icon: 'brave',
      running: true,
      active: true,
      launcherId: 'browser',
    },
    {
      id: 'youtube',
      label: 'YouTube',
      icon: 'youtube',
      running: false,
      active: false,
    },
    {
      id: 'antigravity',
      label: 'Antigravity Studio',
      icon: 'antigravity',
      running: false,
      active: false,
    },
    {
      id: 'chatgpt',
      label: 'ChatGPT Desktop',
      icon: 'chatgpt',
      running: false,
      active: false,
    },
    {
      id: 'whatsapp',
      label: 'WhatsApp',
      icon: 'whatsapp',
      running: false,
      active: false,
    },
    {
      id: 'roblox',
      label: 'Roblox Player',
      icon: 'roblox',
      running: false,
      active: false,
    },
    {
      id: 'spotify',
      label: 'Spotify',
      icon: 'spotify',
      running: false,
      active: false,
    },
    {
      id: 'vscode',
      label: 'Visual Studio Code',
      icon: 'vscode',
      running: true,
      active: true,
    },
  ]);

  const [accR, accG, accB] = hexToRgbTuple(config.accentColor);

  useEffect(() => {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(config));
    } catch {
      // ignore storage errors
    }
    const root = document.documentElement;
    if (config.dark) {
      root.classList.add('dark');
    } else {
      root.classList.remove('dark');
    }
    root.style.setProperty('--cosmic-accent', config.accentColor);
    root.style.setProperty('--cosmic-accent-rgb', `${accR}, ${accG}, ${accB}`);
    root.style.setProperty(
      '--cosmic-font-scale',
      (config.fontScale / 100).toFixed(2)
    );
  }, [config, accR, accG, accB]);

  const handleTogglePopup = (popup: ActivePopup) => {
    setContextMenuPos(null);
    setActivePopup((prev) => (prev === popup ? null : popup));
  };

  const handleActivateDockLauncher = (id: DockLauncherId) => {
    setContextMenuPos(null);
    if (id === 'home') {
      handleTogglePopup('launcher');
    } else if (id === 'settings') {
      setSettingsOpen((prev) => !prev);
      setActivePopup(null);
    } else if (id === 'terminal') {
      openManagedAppWindow('terminal');
      setActivePopup(null);
    } else if (id === 'files') {
      openManagedAppWindow('files');
      setActivePopup(null);
    } else if (id === 'browser') {
      openManagedAppWindow('brave');
      setActivePopup(null);
    }
  };

  const handleActivateDockApp = (app: DockAppItem) => {
    setContextMenuPos(null);
    setFocusedAppId(app.id);

    // Mark pinned app as running when clicked (simulating real Windows process launch)
    setPinnedAppOrder((prev) =>
      prev.map((item) =>
        item.id === app.id ? { ...item, running: true } : item
      )
    );

    if (app.id === 'settings') {
      setSettingsOpen((prev) => !prev);
      setActivePopup(null);
    } else if (app.id === 'terminal') {
      openManagedAppWindow('terminal');
      setActivePopup(null);
    } else if (app.id === 'vscode') {
      openManagedAppWindow('vscode');
      setActivePopup(null);
    } else if (app.id === 'brave') {
      openManagedAppWindow('brave');
      setActivePopup(null);
    } else if (app.id === 'antigravity') {
      setSettingsOpen(true);
      setSettingsPage('gpu');
      setActivePopup(null);
    } else if (app.id === 'spotify' || app.id === 'youtube') {
      setSettingsOpen(true);
      setSettingsPage('applets');
      setActivePopup(null);
    } else {
      openManagedAppWindow('notes');
      setActivePopup(null);
    }
  };

  const handleReorderPinned = (fromIdx: number, toIdx: number) => {
    setPinnedAppOrder((prev) => {
      if (
        fromIdx < 0 ||
        toIdx < 0 ||
        fromIdx >= prev.length ||
        toIdx >= prev.length ||
        fromIdx === toIdx
      ) {
        return prev;
      }
      const next = [...prev];
      const [moved] = next.splice(fromIdx, 1);
      next.splice(toIdx, 0, moved);
      return next;
    });
  };

  // Highlight Brave & active/focused app (e.g., VS Code) with circular halos matching screenshot
  const computedPinnedApps: DockAppItem[] = pinnedAppOrder.map((app) => ({
    ...app,
    active:
      app.id === focusedAppId ||
      (app.id === 'brave' && focusedAppId === 'vscode'),
  }));

  // Right side of the vertical divider: currently running system apps (Terminal & COSMIC Settings)
  const runningSystemApps: DockAppItem[] = [
    {
      id: 'terminal',
      label: 'Windows Terminal',
      icon: 'terminal',
      running: true,
      active: workspaceWindows.some((w) => w.appKind === 'terminal'),
      launcherId: 'terminal',
    },
    {
      id: 'settings',
      label: 'COSMIC Settings',
      icon: 'settings',
      running: true,
      active: settingsOpen,
      launcherId: 'settings',
    },
  ];

  const focusedWindow =
    workspaceWindows.find((w) => w.id === focusedWindowId) ??
    workspaceWindows.at(-1) ??
    null;

  return (
    <div
      data-state="cosmic-desktop-active"
      style={
        {
          '--cosmic-accent': config.accentColor,
          '--cosmic-accent-rgb': `${accR}, ${accG}, ${accB}`,
          fontFamily: resolveFontStack(config.fontFamily),
        } as React.CSSProperties
      }
      className={`relative w-screen h-screen overflow-hidden select-none flex flex-col justify-between ${
        config.dark ? 'bg-slate-950 text-slate-100' : 'bg-slate-100 text-slate-900'
      }`}
    >
      {/* 1. WebGL2 / Hardware-Accelerated GPU Desktop Shader Surface (Default: Orbital Earth Horizon) */}
      <GpuCompositorCanvas
        config={config}
        animateShader={animateShader}
        onDesktopClick={() => {
          setActivePopup(null);
          setContextMenuPos(null);
        }}
        onDesktopContextMenu={(x, y) => {
          setActivePopup(null);
          setContextMenuPos({ x, y });
        }}
      />

      {/* 2. COSMIC Top Panel with 3-Zone Applets (Left: Workspaces 1 2 3 (4), Center: 23 Nov, 11:13 am, Right: Tiling + US + System Tray Icons + Status Applets) */}
      <CosmicTopPanel
        config={config}
        activeWorkspace={activeWorkspace}
        onSelectWorkspace={setActiveWorkspace}
        activePopup={activePopup}
        onTogglePopup={handleTogglePopup}
        onToggleSettings={() => setSettingsOpen((v) => !v)}
        managedWindowCount={workspaceWindows.length}
        trayItems={trayItems}
        onSelectTrayItem={() => handleTogglePopup('tray')}
        masterVolume={masterVolume}
        masterMuted={masterMuted}
        activeAudioSessions={
          audioSessions.filter((s) => !s.muted && s.volume > 0).length
        }
        wifiSsid={wifiSsid}
        wifiConnected={wifiConnected}
        bluetoothConnected={bluetoothDevices.some((d) => d.connected)}
        batteryPercent={94}
        doNotDisturb={doNotDisturb}
        onToggleDoNotDisturb={() => setDoNotDisturb((v) => !v)}
        mediaPlaying={mediaPlaying}
        mediaTrack={mediaTrack}
        onToggleMediaPlay={() => setMediaPlaying((v) => !v)}
      />

      {/* 2B. GlazeWM SplitContainer × COSMIC Animated Tiling & Floating Window Layer */}
      <CosmicWindowLayer
        config={config}
        windows={workspaceWindows}
        tiledRects={tiledRects}
        workArea={workArea}
        focusedWindowId={focusedWindow?.id ?? null}
        onFocusWindow={focusWindow}
        onCloseWindow={closeManagedWindow}
        onToggleWindowMode={toggleWindowMode}
        onFlipSplitDirection={(id) => flipSplitDirectionForWindow(id)}
        onSwapWindow={swapTiledWindow}
        onResizeTiledWindow={resizeTiledWindow}
        onUpdateFloatingRect={updateFloatingRect}
      />

      {/* 3. Main COSMIC Settings Studio Window */}
      {settingsOpen && (
        <div className="relative z-30 flex-1 flex items-start justify-center overflow-hidden px-3">
          <CosmicSettingsWindow
            config={config}
            onChangeConfig={setConfig}
            activePage={settingsPage}
            onSelectPage={setSettingsPage}
            onClose={() => setSettingsOpen(false)}
            animateShader={animateShader}
            onToggleAnimateShader={() => setAnimateShader((v) => !v)}
            onLaunchDockApp={handleActivateDockLauncher}
          />
        </div>
      )}

      {/* 4. Interactive Shell Popups (Launcher, Audio Mixer, Network, Bluetooth, Switcher, Power) */}
      <CosmicPopups
        config={config}
        activePopup={activePopup}
        onClosePopup={() => setActivePopup(null)}
        masterVolume={masterVolume}
        masterMuted={masterMuted}
        onChangeMasterVolume={(vol) => {
          setMasterVolume(vol);
          if (vol > 0) setMasterMuted(false);
        }}
        onToggleMasterMute={() => setMasterMuted((m) => !m)}
        audioSessions={audioSessions}
        onChangeSessionVolume={(id, vol) =>
          setAudioSessions((rows) =>
            rows.map((r) =>
              r.id === id ? { ...r, volume: vol, muted: false } : r
            )
          )
        }
        onToggleSessionMute={(id) =>
          setAudioSessions((rows) =>
            rows.map((r) => (r.id === id ? { ...r, muted: !r.muted } : r))
          )
        }
        wifiSsid={wifiSsid}
        wifiConnected={wifiConnected}
        onConnectWifi={(ssid) => {
          setWifiSsid(ssid);
          setWifiConnected(true);
        }}
        bluetoothDevices={bluetoothDevices}
        onToggleBluetoothDevice={(id) =>
          setBluetoothDevices((devs) =>
            devs.map((d) =>
              d.id === id ? { ...d, connected: !d.connected } : d
            )
          )
        }
        launcherApps={launcherApps}
        onToggleFavoriteApp={(id) =>
          setLauncherApps((apps) =>
            apps.map((a) => (a.id === id ? { ...a, favorite: !a.favorite } : a))
          )
        }
        onLaunchApp={(app) => {
          if (app.id === 'app-dock') {
            setSettingsOpen(true);
            setSettingsPage('dock');
          } else if (app.id === 'app-applets') {
            setSettingsOpen(true);
            setSettingsPage('applets');
          } else if (app.id === 'app-appearance') {
            setSettingsOpen(true);
            setSettingsPage('appearance');
          } else if (app.id === 'app-gpu') {
            setSettingsOpen(true);
            setSettingsPage('gpu');
          } else if (app.id === 'app-term') {
            openManagedAppWindow('terminal');
          } else if (app.id === 'app-files') {
            openManagedAppWindow('files');
          }
        }}
        trayItems={trayItems}
        onToggleTrayItemActive={(id) =>
          setTrayItems((items) =>
            items.map((it) => (it.id === id ? { ...it, active: !it.active } : it))
          )
        }
        activeWorkspace={activeWorkspace}
        onSelectWorkspace={setActiveWorkspace}
        onOpenSettingsPage={(page) => {
          setSettingsOpen(true);
          setSettingsPage(page);
        }}
        onUpdateConfig={setConfig}
        managedWindowCount={workspaceWindows.length}
        focusedWindowTitle={focusedWindow?.title ?? null}
        focusedWindowIsFloating={
          focusedWindow
            ? !config.tilingEnabled || focusedWindow.mode === 'floating'
            : false
        }
        onToggleFocusedFloat={() => {
          if (focusedWindow) toggleWindowMode(focusedWindow.id);
        }}
        onFlipFocusedSplit={() =>
          flipSplitDirectionForWindow(focusedWindow?.id ?? null)
        }
        onSwapFocusedWindow={(dir) => {
          if (focusedWindow) swapTiledWindow(focusedWindow.id, dir);
        }}
        onOpenThreeTilePreset={openThreeTileWorkspace}
        onSetAllWindowsMode={setAllWorkspaceWindowsMode}
        onClearWorkspaceWindows={clearWorkspaceWindows}
      />

      {/* 6. Desktop Right-Click Context Menu */}
      {contextMenuPos && (
        <div
          role="menu"
          aria-label="Luma Desktop Context Menu"
          style={{
            left: `${Math.min(contextMenuPos.x, window.innerWidth - 230)}px`,
            top: `${Math.min(contextMenuPos.y, window.innerHeight - 260)}px`,
          }}
          className={`fixed z-50 w-56 rounded-xl p-1.5 border shadow-2xl ${
            config.dark
              ? 'bg-slate-900/95 border-white/15 text-slate-100'
              : 'bg-white/95 border-slate-300 text-slate-900'
          }`}
        >
          <button
            type="button"
            onClick={() => {
              setSettingsOpen(true);
              setSettingsPage('appearance');
              setContextMenuPos(null);
            }}
            className="w-full flex items-center gap-2.5 px-3 py-2 rounded-lg text-xs font-medium hover:bg-white/10 text-left"
          >
            <Palette
              style={{ color: config.accentColor }}
              className="w-3.5 h-3.5"
            />
            <span>Global Accent & Wallpaper</span>
          </button>
          <button
            type="button"
            onClick={() => {
              setSettingsOpen(true);
              setSettingsPage('dock');
              setContextMenuPos(null);
            }}
            className="w-full flex items-center gap-2.5 px-3 py-2 rounded-lg text-xs font-medium hover:bg-white/10 text-left"
          >
            <LayoutGrid
              style={{ color: config.accentColor }}
              className="w-3.5 h-3.5"
            />
            <span>COSMIC Dock Settings</span>
          </button>
          <button
            type="button"
            onClick={() => {
              setSettingsOpen(true);
              setSettingsPage('applets');
              setContextMenuPos(null);
            }}
            className="w-full flex items-center gap-2.5 px-3 py-2 rounded-lg text-xs font-medium hover:bg-white/10 text-left"
          >
            <PanelTop
              style={{ color: config.accentColor }}
              className="w-3.5 h-3.5"
            />
            <span>Panel Applets & Zones</span>
          </button>
          <button
            type="button"
            onClick={() => {
              setSettingsOpen(true);
              setSettingsPage('gpu');
              setContextMenuPos(null);
            }}
            className="w-full flex items-center gap-2.5 px-3 py-2 rounded-lg text-xs font-medium hover:bg-white/10 text-left"
          >
            <Cpu
              style={{ color: config.accentColor }}
              className="w-3.5 h-3.5"
            />
            <span>GPU Shader & Blur</span>
          </button>
          <div className="my-1 border-t border-white/10" />
          <button
            type="button"
            onClick={() => {
              openThreeTileWorkspace();
              setContextMenuPos(null);
            }}
            className="w-full flex items-center gap-2.5 px-3 py-2 rounded-lg text-xs font-medium hover:bg-white/10 text-left"
          >
            <Sparkles
              style={{ color: config.accentColor }}
              className="w-3.5 h-3.5"
            />
            <span>Open 3-Tile Workspace</span>
          </button>
          <button
            type="button"
            onClick={() => {
              openManagedAppWindow('terminal');
              setContextMenuPos(null);
            }}
            className="w-full flex items-center gap-2.5 px-3 py-2 rounded-lg text-xs font-medium hover:bg-white/10 text-left"
          >
            <Terminal className="w-3.5 h-3.5 text-emerald-400" />
            <span>Open COSMIC Terminal</span>
          </button>
          <button
            type="button"
            onClick={() => {
              openManagedAppWindow('files');
              setContextMenuPos(null);
            }}
            className="w-full flex items-center gap-2.5 px-3 py-2 rounded-lg text-xs font-medium hover:bg-white/10 text-left"
          >
            <FolderOpen className="w-3.5 h-3.5 text-amber-400" />
            <span>Open Luma Files</span>
          </button>
          <button
            type="button"
            onClick={() => {
              setSettingsOpen(true);
              setSettingsPage('config_sync');
              setContextMenuPos(null);
            }}
            className="w-full flex items-center gap-2.5 px-3 py-2 rounded-lg text-xs font-medium hover:bg-white/10 text-left"
          >
            <Sliders className="w-3.5 h-3.5 text-slate-400" />
            <span>Export settings.ini</span>
          </button>
        </div>
      )}

      {/* 7. COSMIC Pill Capsule Dock with High-Res App Icons & Adaptive Translucency */}
      <CosmicDock
        config={config}
        hasAppBehind={settingsOpen || workspaceWindows.length > 0}
        pinnedApps={computedPinnedApps}
        runningApps={runningSystemApps}
        onActivateApp={handleActivateDockApp}
        onReorderPinned={handleReorderPinned}
      />
    </div>
  );
}
export default App;
