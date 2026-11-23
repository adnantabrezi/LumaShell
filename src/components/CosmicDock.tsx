import React, { useRef, useState } from 'react';
import {
  calculateDockIconSize,
  DockLauncherId,
  hexToRgbTuple,
  LumaConfig,
} from '../utils/lumaConfig';
import { AppBrandIcon, AppIconId } from './AppBrandIcons';

export interface DockAppItem {
  id: string;
  label: string;
  icon: AppIconId;
  running: boolean;
  active: boolean;
  launcherId?: DockLauncherId;
}

interface CosmicDockProps {
  config: LumaConfig;
  hasAppBehind: boolean;
  pinnedApps: DockAppItem[];
  runningApps: DockAppItem[];
  onActivateApp: (app: DockAppItem) => void;
  onReorderPinned: (fromIndex: number, toIndex: number) => void;
}

export const CosmicDock: React.FC<CosmicDockProps> = ({
  config,
  hasAppBehind,
  pinnedApps,
  runningApps,
  onActivateApp,
  onReorderPinned,
}) => {
  const dockRef = useRef<HTMLDivElement | null>(null);
  const [pointerX, setPointerX] = useState<number | null>(null);
  const [hoveredId, setHoveredId] = useState<string | null>(null);
  const [draggedIdx, setDraggedIdx] = useState<number | null>(null);
  const [isDockHovered, setIsDockHovered] = useState<boolean>(false);

  const isPill = config.dockStyle === 'pill';
  const isPanelMode = config.dockStyle === 'panel';
  const isTransparent = config.dockStyle === 'transparent';
  // Transparent on unobstructed desktop; transitions to translucent frosted glass when an app is open behind or hovered
  const showGlassBackdrop =
    !isTransparent &&
    (!config.adaptiveDockTransparency || hasAppBehind || isDockHovered);
  const alphaFloat = Math.max(0.22, Math.min(0.68, config.transparency / 255));
  const [accR, accG, accB] = hexToRgbTuple(config.accentColor);

  const alignmentClass =
    config.position === 'left'
      ? 'justify-start pl-6'
      : config.position === 'right'
        ? 'justify-end pr-6'
        : 'justify-center';

  const handleMouseMove = (e: React.MouseEvent<HTMLDivElement>) => {
    if (!dockRef.current) return;
    const rect = dockRef.current.getBoundingClientRect();
    setPointerX(e.clientX - rect.left);
  };

  const handleMouseLeave = () => {
    setPointerX(null);
    setHoveredId(null);
    setIsDockHovered(false);
  };

  const isCollapsed = config.autoHide && !isDockHovered;

  const visiblePinned = config.showPinnedApps ? pinnedApps : [];
  const slotWidth = Math.max(48, config.spacing);

  const renderDockSlot = (
    app: DockAppItem,
    globalIndex: number,
    isPinnedDraggable: boolean,
    pinnedIndex: number
  ) => {
    const slotCenter = 16 + globalIndex * slotWidth + slotWidth / 2;
    const computedSize = calculateDockIconSize(
      config.iconSize,
      config.magnification,
      pointerX,
      slotCenter,
      1.0
    );
    const isHovered = hoveredId === app.id;
    const showCircularHalo = showGlassBackdrop
      ? app.active || isHovered
      : isHovered;
    const haloSize = Math.round(config.iconSize + 14);

    return (
      <div
        key={app.id}
        draggable={isPinnedDraggable}
        onDragStart={() => {
          if (isPinnedDraggable) setDraggedIdx(pinnedIndex);
        }}
        onDragOver={(e) => {
          if (isPinnedDraggable) e.preventDefault();
        }}
        onDrop={() => {
          if (
            isPinnedDraggable &&
            draggedIdx !== null &&
            draggedIdx !== pinnedIndex
          ) {
            onReorderPinned(draggedIdx, pinnedIndex);
          }
          setDraggedIdx(null);
        }}
        style={{ width: `${slotWidth}px` }}
        className="relative flex flex-col items-center justify-center select-none"
      >
        {/* COSMIC Frosted Dark Tooltip Pill directly above hovered icon */}
        {isHovered && (
          <div
            role="tooltip"
            className="absolute -top-11 left-1/2 -translate-x-1/2 px-3 py-1 rounded-xl text-xs font-medium whitespace-nowrap pointer-events-none z-50 bg-[#14151a]/88 text-slate-100 border border-white/15 backdrop-blur-md shadow-[0_8px_24px_rgba(0,0,0,0.55)]"
          >
            {app.label}
          </div>
        )}

        {/* Circular active/hover halo disc matching COSMIC Dock */}
        <button
          type="button"
          onClick={() => onActivateApp(app)}
          onMouseEnter={() => setHoveredId(app.id)}
          onFocus={() => setHoveredId(app.id)}
          onBlur={() => setHoveredId(null)}
          aria-label={`Dock app: ${app.label}${app.running ? ' (Running)' : ''}`}
          style={{
            width: `${haloSize}px`,
            height: `${haloSize}px`,
          }}
          className={`relative flex flex-col items-center justify-center rounded-full transition-all duration-150 ${
            showCircularHalo
              ? config.dark
                ? showGlassBackdrop
                  ? 'bg-white/[0.11] shadow-[inset_0_1px_0_rgba(255,255,255,0.09)]'
                  : 'bg-white/[0.16] backdrop-blur-md shadow-[inset_0_1px_0_rgba(255,255,255,0.14)]'
                : 'bg-slate-900/10'
              : 'bg-transparent hover:bg-white/[0.09]'
          }`}
        >
          <div
            style={{
              width: `${computedSize}px`,
              height: `${computedSize}px`,
            }}
            className="flex items-center justify-center transition-transform duration-100 drop-shadow-[0_4px_10px_rgba(0,0,0,0.48)]"
          >
            <AppBrandIcon id={app.icon} className="w-full h-full" />
          </div>

          {/* Crisp small white circular running indicator dot centered at bottom of icon slot */}
          {config.showRunningIndicators && app.running && (
            <span
              aria-hidden="true"
              style={
                app.active
                  ? { backgroundColor: '#ffffff' }
                  : { backgroundColor: 'rgba(226, 232, 240, 0.88)' }
              }
              className="absolute bottom-1 w-1.5 h-1.5 rounded-full shadow-[0_0_4px_rgba(0,0,0,0.9)]"
            />
          )}
        </button>
      </div>
    );
  };

  const effectiveBlur = Math.max(18, config.gpuBlurRadius);

  return (
    <div
      className={`fixed bottom-0 inset-x-0 z-40 flex items-end pointer-events-none ${alignmentClass}`}
      style={{
        transform: `translate(${config.offsetX}px, ${config.offsetY}px)`,
      }}
    >
      {config.autoHide && (
        <div
          onMouseEnter={() => setIsDockHovered(true)}
          className="fixed bottom-0 inset-x-0 h-3 pointer-events-auto"
        />
      )}

      <nav
        ref={dockRef}
        aria-label="Luma COSMIC Dock"
        data-testid="luma-cosmic-dock"
        data-dock-surface={showGlassBackdrop ? 'translucent' : 'transparent'}
        onMouseEnter={() => setIsDockHovered(true)}
        onMouseMove={handleMouseMove}
        onMouseLeave={handleMouseLeave}
        style={{
          borderRadius: isPanelMode
            ? '0px'
            : isPill
              ? '9999px'
              : `${config.cornerRadius}px`,
          backdropFilter: !showGlassBackdrop
            ? 'none'
            : `blur(${effectiveBlur}px) saturate(1.35)`,
          WebkitBackdropFilter: !showGlassBackdrop
            ? 'none'
            : `blur(${effectiveBlur}px) saturate(1.35)`,
          backgroundColor: !showGlassBackdrop
            ? 'transparent'
            : config.highContrast
              ? 'rgba(0, 0, 0, 0.92)'
              : config.dark
                ? isPill
                  ? `rgba(14, 16, 22, ${alphaFloat.toFixed(2)})`
                  : `rgba(20, 23, 32, ${alphaFloat.toFixed(2)})`
                : `rgba(248, 250, 252, ${alphaFloat.toFixed(2)})`,
          boxShadow: !showGlassBackdrop
            ? 'none'
            : config.gpuSpecularBorder
              ? `inset 0 1px 0 0 ${
                  config.dark
                    ? isPill
                      ? 'rgba(255, 255, 255, 0.14)'
                      : `rgba(${accR}, ${accG}, ${accB}, 0.42)`
                    : 'rgba(255,255,255,0.9)'
                }, 0 14px 36px -8px rgba(0, 0, 0, 0.55)`
              : '0 12px 28px -6px rgba(0, 0, 0, 0.45)',
        }}
        className={`pointer-events-auto flex items-center transition-all duration-200 ${
          isPanelMode
            ? 'w-full justify-center py-2 px-6 mb-0'
            : !showGlassBackdrop
              ? 'px-3 py-1.5 mb-2.5'
              : isPill
                ? 'px-3 py-1.5 mb-3'
                : 'px-4 py-2 mb-3'
        } ${
          !showGlassBackdrop
            ? 'border border-transparent'
            : config.highContrast
              ? 'border-2 border-yellow-400'
              : config.dark
                ? 'border border-white/[0.13]'
                : 'border border-slate-300/80'
        } ${
          isCollapsed
            ? 'translate-y-[82%] opacity-40 hover:translate-y-0 hover:opacity-100'
            : 'translate-y-0 opacity-100'
        }`}
      >
        <div className="flex items-center">
          {/* Pinned / Favorite Applications (Left of vertical divider) */}
          {visiblePinned.map((app, index) =>
            renderDockSlot(app, index, true, index)
          )}

          {/* Subtle Vertical Separator between Pinned Apps and Running Apps */}
          {visiblePinned.length > 0 && runningApps.length > 0 && (
            <div
              aria-hidden="true"
              className={`mx-2 h-8 w-px shrink-0 ${
                config.dark ? 'bg-white/15' : 'bg-slate-400/40'
              }`}
            />
          )}

          {/* Active Running Applications (Right of vertical divider: Terminal, COSMIC Settings, etc.) */}
          {runningApps.map((app, rIdx) =>
            renderDockSlot(app, visiblePinned.length + rIdx, false, -1)
          )}
        </div>
      </nav>
    </div>
  );
};
