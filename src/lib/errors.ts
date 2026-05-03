export class AppCommandError extends Error {
  kind: string | null;

  constructor(message: string, kind: string | null = null) {
    super(message);
    this.name = "AppCommandError";
    this.kind = kind;
  }
}

export function normalizeCommandError(error: unknown): AppCommandError {
  if (error instanceof AppCommandError) {
    return error;
  }

  if (error instanceof Error) {
    return new AppCommandError(error.message);
  }

  if (typeof error === "string") {
    return new AppCommandError(error);
  }

  if (error && typeof error === "object") {
    const payload = error as { kind?: unknown; message?: unknown };
    const kind = typeof payload.kind === "string" ? payload.kind : null;
    const message =
      typeof payload.message === "string" ? payload.message : JSON.stringify(error);

    return new AppCommandError(message, kind);
  }

  return new AppCommandError(String(error));
}
