import test from 'node:test';
import assert from 'node:assert/strict';
import {
  defaultLumaConfig,
  parseAppletOrder,
  parsePanelZones,
  parseDockOrder,
  serializeSettingsIni,
  parseSettingsIni,
  calculateDockIconSize,
  moveAppletToZone,
  reorderAppletInZone,
  reorderDockItem,
  hexToRgbTuple,
} from './lumaConfig';

test('parseAppletOrder deduplicates and appends missing applets', () => {
  const order = parseAppletOrder('volume,clock,volume,invalid');
  assert.equal(order[0], 'volume');
  assert.equal(order[1], 'clock');
  assert.equal(order.length, 7);
  assert.ok(order.includes('notifications'));
});

test('parsePanelZones parses colon and equals pairs accurately', () => {
  const zones = parsePanelZones('clock:center,media=left,volume:right');
  assert.equal(zones.clock, 'center');
  assert.equal(zones.media, 'left');
  assert.equal(zones.volume, 'right');
  assert.equal(zones.network, 'right');
});

test('parseDockOrder preserves custom dock launcher sequence', () => {
  const dock = parseDockOrder('terminal,files,home');
  assert.deepEqual(dock, ['terminal', 'files', 'home', 'browser', 'settings']);
});

test('serializeSettingsIni and parseSettingsIni round-trip global accent, wallpaper, and font settings', () => {
  const cfg = defaultLumaConfig();
  cfg.position = 'left';
  cfg.dockStyle = 'panel';
  cfg.iconSize = 52;
  cfg.magnification = 65;
  cfg.panelZones.clock = 'center';
  cfg.panelZones.media = 'left';
  cfg.gpuBackend = 'direct2d';
  cfg.accentColor = '#f59e0b';
  cfg.wallpaperMode = 'vector_scene';
  cfg.wallpaperVectorPreset = 'nordic_ridges';
  cfg.fontScale = 105;

  const ini = serializeSettingsIni(cfg);
  const parsed = parseSettingsIni(ini);

  assert.equal(parsed.position, 'left');
  assert.equal(parsed.dockStyle, 'panel');
  assert.equal(parsed.iconSize, 52);
  assert.equal(parsed.magnification, 65);
  assert.equal(parsed.panelZones.clock, 'center');
  assert.equal(parsed.panelZones.media, 'left');
  assert.equal(parsed.gpuBackend, 'direct2d');
  assert.equal(parsed.accentColor, '#f59e0b');
  assert.equal(parsed.wallpaperMode, 'vector_scene');
  assert.equal(parsed.wallpaperVectorPreset, 'nordic_ridges');
  assert.equal(parsed.fontScale, 105);

  const orbitalDefault = defaultLumaConfig();
  assert.equal(orbitalDefault.dockStyle, 'pill');
  assert.equal(orbitalDefault.panelStyle, 'transparent');
  assert.equal(orbitalDefault.gpuShader, 'orbital_earth');
  assert.equal(orbitalDefault.adaptiveDockTransparency, true);
  assert.equal(orbitalDefault.transparency, 148);
  const orbitalParsed = parseSettingsIni(serializeSettingsIni(orbitalDefault));
  assert.equal(orbitalParsed.dockStyle, 'pill');
  assert.equal(orbitalParsed.panelStyle, 'transparent');
  assert.equal(orbitalParsed.gpuShader, 'orbital_earth');
  assert.equal(orbitalParsed.adaptiveDockTransparency, true);
  assert.equal(orbitalParsed.transparency, 148);

  orbitalDefault.adaptiveDockTransparency = false;
  orbitalDefault.transparency = 110;
  orbitalDefault.tilingEnabled = false;
  orbitalDefault.tilingDirection = 'vertical';
  orbitalDefault.tilingInnerGap = 16;
  orbitalDefault.tilingOuterGap = 20;
  orbitalDefault.tilingActiveHint = false;
  orbitalDefault.tilingAnimationMs = 320;
  const customDockParsed = parseSettingsIni(
    serializeSettingsIni(orbitalDefault)
  );
  assert.equal(customDockParsed.adaptiveDockTransparency, false);
  assert.equal(customDockParsed.transparency, 110);
  assert.equal(customDockParsed.tilingEnabled, false);
  assert.equal(customDockParsed.tilingDirection, 'vertical');
  assert.equal(customDockParsed.tilingInnerGap, 16);
  assert.equal(customDockParsed.tilingOuterGap, 20);
  assert.equal(customDockParsed.tilingActiveHint, false);
  assert.equal(customDockParsed.tilingAnimationMs, 320);
});

test('hexToRgbTuple converts hex color strings to RGB tuples', () => {
  assert.deepEqual(hexToRgbTuple('#38bdf8'), [56, 189, 248]);
  assert.deepEqual(hexToRgbTuple('#f59e0b'), [245, 158, 11]);
});

test('calculateDockIconSize magnifies hovered icon and leaves distant icons at base size', () => {
  const base = 40;
  const hoveredSize = calculateDockIconSize(base, 45, 200, 200, 1.0);
  const farSize = calculateDockIconSize(base, 45, 200, 450, 1.0);
  assert.ok(hoveredSize > base);
  assert.equal(farSize, base);
});

test('moveAppletToZone and reorderAppletInZone update COSMIC panel layout', () => {
  let cfg = defaultLumaConfig();
  cfg = moveAppletToZone(cfg, 'battery', 'left');
  assert.equal(cfg.panelZones.battery, 'left');

  const rightBefore = cfg.appletOrder.filter(
    (id) => cfg.panelZones[id] === 'right'
  );
  const secondRight = rightBefore[1];
  cfg = reorderAppletInZone(cfg, secondRight, 'up');
  const rightAfter = cfg.appletOrder.filter(
    (id) => cfg.panelZones[id] === 'right'
  );
  assert.equal(rightAfter[0], secondRight);
});

test('reorderDockItem moves launcher cleanly', () => {
  const initial = defaultLumaConfig().dockOrder;
  const updated = reorderDockItem(initial, 0, 2);
  assert.deepEqual(updated, [
    'files',
    'terminal',
    'home',
    'browser',
    'settings',
  ]);
});
