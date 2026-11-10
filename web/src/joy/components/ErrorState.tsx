/**
 * One calm error line with a way out. Shared so every Dates page fails the same way:
 * what happened (plain words), then a retry button when there is something to retry.
 */
export function ErrorState({
  message,
  onRetry,
  title,
}: {
  /** What went wrong, already in plain words (see `errText`). */
  message: string;
  /** Omit when retrying would not help (a missing page, say). */
  onRetry?: () => void;
  title?: string;
}) {
  return (
    <div className="error-state" role="alert">
      {title && <h2>{title}</h2>}
      <p className="kind-error">{message}</p>
      {onRetry && (
        <button type="button" className="btn" onClick={onRetry}>
          Try again
        </button>
      )}
    </div>
  );
}
