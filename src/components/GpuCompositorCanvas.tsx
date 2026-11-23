import React, { useEffect, useRef, useState } from 'react';
import {
  GpuShaderPreset,
  hexToRgbTuple,
  LumaConfig,
} from '../utils/lumaConfig';

interface GpuCompositorCanvasProps {
  config: LumaConfig;
  animateShader: boolean;
  onDesktopContextMenu: (x: number, y: number) => void;
  onDesktopClick: () => void;
}

const VERTEX_SHADER = `
  attribute vec2 a_position;
  varying vec2 v_uv;
  void main() {
    v_uv = a_position * 0.5 + 0.5;
    gl_Position = vec4(a_position, 0.0, 1.0);
  }
`;

const FRAGMENT_SHADER = `
  precision highp float;
  varying vec2 v_uv;
  uniform float u_time;
  uniform vec2 u_resolution;
  uniform int u_preset;
  uniform float u_bloom;
  uniform int u_dark;
  uniform vec3 u_accent;
  uniform int u_sync_accent;

  // High-frequency 2D hash & value noise for procedural orbital planetary terrain
  float hash21(vec2 p) {
    p = fract(p * vec2(234.34, 435.345));
    p += dot(p, p + 34.23);
    return fract(p.x * p.y);
  }

  float noise2d(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p);
    vec2 u = f * f * (3.0 - 2.0 * f);
    float a = hash21(i);
    float b = hash21(i + vec2(1.0, 0.0));
    float c = hash21(i + vec2(0.0, 1.0));
    float d = hash21(i + vec2(1.0, 1.0));
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
  }

  float fbm(vec2 p) {
    float v = 0.0;
    float a = 0.5;
    mat2 rot = mat2(0.80, -0.60, 0.60, 0.80);
    for (int i = 0; i < 6; i++) {
      v += a * noise2d(p);
      p = rot * p * 2.08 + vec2(1.7, 9.2);
      a *= 0.5;
    }
    return v;
  }

  void main() {
    vec2 uv = v_uv;
    float aspect = u_resolution.x / max(u_resolution.y, 1.0);
    vec2 p = (uv - 0.5) * vec2(aspect, 1.0);
    float t = u_time * 0.15;

    // PRESET 0: ORBITAL EARTH HORIZON (Matches user reference screenshot)
    if (u_preset == 0) {
      // Gentle planetary limb sweeping from bottom-left (x~0.14, y=0) to top-right (x~0.87, y=1)
      vec2 sc = vec2(uv.x * aspect, uv.y);
      vec2 planetCenter = vec2(2.42 * aspect, -1.78);
      float planetRadius = 3.52;
      vec2 rel = sc - planetCenter;
      float dist = length(rel);
      // signed distance to limb: < 0 inside planet, > 0 in deep space
      float sdf = dist - planetRadius;

      // Pure pitch-black deep space background matching reference image
      vec3 spaceCol = vec3(0.002, 0.003, 0.006);

      // Authentic royal-cobalt & sapphire Rayleigh atmospheric limb scattering
      vec3 atmoColor = vec3(0.04, 0.34, 0.92);
      vec3 atmoInner = vec3(0.36, 0.62, 0.94);
      if (u_sync_accent == 1) {
        atmoColor = mix(atmoColor, u_accent, 0.65);
        atmoInner = mix(atmoInner, u_accent * 1.15, 0.5);
      }

      // Tight, realistic atmospheric limb glow into space
      float outerAtmo = exp(-max(sdf, 0.0) * 52.0) * (0.78 + 0.25 * u_bloom);
      float wideAtmo = exp(-max(sdf, 0.0) * 18.0) * 0.18;

      vec3 col = spaceCol + atmoColor * (outerAtmo + wideAtmo);

      if (sdf < 0.012) {
        // Inside planetary disc: compute 3D spherical surface coordinates for foreshortening near the limb
        float rNorm = clamp(dist / planetRadius, 0.0, 0.9995);
        float zSphere = sqrt(max(1.0 - rNorm * rNorm, 0.0005));
        vec2 sphereUV = (rel / planetRadius) / (0.28 + 0.72 * zSphere);

        // Rotate coordinates to align desert ridges & canyons parallel to the diagonal horizon limb
        mat2 limbRot = mat2(0.74, -0.67, 0.67, 0.74);
        vec2 tUV = limbRot * sphereUV * 4.2 + vec2(t * 0.02, -t * 0.01);

        // Domain-warped fBM for wind-carved Sahara / Namib desert dunes & dark basalt plateaus
        float warp1 = fbm(tUV * 1.15);
        float warp2 = fbm(tUV * 2.25 + vec2(warp1 * 2.1, -warp1 * 1.6));
        float ridge = fbm(tUV * 4.5 + vec2(warp2 * 2.4, warp1 * 1.7));
        float fineSand = fbm(tUV * 12.0 + warp2 * 2.8);

        // Rich desert palette from reference screenshot: dark volcanic basalt, terracotta, golden ochre, sunlit sand
        vec3 deepBasalt = vec3(0.15, 0.16, 0.20);
        vec3 terracotta = vec3(0.75, 0.42, 0.31);
        vec3 goldenOchre = vec3(0.88, 0.58, 0.36);
        vec3 paleSand = vec3(0.93, 0.74, 0.52);

        vec3 terrain = mix(terracotta, goldenOchre, smoothstep(0.26, 0.72, warp2));
        terrain = mix(terrain, paleSand, smoothstep(0.52, 0.86, ridge) * 0.72);

        // Crisp dark basalt rock outcrops and canyon formations
        float basaltMask = smoothstep(0.49, 0.28, warp1 * 0.58 + ridge * 0.44);
        terrain = mix(terrain, deepBasalt, basaltMask * 0.86);

        // Fine dune ripples & micro-relief lighting
        terrain *= 0.84 + 0.32 * fineSand;

        // Rayleigh atmospheric blue haze thickening toward the curved planetary horizon limb
        float limbDepth = smoothstep(-0.34, 0.0, sdf);
        float horizonScatter = pow(limbDepth, 1.65);
        terrain = mix(terrain, vec3(0.34, 0.50, 0.78), horizonScatter * 0.76);
        terrain = mix(terrain, atmoInner, pow(limbDepth, 4.2) * 0.85);
        terrain += atmoColor * pow(limbDepth, 7.0) * (0.45 + u_bloom * 0.35);

        // Anti-aliased transition across the planetary limb edge
        float planetMask = smoothstep(0.005, -0.005, sdf);
        col = mix(col, terrain, planetMask);
      }

      if (u_dark == 0) {
        col = mix(col, vec3(0.92, 0.95, 0.99), 0.18);
      }

      gl_FragColor = vec4(col, 1.0);
      return;
    }

    // PRESETS 1..4: COSMIC Procedural Nebulae
    float wave1 = sin(p.x * 2.4 + t * 2.0) * cos(p.y * 2.1 - t * 1.4);
    float wave2 = sin((p.x - p.y) * 3.1 - t) * 0.5;
    float radial = length(p);
    float glow = exp(-radial * 1.65) * (0.65 + 0.35 * sin(t * 2.0 + radial * 3.0));

    vec3 baseDark;
    vec3 accentA;
    vec3 accentB;

    if (u_preset == 1) {
      baseDark = vec3(0.045, 0.055, 0.085);
      accentA = vec3(0.14, 0.56, 0.86);
      accentB = vec3(0.11, 0.72, 0.65);
    } else if (u_preset == 2) {
      baseDark = vec3(0.06, 0.06, 0.075);
      accentA = vec3(0.88, 0.45, 0.12);
      accentB = vec3(0.22, 0.68, 0.92);
    } else if (u_preset == 3) {
      baseDark = vec3(0.06, 0.045, 0.075);
      accentA = vec3(0.95, 0.55, 0.18);
      accentB = vec3(0.88, 0.25, 0.48);
    } else {
      baseDark = vec3(0.035, 0.065, 0.075);
      accentA = vec3(0.10, 0.78, 0.62);
      accentB = vec3(0.22, 0.62, 0.95);
    }

    if (u_sync_accent == 1) {
      accentA = mix(accentA, u_accent, 0.68);
      accentB = mix(accentB, u_accent * vec3(0.85, 1.05, 1.15), 0.42);
    }

    vec3 col = baseDark;
    float nebulaA = smoothstep(-0.6, 0.85, wave1 + glow * 0.6);
    float nebulaB = smoothstep(-0.4, 0.9, wave2 + (1.0 - uv.y) * 0.45);

    col = mix(col, accentA * 0.54, nebulaA * (0.45 + u_bloom * 0.45));
    col = mix(col, accentB * 0.48, nebulaB * (0.35 + u_bloom * 0.40));
    col += (accentA * 0.24 + accentB * 0.18) * glow * u_bloom;

    if (u_dark == 0) {
      vec3 lightBase = vec3(0.92, 0.95, 0.98);
      col = mix(lightBase, accentA * 0.35 + vec3(0.65), nebulaA * 0.45);
      col = mix(col, accentB * 0.30 + vec3(0.70), nebulaB * 0.35);
    }

    gl_FragColor = vec4(col, 1.0);
  }
`;

function presetToIndex(preset: GpuShaderPreset): number {
  switch (preset) {
    case 'orbital_earth':
      return 0;
    case 'cosmic_nebula':
      return 1;
    case 'system76_obsidian':
      return 2;
    case 'solar_flare':
      return 3;
    case 'teal_aurora':
      return 4;
  }
}

export const GpuCompositorCanvas: React.FC<GpuCompositorCanvasProps> = ({
  config,
  animateShader,
  onDesktopContextMenu,
  onDesktopClick,
}) => {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const configRef = useRef<LumaConfig>(config);
  const animateRef = useRef<boolean>(animateShader);
  const drawFrameRef = useRef<((timeMs: number) => void) | null>(null);
  const [customImgError, setCustomImgError] = useState<boolean>(false);

  configRef.current = config;
  animateRef.current = animateShader;

  useEffect(() => {
    setCustomImgError(false);
  }, [config.customWallpaperUrl]);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const gl =
      (canvas.getContext('webgl', {
        antialias: true,
        depth: false,
        stencil: false,
        preserveDrawingBuffer: true,
      }) as WebGLRenderingContext | null) || null;

    let program: WebGLProgram | null = null;
    let posBuffer: WebGLBuffer | null = null;
    let uTimeLoc: WebGLUniformLocation | null = null;
    let uResLoc: WebGLUniformLocation | null = null;
    let uPresetLoc: WebGLUniformLocation | null = null;
    let uBloomLoc: WebGLUniformLocation | null = null;
    let uDarkLoc: WebGLUniformLocation | null = null;
    let uAccentLoc: WebGLUniformLocation | null = null;
    let uSyncAccentLoc: WebGLUniformLocation | null = null;

    if (gl) {
      const compile = (type: number, src: string) => {
        const shader = gl.createShader(type);
        if (!shader) return null;
        gl.shaderSource(shader, src);
        gl.compileShader(shader);
        if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
          gl.deleteShader(shader);
          return null;
        }
        return shader;
      };

      const vs = compile(gl.VERTEX_SHADER, VERTEX_SHADER);
      const fs = compile(gl.FRAGMENT_SHADER, FRAGMENT_SHADER);
      if (vs && fs) {
        const prog = gl.createProgram();
        if (prog) {
          gl.attachShader(prog, vs);
          gl.attachShader(prog, fs);
          gl.linkProgram(prog);
          if (gl.getProgramParameter(prog, gl.LINK_STATUS)) {
            program = prog;
            gl.useProgram(program);

            posBuffer = gl.createBuffer();
            gl.bindBuffer(gl.ARRAY_BUFFER, posBuffer);
            gl.bufferData(
              gl.ARRAY_BUFFER,
              new Float32Array([-1, -1, 1, -1, -1, 1, -1, 1, 1, -1, 1, 1]),
              gl.STATIC_DRAW
            );

            const aPos = gl.getAttribLocation(program, 'a_position');
            gl.enableVertexAttribArray(aPos);
            gl.vertexAttribPointer(aPos, 2, gl.FLOAT, false, 0, 0);

            uTimeLoc = gl.getUniformLocation(program, 'u_time');
            uResLoc = gl.getUniformLocation(program, 'u_resolution');
            uPresetLoc = gl.getUniformLocation(program, 'u_preset');
            uBloomLoc = gl.getUniformLocation(program, 'u_bloom');
            uDarkLoc = gl.getUniformLocation(program, 'u_dark');
            uAccentLoc = gl.getUniformLocation(program, 'u_accent');
            uSyncAccentLoc = gl.getUniformLocation(program, 'u_sync_accent');
          }
        }
      }
    }

    const renderFrame = (timeMs: number) => {
      const c = configRef.current;
      const dpr = Math.min(window.devicePixelRatio || 1, 2);
      const width = Math.round((canvas.clientWidth || 1440) * dpr);
      const height = Math.round((canvas.clientHeight || 900) * dpr);
      if (canvas.width !== width || canvas.height !== height) {
        canvas.width = width;
        canvas.height = height;
      }

      const [r, g, b] = hexToRgbTuple(c.accentColor);

      if (gl && program) {
        gl.viewport(0, 0, canvas.width, canvas.height);
        gl.useProgram(program);
        gl.uniform1f(uTimeLoc, timeMs * 0.001);
        gl.uniform2f(uResLoc, canvas.width, canvas.height);
        gl.uniform1i(uPresetLoc, presetToIndex(c.gpuShader));
        gl.uniform1f(uBloomLoc, c.gpuBloomStrength / 100.0);
        gl.uniform1i(uDarkLoc, c.dark ? 1 : 0);
        gl.uniform3f(uAccentLoc, r / 255.0, g / 255.0, b / 255.0);
        gl.uniform1i(uSyncAccentLoc, c.syncAccentToWallpaper ? 1 : 0);
        gl.drawArrays(gl.TRIANGLES, 0, 6);
      } else {
        const ctx2d = canvas.getContext('2d');
        if (ctx2d) {
          ctx2d.fillStyle = '#020409';
          ctx2d.fillRect(0, 0, canvas.width, canvas.height);

          const cx = canvas.width * 1.55;
          const cy = canvas.height * 1.85;
          const rad = Math.max(canvas.width, canvas.height) * 1.92;
          const planetGrad = ctx2d.createRadialGradient(
            cx - rad * 0.25,
            cy - rad * 0.25,
            rad * 0.1,
            cx,
            cy,
            rad
          );
          planetGrad.addColorStop(0, '#e09a63');
          planetGrad.addColorStop(0.55, '#c26944');
          planetGrad.addColorStop(0.88, '#3b73b8');
          planetGrad.addColorStop(0.97, '#1d6feb');
          planetGrad.addColorStop(1, 'rgba(3, 7, 18, 0)');
          ctx2d.fillStyle = planetGrad;
          ctx2d.beginPath();
          ctx2d.arc(cx, cy, rad, 0, Math.PI * 2);
          ctx2d.fill();
        }
      }
    };

    drawFrameRef.current = renderFrame;
    renderFrame(1200);

    let timerId: number | null = null;
    if (animateRef.current) {
      let virtualTime = 1200;
      timerId = window.setInterval(() => {
        if (animateRef.current) {
          virtualTime += 120;
          renderFrame(virtualTime);
        }
      }, 150);
    }

    const onResize = () => renderFrame(1500);
    window.addEventListener('resize', onResize);

    return () => {
      window.removeEventListener('resize', onResize);
      if (timerId !== null) window.clearInterval(timerId);
    };
  }, [animateShader]);

  useEffect(() => {
    if (drawFrameRef.current) {
      drawFrameRef.current(1600);
    }
  }, [
    config.gpuShader,
    config.gpuBloomStrength,
    config.gpuBlurRadius,
    config.gpuBackend,
    config.dark,
    config.desktopEnabled,
    config.accentColor,
    config.syncAccentToWallpaper,
    config.wallpaperMode,
  ]);

  const [r, g, b] = hexToRgbTuple(config.accentColor);

  return (
    <div
      onClick={onDesktopClick}
      onContextMenu={(e) => {
        e.preventDefault();
        if (config.desktopEnabled) {
          onDesktopContextMenu(e.clientX, e.clientY);
        }
      }}
      className={`absolute inset-0 w-full h-full overflow-hidden transition-opacity duration-300 ${
        config.desktopEnabled ? 'opacity-100' : 'opacity-20'
      }`}
    >
      {/* Mode 1: GPU Shader Canvas (Always mounted for instant switching) */}
      <canvas
        ref={canvasRef}
        data-testid="gpu-compositor-canvas"
        className={`absolute inset-0 w-full h-full object-cover ${
          config.wallpaperMode === 'gpu_shader' ? 'block' : 'hidden'
        }`}
      />

      {/* Mode 2: Crisp High-DPI Vector Art Wallpapers */}
      {config.wallpaperMode === 'vector_scene' && (
        <div
          data-testid="vector-wallpaper-surface"
          className="absolute inset-0 w-full h-full"
          style={{
            background: config.dark
              ? `radial-gradient(circle at 75% 20%, rgba(${r},${g},${b},0.32), transparent 55%), linear-gradient(135deg, #090d16 0%, #111827 52%, #0b1322 100%)`
              : `radial-gradient(circle at 75% 20%, rgba(${r},${g},${b},0.25), transparent 55%), linear-gradient(135deg, #f1f5f9 0%, #e2e8f0 100%)`,
          }}
        >
          {config.wallpaperVectorPreset === 'orbit_topography' && (
            <svg
              viewBox="0 0 1440 900"
              className="w-full h-full object-cover opacity-80"
              preserveAspectRatio="xMidYMid slice"
            >
              <defs>
                <linearGradient id="cosmicOrbit" x1="0%" y1="0%" x2="100%" y2="100%">
                  <stop offset="0%" stopColor={config.accentColor} stopOpacity="0.55" />
                  <stop offset="100%" stopColor="#14b8a6" stopOpacity="0.15" />
                </linearGradient>
              </defs>
              <circle cx="1060" cy="240" r="180" fill="url(#cosmicOrbit)" />
              <circle
                cx="1060"
                cy="240"
                r="290"
                fill="none"
                stroke={config.accentColor}
                strokeOpacity="0.28"
                strokeWidth="1.5"
              />
              <circle
                cx="1060"
                cy="240"
                r="420"
                fill="none"
                stroke={config.accentColor}
                strokeOpacity="0.18"
                strokeWidth="1.5"
                strokeDasharray="8 10"
              />
              <path
                d="M0,680 Q360,540 720,650 T1440,590 L1440,900 L0,900 Z"
                fill={config.accentColor}
                fillOpacity="0.12"
              />
              <path
                d="M0,760 Q420,640 860,730 T1440,700 L1440,900 L0,900 Z"
                fill={config.accentColor}
                fillOpacity="0.18"
              />
            </svg>
          )}

          {config.wallpaperVectorPreset === 'nordic_ridges' && (
            <svg
              viewBox="0 0 1440 900"
              className="w-full h-full object-cover"
              preserveAspectRatio="xMidYMid slice"
            >
              <polygon
                points="0,900 0,540 280,380 540,510 860,310 1190,480 1440,360 1440,900"
                fill={config.accentColor}
                fillOpacity="0.16"
              />
              <polygon
                points="0,900 0,650 380,470 710,610 1050,430 1440,590 1440,900"
                fill={config.accentColor}
                fillOpacity="0.24"
              />
              <polygon
                points="0,900 0,740 480,610 920,730 1440,640 1440,900"
                fill="#0f172a"
                fillOpacity="0.65"
              />
            </svg>
          )}

          {config.wallpaperVectorPreset === 'isometric_grid' && (
            <svg
              viewBox="0 0 1440 900"
              className="w-full h-full object-cover opacity-55"
              preserveAspectRatio="xMidYMid slice"
            >
              <defs>
                <pattern
                  id="isoGrid"
                  width="60"
                  height="60"
                  patternUnits="userSpaceOnUse"
                >
                  <path
                    d="M 60 0 L 0 0 0 60"
                    fill="none"
                    stroke={config.accentColor}
                    strokeWidth="1"
                    strokeOpacity="0.25"
                  />
                </pattern>
              </defs>
              <rect width="1440" height="900" fill="url(#isoGrid)" />
              <circle
                cx="720"
                cy="450"
                r="260"
                fill={config.accentColor}
                fillOpacity="0.18"
              />
            </svg>
          )}

          {config.wallpaperVectorPreset === 'sunset_dunes' && (
            <svg
              viewBox="0 0 1440 900"
              className="w-full h-full object-cover"
              preserveAspectRatio="xMidYMid slice"
            >
              <circle
                cx="720"
                cy="340"
                r="170"
                fill={config.accentColor}
                fillOpacity="0.38"
              />
              <path
                d="M0,620 C420,500 980,740 1440,580 L1440,900 L0,900 Z"
                fill={config.accentColor}
                fillOpacity="0.22"
              />
              <path
                d="M0,730 C520,640 880,820 1440,710 L1440,900 L0,900 Z"
                fill="#090d16"
                fillOpacity="0.7"
              />
            </svg>
          )}
        </div>
      )}

      {/* Mode 3: Global Accent Architectural Gradient Studio */}
      {config.wallpaperMode === 'gradient' && (
        <div
          data-testid="gradient-wallpaper-surface"
          className="absolute inset-0 w-full h-full"
          style={{
            background: config.dark
              ? `radial-gradient(1200px circle at 18% 22%, rgba(${r},${g},${b},0.38), transparent 60%), radial-gradient(1000px circle at 84% 78%, rgba(${r},${g},${b},0.22), transparent 55%), linear-gradient(160deg, #070a12 0%, #0f172a 55%, #090d16 100%)`
              : `radial-gradient(1200px circle at 18% 22%, rgba(${r},${g},${b},0.28), transparent 60%), linear-gradient(160deg, #f8fafc 0%, #e2e8f0 100%)`,
          }}
        />
      )}

      {/* Mode 4: Custom Wallpaper Image URL / Upload with Zero-Broken-Image Fallback */}
      {config.wallpaperMode === 'custom_image' && (
        <div
          data-testid="custom-wallpaper-surface"
          className="absolute inset-0 w-full h-full"
          style={{
            background: `radial-gradient(circle at 50% 35%, rgba(${r},${g},${b},0.35), #090d16 80%)`,
          }}
        >
          {config.customWallpaperUrl && !customImgError && (
            <img
              src={config.customWallpaperUrl}
              alt="Custom Luma Desktop Wallpaper"
              referrerPolicy="no-referrer"
              onError={() => setCustomImgError(true)}
              className="w-full h-full object-cover"
            />
          )}
        </div>
      )}

      {/* Global Wallpaper Contrast Scrim / Dimming Layer */}
      {config.wallpaperDimming > 0 && (
        <div
          aria-hidden="true"
          className="absolute inset-0 pointer-events-none transition-opacity"
          style={{
            backgroundColor: config.dark ? '#020617' : '#0f172a',
            opacity: config.wallpaperDimming / 100,
          }}
        />
      )}
    </div>
  );
};
