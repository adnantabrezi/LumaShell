import { clamp, TilingDirection } from './lumaConfig';

export interface WindowRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export type WindowMode = 'tiling' | 'floating';

export type ManagedAppKind = 'vscode' | 'terminal' | 'files' | 'brave' | 'notes';

export interface ManagedWindow {
  id: string;
  appKind: ManagedAppKind;
  title: string;
  subtitle: string;
  workspace: number;
  mode: WindowMode;
  floatingRect: WindowRect;
  zIndex: number;
}

/**
 * GlazeWM v3-inspired recursive SplitContainer tree node.
 * Either a Leaf window node (referencing ManagedWindow.id) or a SplitContainer
 * whose children divide available space proportionally by `size` weights.
 */
export type TilingNode = TilingWindowLeaf | TilingSplitContainer;

export interface TilingWindowLeaf {
  kind: 'window';
  windowId: string;
  size: number; // proportional weight within parent (normalized positive float)
}

export interface TilingSplitContainer {
  kind: 'split';
  id: string;
  direction: TilingDirection;
  size: number;
  children: TilingNode[];
}

export interface TilingLayoutOptions {
  innerGap: number;
  outerGap: number;
}

let splitIdCounter = 1;

function nextSplitId(): string {
  splitIdCounter += 1;
  return `split-${splitIdCounter}`;
}

export function createLeaf(windowId: string, size = 1): TilingWindowLeaf {
  return { kind: 'window', windowId, size };
}

export function createSplit(
  direction: TilingDirection,
  children: TilingNode[],
  size = 1
): TilingSplitContainer {
  return {
    kind: 'split',
    id: nextSplitId(),
    direction,
    size,
    children,
  };
}

/**
 * Collects all window IDs currently present in a TilingNode subtree in visual order.
 */
export function collectWindowIds(node: TilingNode | null): string[] {
  if (!node) return [];
  if (node.kind === 'window') return [node.windowId];
  return node.children.flatMap(collectWindowIds);
}

/**
 * Checks whether a windowId exists anywhere in the tiling tree.
 */
export function treeContainsWindow(
  node: TilingNode | null,
  windowId: string
): boolean {
  if (!node) return false;
  if (node.kind === 'window') return node.windowId === windowId;
  return node.children.some((c) => treeContainsWindow(c, windowId));
}

/**
 * Normalizes sibling weights so they sum cleanly to children.length (average 1.0)
 * while preserving relative proportions and preventing zero/negative collapse.
 */
function normalizeSiblingWeights(children: TilingNode[]): TilingNode[] {
  if (children.length === 0) return children;
  const total = children.reduce((acc, c) => acc + Math.max(0.1, c.size), 0);
  if (total <= 0) {
    return children.map((c) => ({ ...c, size: 1 }));
  }
  const scale = children.length / total;
  return children.map((c) => ({
    ...c,
    size: Math.max(0.15, c.size * scale),
  }));
}

/**
 * GlazeWM-style insertion:
 * - If tree is empty -> single Leaf.
 * - If focusedWindowId is in a SplitContainer with the SAME direction as `direction`,
 *   insert the new leaf directly beside it (flat sibling, just like GlazeWM's sibling append).
 * - If focusedWindowId is in a SplitContainer with a DIFFERENT direction (or root is a leaf),
 *   wrap the focused leaf + new leaf in a new nested SplitContainer with `direction`.
 *   This produces classic COSMIC/GlazeWM master-stack and spiral layouts naturally!
 */
export function insertWindowIntoTree(
  root: TilingNode | null,
  newWindowId: string,
  focusedWindowId: string | null,
  direction: TilingDirection
): TilingNode {
  if (root && treeContainsWindow(root, newWindowId)) {
    return root;
  }
  const newLeaf = createLeaf(newWindowId, 1);
  if (!root) {
    return newLeaf;
  }
  if (root.kind === 'window') {
    return createSplit(direction, [{ ...root, size: 1 }, newLeaf], 1);
  }

  const targetId =
    focusedWindowId && treeContainsWindow(root, focusedWindowId)
      ? focusedWindowId
      : collectWindowIds(root).at(-1) ?? null;

  if (!targetId) {
    return {
      ...root,
      children: normalizeSiblingWeights([...root.children, newLeaf]),
    };
  }

  const insertRecursive = (node: TilingNode): [TilingNode, boolean] => {
    if (node.kind === 'window') {
      if (node.windowId === targetId) {
        return [
          createSplit(direction, [{ ...node, size: 1 }, newLeaf], node.size),
          true,
        ];
      }
      return [node, false];
    }

    const directIdx = node.children.findIndex(
      (c) => c.kind === 'window' && c.windowId === targetId
    );
    if (directIdx !== -1) {
      if (node.direction === direction) {
        const nextChildren = [...node.children];
        nextChildren.splice(directIdx + 1, 0, newLeaf);
        return [
          { ...node, children: normalizeSiblingWeights(nextChildren) },
          true,
        ];
      }
      // Different split direction -> wrap focused leaf into a nested SplitContainer
      const targetLeaf = node.children[directIdx] as TilingWindowLeaf;
      const wrapped = createSplit(
        direction,
        [{ ...targetLeaf, size: 1 }, newLeaf],
        targetLeaf.size
      );
      const nextChildren = [...node.children];
      nextChildren[directIdx] = wrapped;
      return [{ ...node, children: nextChildren }, true];
    }

    let inserted = false;
    const nextChildren = node.children.map((child) => {
      if (inserted) return child;
      const [updated, didInsert] = insertRecursive(child);
      if (didInsert) inserted = true;
      return updated;
    });
    return [inserted ? { ...node, children: nextChildren } : node, inserted];
  };

  const [updated] = insertRecursive(root);
  return updated;
}

/**
 * Removes a window leaf from the GlazeWM tree and automatically flattens
 * single-child SplitContainers (just like GlazeWM's flatten_split_container).
 */
export function removeWindowFromTree(
  root: TilingNode | null,
  windowId: string
): TilingNode | null {
  if (!root) return null;
  if (root.kind === 'window') {
    return root.windowId === windowId ? null : root;
  }

  const nextChildren: TilingNode[] = [];
  for (const child of root.children) {
    const cleaned = removeWindowFromTree(child, windowId);
    if (cleaned) {
      nextChildren.push(cleaned);
    }
  }

  if (nextChildren.length === 0) return null;
  if (nextChildren.length === 1) {
    // Flatten single-child container, preserving outer weight
    return { ...nextChildren[0], size: root.size };
  }
  return {
    ...root,
    children: normalizeSiblingWeights(nextChildren),
  };
}

/**
 * Computes exact pixel rectangles for every leaf window in the GlazeWM SplitContainer tree
 * within `workArea`, applying `outerGap` around the workspace edges and `innerGap` between tiles.
 */
export function computeTiledLayout(
  root: TilingNode | null,
  workArea: WindowRect,
  options: TilingLayoutOptions
): Record<string, WindowRect> {
  const result: Record<string, WindowRect> = {};
  if (!root) return result;

  const outerGap = clamp(Math.round(options.outerGap), 0, 48);
  const innerGap = clamp(Math.round(options.innerGap), 0, 48);

  const usable: WindowRect = {
    x: Math.round(workArea.x + outerGap),
    y: Math.round(workArea.y + outerGap),
    width: Math.max(120, Math.round(workArea.width - outerGap * 2)),
    height: Math.max(120, Math.round(workArea.height - outerGap * 2)),
  };

  const layoutNode = (node: TilingNode, bounds: WindowRect) => {
    if (node.kind === 'window') {
      result[node.windowId] = {
        x: Math.round(bounds.x),
        y: Math.round(bounds.y),
        width: Math.max(120, Math.round(bounds.width)),
        height: Math.max(100, Math.round(bounds.height)),
      };
      return;
    }

    const count = node.children.length;
    if (count === 0) return;

    const totalWeight = node.children.reduce(
      (sum, c) => sum + Math.max(0.1, c.size),
      0
    );
    const totalGap = innerGap * Math.max(0, count - 1);

    if (node.direction === 'horizontal') {
      const availW = Math.max(count * 80, bounds.width - totalGap);
      let cursorX = bounds.x;
      node.children.forEach((child, idx) => {
        const isLast = idx === count - 1;
        const share = Math.max(0.1, child.size) / totalWeight;
        const childW = isLast
          ? Math.max(80, bounds.x + bounds.width - cursorX)
          : Math.round(availW * share);
        layoutNode(child, {
          x: cursorX,
          y: bounds.y,
          width: childW,
          height: bounds.height,
        });
        cursorX += childW + innerGap;
      });
    } else {
      const availH = Math.max(count * 80, bounds.height - totalGap);
      let cursorY = bounds.y;
      node.children.forEach((child, idx) => {
        const isLast = idx === count - 1;
        const share = Math.max(0.1, child.size) / totalWeight;
        const childH = isLast
          ? Math.max(80, bounds.y + bounds.height - cursorY)
          : Math.round(availH * share);
        layoutNode(child, {
          x: bounds.x,
          y: cursorY,
          width: bounds.width,
          height: childH,
        });
        cursorY += childH + innerGap;
      });
    }
  };

  layoutNode(root, usable);
  return result;
}

/**
 * Swaps two window IDs in-place inside the SplitContainer tree while preserving split geometry.
 */
export function swapWindowsInTree(
  root: TilingNode | null,
  idA: string,
  idB: string
): TilingNode | null {
  if (!root || idA === idB) return root;
  const swapRec = (node: TilingNode): TilingNode => {
    if (node.kind === 'window') {
      if (node.windowId === idA) return { ...node, windowId: idB };
      if (node.windowId === idB) return { ...node, windowId: idA };
      return node;
    }
    return { ...node, children: node.children.map(swapRec) };
  };
  return swapRec(root);
}

/**
 * Moves a window one slot backward ('prev') or forward ('next') in visual tree order
 * by swapping it with its neighbor (simple, predictable COSMIC-style keyboard reordering).
 */
export function swapWindowWithNeighbor(
  root: TilingNode | null,
  windowId: string,
  direction: 'prev' | 'next'
): TilingNode | null {
  if (!root) return null;
  const ordered = collectWindowIds(root);
  const idx = ordered.indexOf(windowId);
  if (idx === -1 || ordered.length < 2) return root;
  const neighborIdx =
    direction === 'prev'
      ? (idx - 1 + ordered.length) % ordered.length
      : (idx + 1) % ordered.length;
  return swapWindowsInTree(root, windowId, ordered[neighborIdx]);
}

/**
 * Toggles the split direction (horizontal <-> vertical) of the SplitContainer
 * that directly parents `windowId` (or the root container if not found).
 */
export function toggleSplitDirectionAt(
  root: TilingNode | null,
  windowId: string | null
): TilingNode | null {
  if (!root || root.kind === 'window') return root;

  const flip = (dir: TilingDirection): TilingDirection =>
    dir === 'horizontal' ? 'vertical' : 'horizontal';

  if (!windowId) {
    return { ...root, direction: flip(root.direction) };
  }

  let flipped = false;
  const walk = (node: TilingNode): TilingNode => {
    if (node.kind === 'window') return node;
    const hasDirectChild = node.children.some(
      (c) => c.kind === 'window' && c.windowId === windowId
    );
    if (hasDirectChild) {
      flipped = true;
      return { ...node, direction: flip(node.direction) };
    }
    return { ...node, children: node.children.map(walk) };
  };

  const next = walk(root);
  if (!flipped && next.kind === 'split') {
    return { ...next, direction: flip(next.direction) };
  }
  return next;
}

/**
 * Resizes the proportional weight of `windowId` within its parent SplitContainer
 * by `delta` (e.g. +0.2 or -0.2), safely clamping between 0.35 and 2.5 so no tile collapses.
 */
export function resizeWindowInTree(
  root: TilingNode | null,
  windowId: string,
  delta: number
): TilingNode | null {
  if (!root || root.kind === 'window') return root;

  const walk = (node: TilingNode): TilingNode => {
    if (node.kind === 'window') return node;
    const directIdx = node.children.findIndex(
      (c) => c.kind === 'window' && c.windowId === windowId
    );
    if (directIdx !== -1 && node.children.length >= 2) {
      const target = node.children[directIdx];
      const nextSize = clamp(target.size + delta, 0.35, 2.5);
      const actualDelta = nextSize - target.size;
      if (Math.abs(actualDelta) < 0.001) return node;

      const othersCount = node.children.length - 1;
      const otherDelta = -actualDelta / othersCount;
      const nextChildren = node.children.map((child, idx) => {
        if (idx === directIdx) {
          return { ...child, size: nextSize };
        }
        return { ...child, size: clamp(child.size + otherDelta, 0.35, 2.5) };
      });
      return { ...node, children: normalizeSiblingWeights(nextChildren) };
    }
    return { ...node, children: node.children.map(walk) };
  };

  return walk(root);
}

/**
 * Builds the classic COSMIC 3-pane workspace tree:
 * Left 55% primary pane (e.g. VS Code), Right 45% stacked vertically (Terminal top, Files bottom).
 */
export function createThreePanePresetTree(
  primaryId: string,
  topRightId: string,
  bottomRightId: string
): TilingNode {
  return createSplit(
    'horizontal',
    [
      createLeaf(primaryId, 1.12),
      createSplit(
        'vertical',
        [createLeaf(topRightId, 1.0), createLeaf(bottomRightId, 1.0)],
        0.88
      ),
    ],
    1
  );
}

/**
 * Smooth cubic-out easing matching native Rust `tiling.rs` animations.
 */
export function easeOutCubic(t: number): number {
  const clamped = clamp(t, 0, 1);
  return 1 - Math.pow(1 - clamped, 3);
}

/**
 * Interpolates two WindowRects using eased progress `t` in [0, 1].
 */
export function lerpRect(from: WindowRect, to: WindowRect, t: number): WindowRect {
  const k = easeOutCubic(t);
  return {
    x: Math.round(from.x + (to.x - from.x) * k),
    y: Math.round(from.y + (to.y - from.y) * k),
    width: Math.round(from.width + (to.width - from.width) * k),
    height: Math.round(from.height + (to.height - from.height) * k),
  };
}

/**
 * Computes a clean staggered floating rectangle inside `workArea` for newly floated windows.
 */
export function staggeredFloatingRect(
  workArea: WindowRect,
  index: number
): WindowRect {
  const width = Math.min(720, Math.max(420, Math.round(workArea.width * 0.52)));
  const height = Math.min(480, Math.max(300, Math.round(workArea.height * 0.62)));
  const stepX = 46;
  const stepY = 34;
  const offsetIndex = index % 5;
  const baseX = Math.round(workArea.x + (workArea.width - width) * 0.36);
  const baseY = Math.round(workArea.y + (workArea.height - height) * 0.32);
  const x = clamp(
    baseX + offsetIndex * stepX,
    workArea.x + 16,
    Math.max(workArea.x + 16, workArea.x + workArea.width - width - 16)
  );
  const y = clamp(
    baseY + offsetIndex * stepY,
    workArea.y + 12,
    Math.max(workArea.y + 12, workArea.y + workArea.height - height - 12)
  );
  return { x, y, width, height };
}
