// The dusk-over-water picture on the best-night print when the night has no photo of its own.
export function DuskArt() {
  return (
    <svg viewBox="0 0 192 196" preserveAspectRatio="xMidYMid slice" aria-hidden="true">
      <defs>
        <linearGradient id="y-dusk" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0" stopColor="#e9875a" />
          <stop offset="1" stopColor="#f4b371" />
        </linearGradient>
        <radialGradient id="y-vig" cx="50%" cy="50%" r="70%">
          <stop offset=".55" stopColor="#3c190a" stopOpacity="0" />
          <stop offset="1" stopColor="#3c190a" stopOpacity=".28" />
        </radialGradient>
      </defs>
      <rect width="192" height="196" fill="#f0a35e" />
      <rect width="192" height="102" fill="url(#y-dusk)" />
      <circle cx="96" cy="109" r="31" fill="#fde7b0" />
      <rect y="102" width="192" height="24" fill="#f6c890" />
      <rect y="126" width="192" height="70" fill="#3c6b8f" />
      <g fill="#f8d9a0" opacity=".8">
        <rect x="73" y="137" width="46" height="3" rx="1.5" />
        <rect x="61" y="157" width="70" height="3" rx="1.5" />
        <rect x="79" y="177" width="34" height="3" rx="1.5" />
      </g>
      <g fill="none" stroke="#7a3a2a" strokeWidth="1.6" strokeLinecap="round">
        <path d="M24,32 Q28,27 32,32 Q36,27 40,32" />
        <path d="M52,38 Q55,34 58,38 Q61,34 64,38" />
        <path d="M66,28 Q69,24 72,28 Q75,24 78,28" />
      </g>
      <rect width="192" height="196" fill="url(#y-vig)" />
    </svg>
  );
}
