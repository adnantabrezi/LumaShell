import React, { useEffect, useRef, useState } from 'react';
import {
  ArrowLeftRight,
  Code2,
  Columns2,
  FolderOpen,
  Globe,
  Layers,
  LayoutGrid,
  Minus,
  Plus,
  StickyNote,
  Terminal,
  X,
} from 'lucide-react';
import { hexToRgbTuple, LumaConfig } from '../utils/lumaConfig';
import { ManagedWindow, WindowRect } from '../utils/tilingEngine';

interface CosmicWindowLayerProps {
  config: LumaConfig;
  windows: ManagedWindow[];
  tiledRects: Record<string, WindowRect>;
  workArea: WindowRect;
  focusedWindowId: string | null;
  onFocusWindow: (id: string) => void;
  onCloseWindow: (id: string) => void;
  onToggleWindowMode: (id: string) => void;
  onFlipSplitDirection: (id: string) => void;
  onSwapWindow: (id: string, dir: 'prev' | 'next') => void;
  onResizeTiledWindow: (id: string, delta: number) => void;
  onUpdateFloatingRect: (id: string, rect: WindowRect) => void;
}

interface DragSession {
  windowId: string;
  kind: 'move' | 'resize';
  startPointerX: number;
  startPointerY: number;
  startRect: WindowRect;
}

const WindowInterior: React.FC<{
  win: ManagedWindow;
  config: LumaConfig;
  isEffectivelyTiled: boolean;
}> = ({ win, config, isEffectivelyTiled }) => {
  switch (win.appKind) {
    case 'vscode':
      return (
        <div className="flex-1 flex flex-col min-h-0 bg-[#0d1117]/95 text-slate-200 font-mono text-xs overflow-hidden">
          <div className="flex items-center justify-between px-3 py-1.5 border-b border-white/10 bg-white/[0.03] text-[11px] text-slate-400">
            <div className="flex items-center gap-3">
              <span
                style={{ borderBottomColor: config.accentColor }}
                className="text-slate-100 font-semibold border-b-2 pb-0.5"
              >
                src/shell/tiling.rs
              </span>
              <span>src/shell/panel.rs</span>
              <span>src/shell/config.rs</span>
            </div>
            <span className="text-[10px] uppercase tracking-wider text-slate-500">
              Rust · GlazeWM SplitContainer
            </span>
          </div>
          <div className="flex-1 p-3.5 overflow-auto leading-relaxed space-y-1">
            <div>
              <span className="text-slate-500 select-none mr-3">01</span>
              <span className="text-sky-400">pub(crate) enum</span>{' '}
              <span className="text-amber-300">TilingDirection</span> {'{ '}
              <span className="text-emerald-300">Horizontal</span>,{' '}
              <span className="text-emerald-300">Vertical</span> {'}'}
            </div>
            <div>
              <span className="text-slate-500 select-none mr-3">02</span>
              <span className="text-sky-400">pub(crate) enum</span>{' '}
              <span className="text-amber-300">TilingNode</span> {'{'}
            </div>
            <div>
              <span className="text-slate-500 select-none mr-3">03</span>
              {'  '}
              <span className="text-emerald-300">Window</span> {'{ '}
              hwnd: <span className="text-amber-300">isize</span>, size:{' '}
              <span className="text-amber-300">f32</span> {'},'}
            </div>
            <div>
              <span className="text-slate-500 select-none mr-3">04</span>
              {'  '}
              <span className="text-emerald-300">Split</span> {'{ '}
              direction: <span className="text-amber-300">TilingDirection</span>
              , size: <span className="text-amber-300">f32</span>, children:{' '}
              <span className="text-amber-300">Vec</span>&lt;TilingNode&gt;{' '}
              {'},'}
            </div>
            <div>
              <span className="text-slate-500 select-none mr-3">05</span>
              {'}'}
            </div>
            <div className="pt-1">
              <span className="text-slate-500 select-none mr-3">06</span>
              <span className="text-slate-500">
                // GlazeWM-style DWMWA_EXTENDED_FRAME_BOUNDS border compensation
              </span>
            </div>
            <div>
              <span className="text-slate-500 select-none mr-3">07</span>
              <span className="text-sky-400">let</span> hdwp ={' '}
              <span className="text-purple-300">BeginDeferWindowPos</span>
              (targets.len() <span className="text-sky-400">as</span> i32)?;
            </div>
            <div>
              <span className="text-slate-500 select-none mr-3">08</span>
              <span className="text-sky-400">for</span> (hwnd, rect){' '}
              <span className="text-sky-400">in</span> interpolated_rects {'{'}
            </div>
            <div>
              <span className="text-slate-500 select-none mr-3">09</span>
              {'  '}
              <span className="text-purple-300">DeferWindowPos</span>(hdwp,
              hwnd, rect.with_dwm_border_delta());
            </div>
            <div>
              <span className="text-slate-500 select-none mr-3">10</span>
              {'}'}
            </div>
            <div>
              <span className="text-slate-500 select-none mr-3">11</span>
              <span className="text-purple-300">EndDeferWindowPos</span>(hdwp)?;
            </div>
          </div>
          <div className="px-3 py-1 border-t border-white/10 bg-black/35 flex items-center justify-between text-[11px] text-slate-400">
            <span style={{ color: config.accentColor }}>
              {isEffectivelyTiled ? 'TILE · GlazeWM Tree' : 'FLOAT · Freeform'}
            </span>
            <span>
              Gap {config.tilingInnerGap}px / {config.tilingOuterGap}px ·{' '}
              {config.tilingAnimationMs}ms ease-out
            </span>
          </div>
        </div>
      );

    case 'terminal':
      return (
        <div className="flex-1 flex flex-col min-h-0 bg-[#080b12]/95 text-slate-200 font-mono text-xs p-3.5 overflow-auto space-y-1.5">
          <div className="text-emerald-400">
            PS C:\Users\Luma\rust_shell&gt; cargo build --release --target
            x86_64-pc-windows-msvc
          </div>
          <div className="text-slate-400">
            Compiling rust_shell v0.1.0 (src/shell/tiling.rs, src/shell/panel.rs)
          </div>
          <div style={{ color: config.accentColor }}>
            [tiling] SetWinEventHook active · BeginDeferWindowPos atomic layout
          </div>
          <div className="text-slate-300">
            Mode:{' '}
            <span className="font-semibold text-white">
              {config.tilingEnabled ? 'COSMIC Auto-Tiling' : 'Floating Desktop'}
            </span>{' '}
            · Split: {config.tilingDirection} · Active Hint:{' '}
            {config.tilingActiveHint ? '2px Accent' : 'Off'}
          </div>
          <div className="pt-2 border-t border-white/10 text-[11px] text-slate-400 space-y-1">
            <div>
              <span className="text-slate-200 font-semibold">Alt+Shift+T</span>{' '}
              Toggle Tiling / Floating ·{' '}
              <span className="text-slate-200 font-semibold">Alt+Space</span>{' '}
              Float Active
            </div>
            <div>
              <span className="text-slate-200 font-semibold">Alt+V</span> Flip
              Split Direction ·{' '}
              <span className="text-slate-200 font-semibold">Alt+H / L</span>{' '}
              Swap Tile Order
            </div>
          </div>
        </div>
      );

    case 'files':
      return (
        <div className="flex-1 flex flex-col min-h-0 bg-[#101521]/95 text-slate-200 text-xs overflow-auto p-3.5 space-y-2.5">
          <div className="flex items-center justify-between text-[11px] text-slate-400 font-mono">
            <span>%APPDATA%\LumaShell\</span>
            <span>6 items · Synced</span>
          </div>
          <div className="grid grid-cols-2 gap-2">
            {[
              { name: 'tiling.rs', meta: 'GlazeWM SplitContainer' },
              { name: 'settings.ini', meta: `gap=${config.tilingInnerGap}px` },
              { name: 'panel.rs', meta: '3-Zone Top Panel' },
              { name: 'desktop.rs', meta: 'Orbital Earth DIB' },
              { name: 'luma_iced.rs', meta: 'WGPU Compositor' },
              { name: 'settings.ini.bak', meta: 'Atomic Backup' },
            ].map((entry) => (
              <div
                key={entry.name}
                className="p-2.5 rounded-xl bg-white/[0.04] border border-white/10 hover:bg-white/[0.08] transition-colors"
              >
                <div className="flex items-center gap-2 font-mono font-medium text-slate-100 truncate">
                  <FolderOpen
                    style={{ color: config.accentColor }}
                    className="w-3.5 h-3.5 shrink-0"
                  />
                  <span className="truncate">{entry.name}</span>
                </div>
                <div className="text-[11px] text-slate-400 mt-0.5 truncate">
                  {entry.meta}
                </div>
              </div>
            ))}
          </div>
        </div>
      );

    case 'brave':
      return (
        <div className="flex-1 flex flex-col min-h-0 bg-[#0f1420]/95 text-slate-200 text-xs overflow-auto">
          <div className="px-3 py-2 border-b border-white/10 bg-black/30 flex items-center gap-2">
            <div className="flex-1 px-3 py-1 rounded-full bg-white/[0.07] border border-white/10 font-mono text-[11px] text-slate-300 truncate">
              https://github.com/glzr-io/glazewm · Luma COSMIC Tiling Spec
            </div>
          </div>
          <div className="p-4 space-y-2.5">
            <div className="text-sm font-bold text-white">
              GlazeWM × COSMIC Window Management
            </div>
            <p className="text-slate-300 leading-relaxed">
              Combines GlazeWM&apos;s recursive <code className="font-mono text-sky-300">SplitContainer</code>{' '}
              tree, DWM invisible border compensation, and atomic{' '}
              <code className="font-mono text-sky-300">DeferWindowPos</code>{' '}
              placement with System76 COSMIC&apos;s effortless tiling toggle, active
              focus hint, and floating exceptions.
            </p>
            <div className="grid grid-cols-2 gap-2 pt-1">
              <div className="p-2.5 rounded-xl bg-white/[0.04] border border-white/10">
                <div className="font-semibold text-white">Tiling Mode</div>
                <div className="text-slate-400 text-[11px] mt-0.5">
                  Auto-splits space with balanced weights & gaps
                </div>
              </div>
              <div className="p-2.5 rounded-xl bg-white/[0.04] border border-white/10">
                <div className="font-semibold text-white">Floating Mode</div>
                <div className="text-slate-400 text-[11px] mt-0.5">
                  Drag titlebar & resize corner freely above tiles
                </div>
              </div>
            </div>
          </div>
        </div>
      );

    case 'notes':
      return (
        <div className="flex-1 flex flex-col min-h-0 bg-[#121622]/95 text-slate-200 text-xs p-3.5 overflow-auto space-y-2">
          <div className="text-xs font-semibold text-white">
            COSMIC Workspace Notes
          </div>
          <p className="text-slate-300 leading-relaxed">
            Use the Top Panel Tiling icon to switch the entire workspace between
            auto-tiling and floating mode, or pop individual windows out with the
            Float button in any titlebar.
          </p>
        </div>
      );
  }
};

function renderWindowIcon(kind: ManagedWindow['appKind'], accentColor: string) {
  switch (kind) {
    case 'vscode':
      return <Code2 style={{ color: accentColor }} className="w-3.5 h-3.5" />;
    case 'terminal':
      return <Terminal className="w-3.5 h-3.5 text-emerald-400" />;
    case 'files':
      return <FolderOpen className="w-3.5 h-3.5 text-amber-400" />;
    case 'brave':
      return <Globe className="w-3.5 h-3.5 text-orange-400" />;
    case 'notes':
      return <StickyNote className="w-3.5 h-3.5 text-purple-400" />;
  }
}

export const CosmicWindowLayer: React.FC<CosmicWindowLayerProps> = ({
  config,
  windows,
  tiledRects,
  workArea,
  focusedWindowId,
  onFocusWindow,
  onCloseWindow,
  onToggleWindowMode,
  onFlipSplitDirection,
  onSwapWindow,
  onResizeTiledWindow,
  onUpdateFloatingRect,
}) => {
  const [dragSession, setDragSession] = useState<DragSession | null>(null);
  const dragRef = useRef<DragSession | null>(null);
  dragRef.current = dragSession;

  useEffect(() => {
    if (!dragSession) return;

    const handlePointerMove = (e: PointerEvent) => {
      const current = dragRef.current;
      if (!current) return;
      const dx = e.clientX - current.startPointerX;
      const dy = e.clientY - current.startPointerY;

      if (current.kind === 'move') {
        const nextX = Math.max(
          workArea.x + 6,
          Math.min(
            workArea.x + workArea.width - current.startRect.width - 6,
            current.startRect.x + dx
          )
        );
        const nextY = Math.max(
          workArea.y + 4,
          Math.min(
            workArea.y + workArea.height - 64,
            current.startRect.y + dy
          )
        );
        onUpdateFloatingRect(current.windowId, {
          ...current.startRect,
          x: Math.round(nextX),
          y: Math.round(nextY),
        });
      } else {
        const nextW = Math.max(
          340,
          Math.min(
            workArea.x + workArea.width - current.startRect.x - 8,
            current.startRect.width + dx
          )
        );
        const nextH = Math.max(
          220,
          Math.min(
            workArea.y + workArea.height - current.startRect.y - 8,
            current.startRect.height + dy
          )
        );
        onUpdateFloatingRect(current.windowId, {
          ...current.startRect,
          width: Math.round(nextW),
          height: Math.round(nextH),
        });
      }
    };

    const handlePointerUp = () => {
      setDragSession(null);
    };

    window.addEventListener('pointermove', handlePointerMove);
    window.addEventListener('pointerup', handlePointerUp);
    return () => {
      window.removeEventListener('pointermove', handlePointerMove);
      window.removeEventListener('pointerup', handlePointerUp);
    };
  }, [dragSession, onUpdateFloatingRect, workArea]);

  if (windows.length === 0) return null;

  const [accR, accG, accB] = hexToRgbTuple(config.accentColor);
  const transitionDuration = `${config.tilingAnimationMs}ms`;

  return (
    <div
      data-testid="cosmic-window-layer"
      data-tiling-mode={config.tilingEnabled ? 'tiled' : 'floating'}
      className="fixed inset-0 pointer-events-none z-20"
    >
      {windows.map((win) => {
        const isEffectivelyTiled =
          config.tilingEnabled &&
          win.mode === 'tiling' &&
          Boolean(tiledRects[win.id]);
        const rect = isEffectivelyTiled
          ? tiledRects[win.id]
          : win.floatingRect;
        const isFocused = focusedWindowId === win.id;
        const isDraggingThis = dragSession?.windowId === win.id;

        const borderColor =
          isFocused && config.tilingActiveHint
            ? config.accentColor
            : config.dark
              ? 'rgba(255, 255, 255, 0.14)'
              : 'rgba(15, 23, 42, 0.16)';

        const boxShadow =
          isFocused && config.tilingActiveHint
            ? isEffectivelyTiled
              ? `0 0 0 1px rgba(${accR}, ${accG}, ${accB}, 0.45), 0 14px 36px rgba(0, 0, 0, 0.48)`
              : `0 0 0 1px rgba(${accR}, ${accG}, ${accB}, 0.55), 0 24px 56px rgba(0, 0, 0, 0.65)`
            : isEffectivelyTiled
              ? '0 10px 28px rgba(0, 0, 0, 0.38)'
              : '0 20px 48px rgba(0, 0, 0, 0.56)';

        return (
          <section
            key={win.id}
            data-testid={`managed-window-${win.appKind}`}
            data-window-mode={isEffectivelyTiled ? 'tiling' : 'floating'}
            data-focused={isFocused ? 'true' : 'false'}
            aria-label={`${win.title} (${isEffectivelyTiled ? 'Tiled' : 'Floating'})`}
            onPointerDown={() => onFocusWindow(win.id)}
            style={{
              left: `${rect.x}px`,
              top: `${rect.y}px`,
              width: `${rect.width}px`,
              height: `${rect.height}px`,
              zIndex: isEffectivelyTiled ? (isFocused ? 24 : 21) : 30 + win.zIndex,
              borderColor,
              borderWidth: isFocused && config.tilingActiveHint ? '2px' : '1px',
              boxShadow,
              backdropFilter: `blur(${Math.min(20, config.gpuBlurRadius)}px)`,
              transitionProperty: isDraggingThis
                ? 'none'
                : 'left, top, width, height, box-shadow, border-color',
              transitionDuration: isDraggingThis ? '0ms' : transitionDuration,
              transitionTimingFunction: 'cubic-bezier(0.22, 1, 0.36, 1)',
            }}
            className={`fixed pointer-events-auto rounded-2xl overflow-hidden flex flex-col border ${
              config.dark
                ? 'bg-slate-950/88 text-slate-100'
                : 'bg-white/92 text-slate-900'
            }`}
          >
            {/* COSMIC Window Header / Titlebar (draggable in floating mode) */}
            <div
              onPointerDown={(e) => {
                if ((e.target as HTMLElement).closest('button')) return;
                onFocusWindow(win.id);
                if (!isEffectivelyTiled) {
                  e.preventDefault();
                  setDragSession({
                    windowId: win.id,
                    kind: 'move',
                    startPointerX: e.clientX,
                    startPointerY: e.clientY,
                    startRect: { ...rect },
                  });
                }
              }}
              className={`flex items-center justify-between gap-2 px-3 py-2 border-b shrink-0 ${
                config.dark
                  ? 'border-white/10 bg-white/[0.04]'
                  : 'border-slate-200 bg-slate-100/80'
              } ${!isEffectivelyTiled ? 'cursor-grab active:cursor-grabbing' : ''}`}
            >
              <div className="flex items-center gap-2 min-w-0">
                <span className="shrink-0">
                  {renderWindowIcon(win.appKind, config.accentColor)}
                </span>
                <span className="text-xs font-semibold truncate">
                  {win.title}
                </span>
                <span
                  style={
                    isEffectivelyTiled
                      ? {
                          color: config.accentColor,
                          backgroundColor: `rgba(${accR}, ${accG}, ${accB}, 0.14)`,
                        }
                      : undefined
                  }
                  className={`px-1.5 py-0.5 rounded text-[10px] font-semibold uppercase tracking-wide shrink-0 ${
                    isEffectivelyTiled
                      ? ''
                      : 'bg-amber-400/15 text-amber-300'
                  }`}
                >
                  {isEffectivelyTiled ? 'Tiled' : 'Floating'}
                </span>
              </div>

              <div className="flex items-center gap-0.5 shrink-0">
                {isEffectivelyTiled && (
                  <>
                    <button
                      type="button"
                      onClick={() => onResizeTiledWindow(win.id, -0.22)}
                      aria-label={`Shrink ${win.title} tile`}
                      title="Shrink tile split"
                      className="p-1 rounded-md text-slate-400 hover:text-white hover:bg-white/10 transition-colors"
                    >
                      <Minus className="w-3 h-3" />
                    </button>
                    <button
                      type="button"
                      onClick={() => onResizeTiledWindow(win.id, 0.22)}
                      aria-label={`Grow ${win.title} tile`}
                      title="Grow tile split"
                      className="p-1 rounded-md text-slate-400 hover:text-white hover:bg-white/10 transition-colors"
                    >
                      <Plus className="w-3 h-3" />
                    </button>
                    <button
                      type="button"
                      onClick={() => onFlipSplitDirection(win.id)}
                      aria-label={`Flip split direction for ${win.title}`}
                      title="Flip Split Direction (Alt+V)"
                      className="p-1 rounded-md text-slate-400 hover:text-white hover:bg-white/10 transition-colors"
                    >
                      <Columns2 className="w-3 h-3" />
                    </button>
                    <button
                      type="button"
                      onClick={() => onSwapWindow(win.id, 'next')}
                      aria-label={`Swap ${win.title} with next tile`}
                      title="Swap with next tile (Alt+L)"
                      className="p-1 rounded-md text-slate-400 hover:text-white hover:bg-white/10 transition-colors"
                    >
                      <ArrowLeftRight className="w-3 h-3" />
                    </button>
                  </>
                )}

                <button
                  type="button"
                  onClick={() => onToggleWindowMode(win.id)}
                  aria-label={
                    isEffectivelyTiled
                      ? `Float ${win.title}`
                      : `Tile ${win.title}`
                  }
                  title={
                    isEffectivelyTiled
                      ? 'Pop out to Floating Mode (Alt+Space)'
                      : 'Snap back into Tiling Tree (Alt+Space)'
                  }
                  style={
                    !isEffectivelyTiled
                      ? { color: config.accentColor }
                      : undefined
                  }
                  className="p-1 rounded-md text-slate-300 hover:text-white hover:bg-white/10 transition-colors"
                >
                  {isEffectivelyTiled ? (
                    <Layers className="w-3.5 h-3.5" />
                  ) : (
                    <LayoutGrid className="w-3.5 h-3.5" />
                  )}
                </button>

                <button
                  type="button"
                  onClick={() => onCloseWindow(win.id)}
                  aria-label={`Close ${win.title}`}
                  title="Close Window"
                  className="p-1 rounded-md text-slate-400 hover:text-rose-300 hover:bg-rose-500/20 transition-colors"
                >
                  <X className="w-3.5 h-3.5" />
                </button>
              </div>
            </div>

            {/* Application Content Surface */}
            <WindowInterior
              win={win}
              config={config}
              isEffectivelyTiled={isEffectivelyTiled}
            />

            {/* Bottom-right resize grip when floating */}
            {!isEffectivelyTiled && (
              <div
                aria-hidden="true"
                onPointerDown={(e) => {
                  e.stopPropagation();
                  e.preventDefault();
                  onFocusWindow(win.id);
                  setDragSession({
                    windowId: win.id,
                    kind: 'resize',
                    startPointerX: e.clientX,
                    startPointerY: e.clientY,
                    startRect: { ...rect },
                  });
                }}
                className="absolute right-0 bottom-0 w-4 h-4 cursor-nwse-resize flex items-end justify-end p-1"
              >
                <span className="w-2 h-2 border-r-2 border-b-2 border-white/35 rounded-br-[2px]" />
              </div>
            )}
          </section>
        );
      })}
    </div>
  );
};
