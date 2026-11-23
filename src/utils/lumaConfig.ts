export type DockPosition = 'left' | 'center' | 'right';
export type DockStyle = 'pill' | 'transparent' | 'island' | 'panel';
export type PanelStyle = 'transparent' | 'full' | 'island';
export type DateFormatMode = 'none' | 'short' | 'long';
export type PanelZone = 'left' | 'center' | 'right';
export type TilingDirection = 'horizontal' | 'vertical';
export type GpuBackendMode = 'wgpu' | 'direct2d' | 'software';
export type GpuShaderPreset =
  | 'orbital_earth'
  | 'cosmic_nebula'
  | 'system76_obsidian'
  | 'solar_flare'
  | 'teal_aurora';

export type WallpaperMode =
  | 'gpu_shader'
  | 'vector_scene'
  | 'gradient'
  | 'custom_image';

export type VectorWallpaperPreset =
  | 'orbit_topography'
  | 'nordic_ridges'
  | 'isometric_grid'
  | 'sunset_dunes';

export type FontFamilyPreset =
  | 'jakarta'
  | 'dm_sans'
  | 'system_segoe'
  | 'jetbrains_mono';

export type AppletId =
  | 'clock'
  | 'network'
  | 'volume'
  | 'bluetooth'
  | 'battery'
  | 'notifications'
  | 'media';

export type DockLauncherId =
  | 'home'
  | 'files'
  | 'terminal'
  | 'browser'
  | 'settings';

export const ALL_APPLETS: AppletId[] = [
  'clock',
  'network',
  'volume',
  'bluetooth',
  'battery',
  'notifications',
  'media',
];

export const ALL_DOCK_LAUNCHERS: DockLauncherId[] = [
  'home',
  'files',
  'terminal',
  'browser',
  'settings',
];

export interface LumaConfig {
  version: number;
  dark: boolean;
  highContrast: boolean;
  desktopEnabled: boolean;
  clock24h: boolean;
  dateFormat: DateFormatMode;
  // Global Personalization (Accent, Wallpaper, Typography)
  accentColor: string;
  syncAccentToWallpaper: boolean;
  wallpaperMode: WallpaperMode;
  wallpaperVectorPreset: VectorWallpaperPreset;
  customWallpaperUrl: string;
  wallpaperDimming: number; // 0..70
  fontFamily: FontFamilyPreset;
  fontScale: number; // 90..115 (%)
  // Dock configuration (COSMIC Dock)
  position: DockPosition;
  dockStyle: DockStyle;
  adaptiveDockTransparency: boolean;
  autoHide: boolean;
  iconSize: number; // 24..64
  magnification: number; // 0..100
  transparency: number; // 40..230
  spacing: number; // 36..96
  offsetX: number; // -200..200
  offsetY: number; // -100..100
  cornerRadius: number; // 8..28
  dockOrder: DockLauncherId[];
  showPinnedApps: boolean;
  showRunningIndicators: boolean;
  // Panel & Applets configuration (COSMIC Panel)
  panelStyle: PanelStyle;
  panelHeight: number; // 32..48
  panelMargin: number; // 0..12
  appletEnabled: Record<AppletId, boolean>;
  appletOrder: AppletId[];
  panelZones: Record<AppletId, PanelZone>;
  // GPU Rendering & Compositor
  gpuBackend: GpuBackendMode;
  gpuShader: GpuShaderPreset;
  gpuBlurRadius: number; // 0..40
  gpuBloomStrength: number; // 0..100
  gpuSpecularBorder: boolean;
  gpuVsync: boolean;
  // GlazeWM-inspired COSMIC Tiling & Floating Window Manager
  tilingEnabled: boolean;
  tilingDirection: TilingDirection;
  tilingInnerGap: number; // 0..24
  tilingOuterGap: number; // 0..24
  tilingActiveHint: boolean;
  tilingAnimationMs: number; // 0..500
  // Shell replacement status
  lumaShellActive: boolean;
  startWithWindows: boolean;
}

export const COSMIC_ACCENTS: { id: string; name: string; hex: string }[] = [
  { id: 'cosmic-cyan', name: 'COSMIC Cyan', hex: '#38bdf8' },
  { id: 'system76-amber', name: 'Amber Gold', hex: '#f59e0b' },
  { id: 'nebula-teal', name: 'Nebula Teal', hex: '#2dd4bf' },
  { id: 'aurora-indigo', name: 'Aurora Indigo', hex: '#818cf8' },
  { id: 'plasma-coral', name: 'Plasma Coral', hex: '#fb7185' },
  { id: 'emerald-pulse', name: 'Emerald Jade', hex: '#34d399' },
  { id: 'solar-orange', name: 'Pop! Orange', hex: '#f97316' },
  { id: 'violet-nova', name: 'Nova Violet', hex: '#c084fc' },
];

export function hexToRgbTuple(hex: string): [number, number, number] {
  const clean = hex.trim().replace(/^#/, '');
  if (!/^[0-9a-fA-F]{6}$/.test(clean)) {
    return [56, 189, 248];
  }
  const num = parseInt(clean, 16);
  return [(num >> 16) & 255, (num >> 8) & 255, num & 255];
}

export function defaultLumaConfig(): LumaConfig {
  return {
    version: 10,
    dark: true,
    highContrast: false,
    desktopEnabled: true,
    clock24h: false,
    dateFormat: 'short',
    accentColor: '#38bdf8',
    syncAccentToWallpaper: false,
    wallpaperMode: 'gpu_shader',
    wallpaperVectorPreset: 'orbit_topography',
    customWallpaperUrl: '',
    wallpaperDimming: 0,
    fontFamily: 'jakarta',
    fontScale: 100,
    position: 'center',
    dockStyle: 'pill',
    adaptiveDockTransparency: true,
    autoHide: false,
    iconSize: 42,
    magnification: 32,
    transparency: 148,
    spacing: 56,
    offsetX: 0,
    offsetY: 0,
    cornerRadius: 28,
    dockOrder: ['home', 'files', 'terminal', 'browser', 'settings'],
    showPinnedApps: true,
    showRunningIndicators: true,
    panelStyle: 'transparent',
    panelHeight: 34,
    panelMargin: 4,
    appletEnabled: {
      clock: true,
      network: true,
      volume: true,
      bluetooth: true,
      battery: true,
      notifications: true,
      media: true,
    },
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
    gpuBackend: 'wgpu',
    gpuShader: 'orbital_earth',
    gpuBlurRadius: 24,
    gpuBloomStrength: 45,
    gpuSpecularBorder: true,
    gpuVsync: true,
    tilingEnabled: true,
    tilingDirection: 'horizontal',
    tilingInnerGap: 10,
    tilingOuterGap: 12,
    tilingActiveHint: true,
    tilingAnimationMs: 240,
    lumaShellActive: true,
    startWithWindows: true,
  };
}

export function parseAppletOrder(raw: string): AppletId[] {
  const result: AppletId[] = [];
  const used = new Set<AppletId>();
  for (const token of raw.split(',')) {
    const trimmed = token.trim().toLowerCase() as AppletId;
    if (ALL_APPLETS.includes(trimmed) && !used.has(trimmed)) {
      result.push(trimmed);
      used.add(trimmed);
    }
  }
  for (const id of ALL_APPLETS) {
    if (!used.has(id)) {
      result.push(id);
    }
  }
  return result;
}

export function formatAppletOrder(order: AppletId[]): string {
  return parseAppletOrder(order.join(',')).join(',');
}

export function parsePanelZones(raw: string): Record<AppletId, PanelZone> {
  const zones: Record<AppletId, PanelZone> = {
    clock: 'right',
    network: 'right',
    volume: 'right',
    bluetooth: 'right',
    battery: 'right',
    notifications: 'right',
    media: 'right',
  };
  for (const token of raw.split(',')) {
    const parts = token.trim().split(/[:=]/);
    if (parts.length !== 2) continue;
    const name = parts[0].trim().toLowerCase() as AppletId;
    const zoneRaw = parts[1].trim().toLowerCase();
    if (!ALL_APPLETS.includes(name)) continue;
    zones[name] =
      zoneRaw === 'left'
        ? 'left'
        : zoneRaw === 'center'
          ? 'center'
          : 'right';
  }
  return zones;
}

export function formatPanelZones(zones: Record<AppletId, PanelZone>): string {
  return ALL_APPLETS.map((id) => `${id}:${zones[id] ?? 'right'}`).join(',');
}

export function parseDockOrder(raw: string): DockLauncherId[] {
  const result: DockLauncherId[] = [];
  const used = new Set<DockLauncherId>();
  for (const token of raw.split(',')) {
    const trimmed = token.trim().toLowerCase() as DockLauncherId;
    if (ALL_DOCK_LAUNCHERS.includes(trimmed) && !used.has(trimmed)) {
      result.push(trimmed);
      used.add(trimmed);
    }
  }
  for (const id of ALL_DOCK_LAUNCHERS) {
    if (!used.has(id)) {
      result.push(id);
    }
  }
  return result;
}

export function formatDockOrder(order: DockLauncherId[]): string {
  return parseDockOrder(order.join(',')).join(',');
}

export function clamp(val: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, val));
}

export function calculateDockIconSize(
  baseSize: number,
  magnificationPercent: number,
  pointerX: number | null,
  slotCenterX: number,
  magnificationEnvelope = 1.0
): number {
  if (pointerX === null || magnificationPercent <= 0) {
    return baseSize;
  }
  const distance = Math.abs(pointerX - slotCenterX);
  const radius = 115;
  if (distance >= radius) {
    return baseSize;
  }
  // Smooth cosine falloff matching GPU shader interpolation
  const normalized = 1 - distance / radius;
  const smoothFactor = 0.5 * (1 - Math.cos(normalized * Math.PI));
  const influence =
    smoothFactor * magnificationEnvelope * (magnificationPercent / 45.0);
  return Math.round(baseSize + baseSize * 0.45 * influence);
}

export function calculateDockLogicalWidth(
  coreCount: number,
  pinnedCount: number,
  spacing: number
): number {
  const totalIcons = coreCount + pinnedCount;
  const separatorExtra = pinnedCount > 0 ? 18 : 0;
  return 48 + totalIcons * spacing + separatorExtra;
}

export function moveAppletToZone(
  config: LumaConfig,
  applet: AppletId,
  targetZone: PanelZone
): LumaConfig {
  return {
    ...config,
    panelZones: {
      ...config.panelZones,
      [applet]: targetZone,
    },
  };
}

export function reorderAppletInZone(
  config: LumaConfig,
  applet: AppletId,
  direction: 'up' | 'down'
): LumaConfig {
  const zone = config.panelZones[applet];
  const zoneApplets = config.appletOrder.filter(
    (id) => config.panelZones[id] === zone
  );
  const idx = zoneApplets.indexOf(applet);
  if (idx === -1) return config;
  const targetIdx = direction === 'up' ? idx - 1 : idx + 1;
  if (targetIdx < 0 || targetIdx >= zoneApplets.length) return config;

  const swapWith = zoneApplets[targetIdx];
  const newOrder = [...config.appletOrder];
  const globalA = newOrder.indexOf(applet);
  const globalB = newOrder.indexOf(swapWith);
  newOrder[globalA] = swapWith;
  newOrder[globalB] = applet;

  return {
    ...config,
    appletOrder: newOrder,
  };
}

export function reorderDockItem(
  order: DockLauncherId[],
  fromIndex: number,
  toIndex: number
): DockLauncherId[] {
  if (
    fromIndex < 0 ||
    toIndex < 0 ||
    fromIndex >= order.length ||
    toIndex >= order.length ||
    fromIndex === toIndex
  ) {
    return order;
  }
  const copy = [...order];
  const [moved] = copy.splice(fromIndex, 1);
  copy.splice(toIndex, 0, moved);
  return copy;
}

export function serializeSettingsIni(c: LumaConfig): string {
  return [
    `version=${c.version}`,
    `dark=${c.dark}`,
    `desktop=${c.desktopEnabled}`,
    `accent_color=${c.accentColor}`,
    `sync_accent_wallpaper=${c.syncAccentToWallpaper}`,
    `wallpaper_mode=${c.wallpaperMode}`,
    `wallpaper_preset=${c.wallpaperVectorPreset}`,
    `wallpaper_dimming=${clamp(Math.round(c.wallpaperDimming), 0, 70)}`,
    `font_family=${c.fontFamily}`,
    `font_scale=${clamp(Math.round(c.fontScale), 90, 115)}`,
    `clock=${c.appletEnabled.clock}`,
    `network=${c.appletEnabled.network}`,
    `volume=${c.appletEnabled.volume}`,
    `battery=${c.appletEnabled.battery}`,
    `notifications=${c.appletEnabled.notifications}`,
    `bluetooth=${c.appletEnabled.bluetooth}`,
    `media=${c.appletEnabled.media}`,
    `position=${c.position}`,
    `dock_style=${c.dockStyle}`,
    `adaptive_dock_transparency=${c.adaptiveDockTransparency}`,
    `panel_style=${c.panelStyle}`,
    `auto_hide=${c.autoHide}`,
    `icon_size=${clamp(Math.round(c.iconSize), 24, 64)}`,
    `magnification=${clamp(Math.round(c.magnification), 0, 100)}`,
    `transparency=${clamp(Math.round(c.transparency), 40, 230)}`,
    `spacing=${clamp(Math.round(c.spacing), 36, 96)}`,
    `corner_radius=${clamp(Math.round(c.cornerRadius), 8, 28)}`,
    `offset_x=${clamp(Math.round(c.offsetX), -200, 200)}`,
    `offset_y=${clamp(Math.round(c.offsetY), -100, 100)}`,
    `clock_24h=${c.clock24h}`,
    `date_format=${c.dateFormat}`,
    `order=${formatAppletOrder(c.appletOrder)}`,
    `zones=${formatPanelZones(c.panelZones)}`,
    `dock_order=${formatDockOrder(c.dockOrder)}`,
    `gpu_backend=${c.gpuBackend}`,
    `gpu_shader=${c.gpuShader}`,
    `gpu_blur_radius=${clamp(Math.round(c.gpuBlurRadius), 0, 40)}`,
    `gpu_bloom=${clamp(Math.round(c.gpuBloomStrength), 0, 100)}`,
    `gpu_specular=${c.gpuSpecularBorder}`,
    `tiling_enabled=${c.tilingEnabled}`,
    `tiling_direction=${c.tilingDirection}`,
    `tiling_inner_gap=${clamp(Math.round(c.tilingInnerGap), 0, 24)}`,
    `tiling_outer_gap=${clamp(Math.round(c.tilingOuterGap), 0, 24)}`,
    `tiling_active_hint=${c.tilingActiveHint}`,
    `tiling_animation_ms=${clamp(Math.round(c.tilingAnimationMs), 0, 500)}`,
  ].join('\n');
}

export function parseSettingsIni(raw: string): LumaConfig {
  const c = defaultLumaConfig();
  for (const line of raw.split(/\r?\n/)) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith('#') || trimmed.startsWith(';')) {
      continue;
    }
    const eqIdx = trimmed.indexOf('=');
    if (eqIdx === -1) continue;
    const k = trimmed.slice(0, eqIdx).trim();
    const v = trimmed.slice(eqIdx + 1).trim();

    switch (k) {
      case 'version':
        c.version = parseInt(v, 10) || 10;
        break;
      case 'dark':
        c.dark = v === 'true';
        break;
      case 'desktop':
        c.desktopEnabled = v === 'true';
        break;
      case 'accent_color':
        if (/^#[0-9a-fA-F]{6}$/.test(v)) {
          c.accentColor = v;
        }
        break;
      case 'sync_accent_wallpaper':
        c.syncAccentToWallpaper = v === 'true';
        break;
      case 'wallpaper_mode':
        if (
          v === 'gpu_shader' ||
          v === 'vector_scene' ||
          v === 'gradient' ||
          v === 'custom_image'
        ) {
          c.wallpaperMode = v;
        }
        break;
      case 'wallpaper_preset':
        if (
          v === 'orbit_topography' ||
          v === 'nordic_ridges' ||
          v === 'isometric_grid' ||
          v === 'sunset_dunes'
        ) {
          c.wallpaperVectorPreset = v;
        }
        break;
      case 'wallpaper_dimming':
        c.wallpaperDimming = clamp(parseInt(v, 10) || 15, 0, 70);
        break;
      case 'font_family':
        if (
          v === 'jakarta' ||
          v === 'dm_sans' ||
          v === 'system_segoe' ||
          v === 'jetbrains_mono'
        ) {
          c.fontFamily = v;
        }
        break;
      case 'font_scale':
        c.fontScale = clamp(parseInt(v, 10) || 100, 90, 115);
        break;
      case 'clock':
        c.appletEnabled.clock = v === 'true';
        break;
      case 'network':
        c.appletEnabled.network = v === 'true';
        break;
      case 'volume':
        c.appletEnabled.volume = v === 'true';
        break;
      case 'battery':
        c.appletEnabled.battery = v === 'true';
        break;
      case 'notifications':
        c.appletEnabled.notifications = v === 'true';
        break;
      case 'bluetooth':
        c.appletEnabled.bluetooth = v === 'true';
        break;
      case 'media':
        c.appletEnabled.media = v === 'true';
        break;
      case 'position':
        c.position =
          v === 'left' ? 'left' : v === 'right' ? 'right' : 'center';
        break;
      case 'dock_style':
        c.dockStyle =
          v === 'panel'
            ? 'panel'
            : v === 'island'
              ? 'island'
              : v === 'transparent'
                ? 'transparent'
                : 'pill';
        break;
      case 'adaptive_dock_transparency':
        c.adaptiveDockTransparency = v === 'true';
        break;
      case 'panel_style':
        c.panelStyle =
          v === 'full'
            ? 'full'
            : v === 'island'
              ? 'island'
              : 'transparent';
        break;
      case 'auto_hide':
        c.autoHide = v === 'true';
        break;
      case 'icon_size':
        c.iconSize = clamp(parseInt(v, 10) || 42, 24, 64);
        break;
      case 'magnification':
        c.magnification = clamp(parseInt(v, 10) || 0, 0, 100);
        break;
      case 'transparency':
        c.transparency = clamp(parseInt(v, 10) || 148, 40, 230);
        break;
      case 'spacing':
        c.spacing = clamp(parseInt(v, 10) || 56, 36, 96);
        break;
      case 'corner_radius':
        c.cornerRadius = clamp(parseInt(v, 10) || 18, 8, 28);
        break;
      case 'offset_x':
        c.offsetX = clamp(parseInt(v, 10) || 0, -200, 200);
        break;
      case 'offset_y':
        c.offsetY = clamp(parseInt(v, 10) || 0, -100, 100);
        break;
      case 'clock_24h':
        c.clock24h = v === 'true';
        break;
      case 'date_format':
        c.dateFormat =
          v === 'none' ? 'none' : v === 'long' ? 'long' : 'short';
        break;
      case 'order':
        c.appletOrder = parseAppletOrder(v);
        break;
      case 'zones':
        c.panelZones = parsePanelZones(v);
        break;
      case 'dock_order':
        c.dockOrder = parseDockOrder(v);
        break;
      case 'gpu_backend':
        c.gpuBackend =
          v === 'direct2d'
            ? 'direct2d'
            : v === 'software'
              ? 'software'
              : 'wgpu';
        break;
      case 'gpu_shader':
        if (
          v === 'orbital_earth' ||
          v === 'cosmic_nebula' ||
          v === 'system76_obsidian' ||
          v === 'solar_flare' ||
          v === 'teal_aurora'
        ) {
          c.gpuShader = v;
        }
        break;
      case 'gpu_blur_radius':
        c.gpuBlurRadius = clamp(parseInt(v, 10) || 24, 0, 40);
        break;
      case 'gpu_bloom':
        c.gpuBloomStrength = clamp(parseInt(v, 10) || 45, 0, 100);
        break;
      case 'gpu_specular':
        c.gpuSpecularBorder = v === 'true';
        break;
      case 'tiling_enabled':
        c.tilingEnabled = v === 'true';
        break;
      case 'tiling_direction':
        c.tilingDirection = v === 'vertical' ? 'vertical' : 'horizontal';
        break;
      case 'tiling_inner_gap': {
        const parsed = parseInt(v, 10);
        c.tilingInnerGap = clamp(Number.isNaN(parsed) ? 10 : parsed, 0, 24);
        break;
      }
      case 'tiling_outer_gap': {
        const parsed = parseInt(v, 10);
        c.tilingOuterGap = clamp(Number.isNaN(parsed) ? 12 : parsed, 0, 24);
        break;
      }
      case 'tiling_active_hint':
        c.tilingActiveHint = v === 'true';
        break;
      case 'tiling_animation_ms': {
        const parsed = parseInt(v, 10);
        c.tilingAnimationMs = clamp(Number.isNaN(parsed) ? 240 : parsed, 0, 500);
        break;
      }
    }
  }
  return c;
}
