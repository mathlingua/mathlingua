type ViewerChromeProps = {
  title: string;
};

export function ViewerChrome({ title }: ViewerChromeProps) {
  return (
    <header className="viewer-chrome">
      <button
        aria-label="Open navigation"
        className="chrome-button"
        type="button"
      >
        <svg
          aria-hidden="true"
          fill="none"
          height="26"
          viewBox="0 0 24 24"
          width="26"
        >
          <path
            d="M4 6.5H20M4 12H20M4 17.5H20"
            stroke="currentColor"
            strokeLinecap="round"
            strokeWidth="1.9"
          />
        </svg>
      </button>
      <div className="chrome-title-block">
        <p className="chrome-kicker">Mathlingua Viewer</p>
        <h1 className="chrome-title">{title}</h1>
      </div>
      <button
        aria-label="Viewer settings"
        className="chrome-button"
        type="button"
      >
        <svg
          aria-hidden="true"
          fill="none"
          height="30"
          viewBox="0 0 24 24"
          width="30"
        >
          <path
            d="M11.95 3.4C12.38 3.4 12.75 3.7 12.84 4.12L13.16 5.72C13.6 5.88 14 6.08 14.39 6.33L15.89 5.73C16.29 5.57 16.75 5.72 16.98 6.08L18.27 8.08C18.5 8.44 18.45 8.92 18.16 9.22L16.98 10.42C17.05 10.68 17.1 10.96 17.13 11.24C17.16 11.5 17.17 11.75 17.16 12.02L18.39 13.12C18.71 13.41 18.79 13.89 18.58 14.27L17.43 16.33C17.22 16.71 16.77 16.89 16.35 16.77L14.76 16.33C14.39 16.65 13.98 16.91 13.53 17.11L13.28 18.74C13.22 19.16 12.86 19.47 12.44 19.49L10.07 19.58C9.63 19.59 9.24 19.29 9.14 18.86L8.76 17.28C8.31 17.11 7.89 16.89 7.5 16.62L6.02 17.18C5.61 17.34 5.14 17.18 4.92 16.81L3.67 14.79C3.45 14.42 3.52 13.95 3.82 13.66L5.03 12.49C4.98 12.24 4.95 11.99 4.93 11.73C4.91 11.44 4.91 11.15 4.94 10.88L3.79 9.75C3.49 9.45 3.43 8.99 3.65 8.63L4.87 6.62C5.1 6.26 5.56 6.12 5.96 6.29L7.49 6.93C7.84 6.69 8.22 6.49 8.63 6.33L8.92 4.7C9 4.28 9.36 3.97 9.79 3.95L11.95 3.4Z"
            stroke="currentColor"
            strokeLinejoin="round"
            strokeWidth="1.2"
          />
          <circle
            cx="11.95"
            cy="11.5"
            r="2.95"
            stroke="currentColor"
            strokeWidth="1.2"
          />
        </svg>
      </button>
    </header>
  );
}
