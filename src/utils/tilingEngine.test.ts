import test from 'node:test';
import assert from 'node:assert/strict';
import {
  collectWindowIds,
  computeTiledLayout,
  createThreePanePresetTree,
  easeOutCubic,
  insertWindowIntoTree,
  lerpRect,
  removeWindowFromTree,
  resizeWindowInTree,
  staggeredFloatingRect,
  swapWindowWithNeighbor,
  toggleSplitDirectionAt,
  WindowRect,
} from './tilingEngine';

const WORK_AREA: WindowRect = {
  x: 0,
  y: 34,
  width: 1440,
  height: 780,
};

test('insertWindowIntoTree builds horizontal and nested vertical GlazeWM split containers', () => {
  let tree = insertWindowIntoTree(null, 'win-1', null, 'horizontal');
  assert.deepEqual(collectWindowIds(tree), ['win-1']);

  tree = insertWindowIntoTree(tree, 'win-2', 'win-1', 'horizontal');
  assert.deepEqual(collectWindowIds(tree), ['win-1', 'win-2']);
  assert.equal(tree.kind, 'split');
  if (tree.kind === 'split') {
    assert.equal(tree.direction, 'horizontal');
    assert.equal(tree.children.length, 2);
  }

  // Splitting win-2 vertically wraps win-2 + win-3 into a nested vertical SplitContainer
  tree = insertWindowIntoTree(tree, 'win-3', 'win-2', 'vertical');
  assert.deepEqual(collectWindowIds(tree), ['win-1', 'win-2', 'win-3']);
  if (tree.kind === 'split') {
    assert.equal(tree.children[1].kind, 'split');
    if (tree.children[1].kind === 'split') {
      assert.equal(tree.children[1].direction, 'vertical');
    }
  }
});

test('computeTiledLayout produces exact non-overlapping tiles respecting inner and outer gaps', () => {
  const tree = createThreePanePresetTree('vscode', 'terminal', 'files');
  const layout = computeTiledLayout(tree, WORK_AREA, {
    innerGap: 10,
    outerGap: 12,
  });

  const vscode = layout['vscode'];
  const terminal = layout['terminal'];
  const files = layout['files'];

  assert.ok(vscode && terminal && files);
  // Outer gap respected on left, top, right, bottom
  assert.equal(vscode.x, 12);
  assert.equal(vscode.y, 34 + 12);
  assert.equal(terminal.x + terminal.width, 1440 - 12);
  assert.equal(files.y + files.height, 34 + 780 - 12);

  // Inner gap respected horizontally between vscode and right stack
  assert.equal(terminal.x - (vscode.x + vscode.width), 10);
  // Inner gap respected vertically between terminal and files
  assert.equal(files.y - (terminal.y + terminal.height), 10);
  // Left primary pane gets slightly more width (1.12 vs 0.88 weight)
  assert.ok(vscode.width > terminal.width);
});

test('removeWindowFromTree flattens single-child containers cleanly', () => {
  const tree = createThreePanePresetTree('vscode', 'terminal', 'files');
  const afterRemove = removeWindowFromTree(tree, 'files');
  assert.ok(afterRemove);
  assert.deepEqual(collectWindowIds(afterRemove), ['vscode', 'terminal']);

  // The nested vertical split should be flattened directly into a leaf sibling
  assert.equal(afterRemove.kind, 'split');
  if (afterRemove.kind === 'split') {
    assert.equal(afterRemove.children[1].kind, 'window');
    if (afterRemove.children[1].kind === 'window') {
      assert.equal(afterRemove.children[1].windowId, 'terminal');
    }
  }

  // Removing both remaining windows yields null
  const oneLeft = removeWindowFromTree(afterRemove, 'terminal');
  assert.ok(oneLeft && oneLeft.kind === 'window');
  assert.equal(removeWindowFromTree(oneLeft, 'vscode'), null);
});

test('swapWindowWithNeighbor swaps positions without changing layout slots', () => {
  const tree = createThreePanePresetTree('vscode', 'terminal', 'files');
  const swapped = swapWindowWithNeighbor(tree, 'vscode', 'next');
  assert.deepEqual(collectWindowIds(swapped), ['terminal', 'vscode', 'files']);
});

test('toggleSplitDirectionAt flips parent container direction between horizontal and vertical', () => {
  const tree = createThreePanePresetTree('vscode', 'terminal', 'files');
  const flippedRight = toggleSplitDirectionAt(tree, 'terminal');
  assert.ok(flippedRight && flippedRight.kind === 'split');
  if (flippedRight && flippedRight.kind === 'split') {
    const rightNode = flippedRight.children[1];
    assert.equal(rightNode.kind, 'split');
    if (rightNode.kind === 'split') {
      assert.equal(rightNode.direction, 'horizontal');
    }
  }
});

test('resizeWindowInTree widens focused pane and clamps extreme sizes safely', () => {
  const tree = createThreePanePresetTree('vscode', 'terminal', 'files');
  const before = computeTiledLayout(tree, WORK_AREA, {
    innerGap: 10,
    outerGap: 12,
  });

  let resized = resizeWindowInTree(tree, 'vscode', 0.35);
  const after = computeTiledLayout(resized, WORK_AREA, {
    innerGap: 10,
    outerGap: 12,
  });
  assert.ok(after['vscode'].width > before['vscode'].width);

  // Repeated shrinking should clamp instead of collapsing to zero
  for (let i = 0; i < 12; i += 1) {
    resized = resizeWindowInTree(resized, 'vscode', -0.5);
  }
  const clamped = computeTiledLayout(resized, WORK_AREA, {
    innerGap: 10,
    outerGap: 12,
  });
  assert.ok(clamped['vscode'].width >= 180);
});

test('easeOutCubic, lerpRect, and staggeredFloatingRect produce smooth bounded animations', () => {
  assert.equal(easeOutCubic(0), 0);
  assert.equal(easeOutCubic(1), 1);
  assert.ok(easeOutCubic(0.5) > 0.8); // fast start, smooth deceleration

  const from: WindowRect = { x: 100, y: 100, width: 400, height: 300 };
  const to: WindowRect = { x: 200, y: 200, width: 800, height: 600 };
  const mid = lerpRect(from, to, 0.5);
  assert.ok(mid.x > 150 && mid.x < 200);
  assert.ok(mid.width > 600 && mid.width < 800);

  const float0 = staggeredFloatingRect(WORK_AREA, 0);
  const float1 = staggeredFloatingRect(WORK_AREA, 1);
  assert.ok(float1.x > float0.x);
  assert.ok(float1.y > float0.y);
  assert.ok(float1.x + float1.width <= WORK_AREA.x + WORK_AREA.width);
  assert.ok(float1.y + float1.height <= WORK_AREA.y + WORK_AREA.height);
});
