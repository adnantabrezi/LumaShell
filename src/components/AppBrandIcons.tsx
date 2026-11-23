import React from 'react';

export type AppIconId =
  | 'brave'
  | 'youtube'
  | 'antigravity'
  | 'chatgpt'
  | 'whatsapp'
  | 'roblox'
  | 'spotify'
  | 'vscode'
  | 'terminal'
  | 'settings'
  | 'files'
  | 'home';

interface AppBrandIconProps {
  id: AppIconId;
  className?: string;
}

export const AppBrandIcon: React.FC<AppBrandIconProps> = ({
  id,
  className = 'w-full h-full',
}) => {
  switch (id) {
    case 'brave':
      return (
        <svg viewBox="0 0 64 64" className={className} aria-hidden="true">
          <defs>
            <linearGradient id="braveGrad" x1="0%" y1="0%" x2="100%" y2="100%">
              <stop offset="0%" stopColor="#ff6b00" />
              <stop offset="55%" stopColor="#f3440d" />
              <stop offset="100%" stopColor="#d9200b" />
            </linearGradient>
          </defs>
          {/* Outer Lion Shield */}
          <path
            d="M19 9 L25 6 L39 6 L45 9 L52 14 L50 22 L53 31 L46 47 L32 58 L18 47 L11 31 L14 22 L12 14 Z"
            fill="url(#braveGrad)"
          />
          {/* Ear accents */}
          <path d="M16 14 L23 11 L21 19 Z" fill="#ff9422" />
          <path d="M48 14 L41 11 L43 19 Z" fill="#ff9422" />
          {/* Geometric White Lion Face */}
          <path
            d="M20 23 L26 19 L32 22 L38 19 L44 23 L41 34 L36 37 L38 41 L32 48 L26 41 L28 37 L23 34 Z"
            fill="#ffffff"
          />
          {/* Eyes & Brow */}
          <path d="M23 26 L29 27 L27 30 Z" fill="#f3440d" />
          <path d="M41 26 L35 27 L37 30 Z" fill="#f3440d" />
          {/* Nose & Chin Bridge */}
          <path d="M29 33 L35 33 L32 38 Z" fill="#f3440d" />
          <path d="M29 41 L32 44 L35 41 L32 42.5 Z" fill="#f3440d" />
        </svg>
      );

    case 'youtube':
      return (
        <svg viewBox="0 0 64 64" className={className} aria-hidden="true">
          <rect
            x="4"
            y="12"
            width="56"
            height="40"
            rx="12"
            fill="#ff0826"
          />
          <polygon points="26,23 44,32 26,41" fill="#ffffff" />
        </svg>
      );

    case 'antigravity':
      return (
        <svg viewBox="0 0 64 64" className={className} aria-hidden="true">
          <defs>
            <linearGradient id="agArch" x1="15%" y1="95%" x2="85%" y2="5%">
              <stop offset="0%" stopColor="#38bdf8" />
              <stop offset="35%" stopColor="#4f46e5" />
              <stop offset="68%" stopColor="#ec4899" />
              <stop offset="100%" stopColor="#fbbf24" />
            </linearGradient>
            <pattern
              id="agGrid"
              width="6"
              height="6"
              patternUnits="userSpaceOnUse"
            >
              <path
                d="M 6 0 L 0 0 0 6"
                fill="none"
                stroke="#ffffff"
                strokeOpacity="0.09"
                strokeWidth="0.6"
              />
            </pattern>
          </defs>
          <rect x="4" y="4" width="56" height="56" rx="10" fill="#14161e" />
          <rect x="4" y="4" width="56" height="56" rx="10" fill="url(#agGrid)" />
          <path
            d="M14 52 C21 20, 26 12, 32 12 C38 12, 43 20, 50 52 C44 47, 39 36, 32 36 C25 36, 20 47, 14 52 Z"
            fill="url(#agArch)"
          />
        </svg>
      );

    case 'chatgpt':
      return (
        <svg viewBox="0 0 64 64" className={className} aria-hidden="true">
          <rect x="4" y="4" width="56" height="56" rx="16" fill="#f4f6f9" />
          <g
            transform="translate(32 32)"
            fill="none"
            stroke="#111827"
            strokeWidth="3.2"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            {[0, 60, 120, 180, 240, 300].map((deg) => (
              <path
                key={deg}
                transform={`rotate(${deg})`}
                d="M 0 -7 L 6 -3.5 L 6 7 C 6 12 1 15 -4 13 C -9 11 -11 6 -9 1 L -6 -3.5"
              />
            ))}
          </g>
        </svg>
      );

    case 'whatsapp':
      return (
        <svg viewBox="0 0 64 64" className={className} aria-hidden="true">
          <defs>
            <linearGradient id="waGrad" x1="0%" y1="0%" x2="100%" y2="100%">
              <stop offset="0%" stopColor="#34d352" />
              <stop offset="100%" stopColor="#16a34a" />
            </linearGradient>
          </defs>
          {/* Outer white speech bubble rim */}
          <path
            d="M32 6 C17.6 6 6 17.4 6 31.5 C6 36.6 7.5 41.4 10.2 45.4 L7 57 L19 53.8 C22.9 55.9 27.3 57 32 57 C46.4 57 58 45.6 58 31.5 C58 17.4 46.4 6 32 6 Z"
            fill="#ffffff"
          />
          {/* Inner green bubble */}
          <path
            d="M32 10.5 C20.1 10.5 10.5 19.9 10.5 31.5 C10.5 36.1 12 40.3 14.6 43.8 L12.5 51.5 L20.5 49.4 C23.9 51.4 27.8 52.5 32 52.5 C43.9 52.5 53.5 43.1 53.5 31.5 C53.5 19.9 43.9 10.5 32 10.5 Z"
            fill="url(#waGrad)"
          />
          {/* Stylized Z / bolt highlight */}
          <path
            d="M17 24 L43 24 L25 38 L47 38"
            fill="none"
            stroke="#ffffff"
            strokeWidth="4.2"
            strokeLinecap="round"
            strokeLinejoin="round"
          />
        </svg>
      );

    case 'roblox':
      return (
        <svg viewBox="0 0 64 64" className={className} aria-hidden="true">
          <defs>
            <linearGradient id="rblxGrad" x1="0%" y1="0%" x2="100%" y2="100%">
              <stop offset="0%" stopColor="#84f05c" />
              <stop offset="100%" stopColor="#26a95b" />
            </linearGradient>
          </defs>
          <g transform="rotate(15 32 32)">
            <rect
              x="11"
              y="11"
              width="42"
              height="42"
              rx="6"
              fill="url(#rblxGrad)"
            />
            {/* Horizontal fold shadow */}
            <line
              x1="11"
              y1="32"
              x2="53"
              y2="32"
              stroke="#0f172a"
              strokeWidth="2.4"
              strokeOpacity="0.65"
            />
            {/* Center tilted square hole */}
            <rect x="26" y="26" width="12" height="12" rx="2" fill="#0d0e12" />
          </g>
        </svg>
      );

    case 'spotify':
      return (
        <svg viewBox="0 0 64 64" className={className} aria-hidden="true">
          <circle cx="32" cy="32" r="26" fill="#1ed760" />
          <path
            d="M18 24.5 C27 21.8 39 22.3 47 26.8"
            fill="none"
            stroke="#0b0d10"
            strokeWidth="4.5"
            strokeLinecap="round"
          />
          <path
            d="M20.5 33.5 C28 31.3 37.8 31.8 44.5 35.5"
            fill="none"
            stroke="#0b0d10"
            strokeWidth="3.8"
            strokeLinecap="round"
          />
          <path
            d="M22.5 41.8 C28.8 40.1 36.8 40.5 42.2 43.4"
            fill="none"
            stroke="#0b0d10"
            strokeWidth="3.2"
            strokeLinecap="round"
          />
        </svg>
      );

    case 'vscode':
      return (
        <svg viewBox="0 0 64 64" className={className} aria-hidden="true">
          <defs>
            <linearGradient id="vscBlue" x1="0%" y1="0%" x2="100%" y2="100%">
              <stop offset="0%" stopColor="#38bdf8" />
              <stop offset="100%" stopColor="#0284c7" />
            </linearGradient>
          </defs>
          {/* Back cross ribbon */}
          <path
            d="M10 22 L16 18 L47 45 L47 56 L10 25 Z"
            fill="#0369a1"
          />
          {/* Mid cross ribbon */}
          <path
            d="M10 42 L16 46 L47 19 L47 8 L10 39 Z"
            fill="#0ea5e9"
          />
          {/* Right thick bar */}
          <path
            d="M44 9 L55 14.5 L55 49.5 L44 55 Z"
            fill="url(#vscBlue)"
          />
        </svg>
      );

    case 'terminal':
      return (
        <svg viewBox="0 0 64 64" className={className} aria-hidden="true">
          <rect
            x="6"
            y="10"
            width="52"
            height="44"
            rx="8"
            fill="#172b4d"
            stroke="#64748b"
            strokeWidth="2.4"
          />
          <polyline
            points="16,24 24,30 16,36"
            fill="none"
            stroke="#f59e0b"
            strokeWidth="3.6"
            strokeLinecap="round"
            strokeLinejoin="round"
          />
          <line
            x1="28"
            y1="36"
            x2="38"
            y2="36"
            stroke="#f59e0b"
            strokeWidth="3.6"
            strokeLinecap="round"
          />
        </svg>
      );

    case 'settings':
      return (
        <svg viewBox="0 0 64 64" className={className} aria-hidden="true">
          <rect x="8" y="8" width="48" height="48" rx="11" fill="#a8a29e" />
          <g transform="translate(32 32)" fill="#1c1917">
            {[0, 45, 90, 135, 180, 225, 270, 315].map((deg) => (
              <rect
                key={deg}
                x="-3.2"
                y="-15.5"
                width="6.4"
                height="7"
                rx="1.5"
                transform={`rotate(${deg})`}
              />
            ))}
            <circle cx="0" cy="0" r="11" />
            <circle cx="0" cy="0" r="5" fill="#a8a29e" />
          </g>
        </svg>
      );

    case 'files':
      return (
        <svg viewBox="0 0 64 64" className={className} aria-hidden="true">
          <path
            d="M8 16 C8 12.7 10.7 10 14 10 L26 10 L31 16 L50 16 C53.3 16 56 18.7 56 22 L56 48 C56 51.3 53.3 54 50 54 L14 54 C10.7 54 8 51.3 8 48 Z"
            fill="#0284c7"
          />
          <rect x="8" y="21" width="48" height="33" rx="6" fill="#38bdf8" />
        </svg>
      );

    case 'home':
    default:
      return (
        <svg viewBox="0 0 64 64" className={className} aria-hidden="true">
          <rect x="6" y="6" width="52" height="52" rx="14" fill="#1e293b" />
          <circle cx="22" cy="22" r="5" fill="#38bdf8" />
          <circle cx="42" cy="22" r="5" fill="#f59e0b" />
          <circle cx="22" cy="42" r="5" fill="#34d399" />
          <circle cx="42" cy="42" r="5" fill="#f472b6" />
        </svg>
      );
  }
};
