// Simple logger with log levels and colored output

export enum LogLevel {
  DEBUG = 0,
  INFO = 1,
  WARN = 2,
  ERROR = 3,
}

const LOG_COLORS = {
  [LogLevel.DEBUG]: "color: #888",
  [LogLevel.INFO]: "color: #0088ff",
  [LogLevel.WARN]: "color: #ff8800",
  [LogLevel.ERROR]: "color: #ff0000",
};

const LOG_PREFIXES = {
  [LogLevel.DEBUG]: "[DEBUG]",
  [LogLevel.INFO]: "[INFO]",
  [LogLevel.WARN]: "[WARN]",
  [LogLevel.ERROR]: "[ERROR]",
};

class Logger {
  private level: LogLevel;

  constructor() {
    // Read from environment or default to INFO
    const envLevel = import.meta.env.VITE_LOG_LEVEL as string | undefined;
    this.level = this.parseLevel(envLevel) ?? LogLevel.INFO;
  }

  private parseLevel(level?: string): LogLevel | null {
    if (level === undefined || level === null || level === "") {
      return null;
    }
    switch (level.toLowerCase()) {
      case "debug":
        return LogLevel.DEBUG;
      case "info":
        return LogLevel.INFO;
      case "warn":
        return LogLevel.WARN;
      case "error":
        return LogLevel.ERROR;
      default:
        return null;
    }
  }

  private log(level: LogLevel, message: string, ...args: unknown[]): void {
    if (level < this.level) {
      return;
    }

    const timestamp = new Date().toISOString().split("T")[1].split(".")[0];
    const prefix = `%c${timestamp} ${LOG_PREFIXES[level]}`;
    const style = LOG_COLORS[level];

    // eslint-disable-next-line no-console
    console.log(prefix, style, message, ...args);
  }

  debug(message: string, ...args: unknown[]): void {
    this.log(LogLevel.DEBUG, message, ...args);
  }

  info(message: string, ...args: unknown[]): void {
    this.log(LogLevel.INFO, message, ...args);
  }

  warn(message: string, ...args: unknown[]): void {
    this.log(LogLevel.WARN, message, ...args);
  }

  error(message: string, ...args: unknown[]): void {
    this.log(LogLevel.ERROR, message, ...args);
  }

  setLevel(level: LogLevel): void {
    this.level = level;
  }
}

export const logger = new Logger();

// Export for debugging
(window as unknown as Record<string, unknown>).logger = logger;
(window as unknown as Record<string, unknown>).LogLevel = LogLevel;
