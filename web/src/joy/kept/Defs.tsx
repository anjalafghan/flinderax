export function Defs() {
  return (
    <svg className="k-defs" width="0" height="0" aria-hidden="true" focusable="false">
      <defs>
        <symbol id="k-heart" viewBox="0 0 32 30">
          <path d="M16.2 27.4C11.6 24.1 3.4 18.3 3.1 10.9 2.9 6.6 6 3.3 9.8 3.4c2.9.1 5 2.2 6.2 4.6 1.3-2.6 3.5-4.8 6.6-4.7 3.9.2 6.5 3.6 6.2 7.7-.5 7.2-8.4 12.6-12.6 16.4z" />
        </symbol>
        <symbol id="k-pin" viewBox="0 0 16 16">
          <path d="M8 14.6c-2.3-2.7-4.6-5.6-4.5-8.2C3.6 3.9 5.6 2 8.1 2.1c2.4.1 4.3 2 4.2 4.4-.1 2.6-2.2 5.4-4.3 8.1z" />
          <circle cx="8" cy="6.4" r="1.6" />
        </symbol>
        <symbol id="k-clock" viewBox="0 0 20 20">
          <path d="M10.2 2.6c4.2.1 7.3 3.4 7.2 7.5-.1 4.1-3.5 7.3-7.6 7.2C5.7 17.2 2.6 13.9 2.7 9.9 2.8 5.8 6.1 2.5 10.2 2.6z" />
          <path d="M10 5.9v4.4l2.9 1.8" />
        </symbol>
        <symbol id="k-cam" viewBox="0 0 20 20">
          <path d="M2.8 6.6c0-.9.7-1.5 1.6-1.5h2.2l1.3-1.9h4.3l1.3 1.9h2.2c.9 0 1.6.7 1.6 1.6l-.1 8.1c0 .9-.7 1.5-1.6 1.5H4.3c-.9 0-1.6-.7-1.6-1.6z" />
          <path d="M10.1 7.6c1.8 0 3.1 1.4 3.1 3.1s-1.4 3.1-3.2 3.1-3.1-1.4-3.1-3.2 1.4-3 3.2-3z" />
        </symbol>
        <symbol id="k-cal" viewBox="0 0 20 20">
          <path d="M3.2 5.4c0-.8.6-1.4 1.4-1.4l10.9-.1c.8 0 1.4.6 1.4 1.4v10c0 .8-.6 1.4-1.4 1.4H4.6c-.8 0-1.4-.6-1.4-1.4z" />
          <path d="M3.4 8.2h13.2M7 2.4v3M13.2 2.3v3.1M10 10.4v4.2M7.9 12.5h4.2" />
        </symbol>
      </defs>
    </svg>
  );
}

export const Icon = ({ id, size, className }: { id: string; size: number; className?: string }) => (
  <svg className={className} width={size} height={size} aria-hidden="true">
    <use href={`#${id}`} />
  </svg>
);
