const DATE_TIME_FORMATTER = new Intl.DateTimeFormat(undefined, {
  year: "numeric",
  month: "2-digit",
  day: "2-digit",
  hour: "2-digit",
  minute: "2-digit",
  second: "2-digit",
});

export function parseTimestamp(value: string | null | undefined): Date | null {
  if (!value) {
    return null;
  }

  const trimmed = value.trim();
  if (!trimmed) {
    return null;
  }

  let parsedDate: Date;

  if (/^\d+$/.test(trimmed)) {
    const numericValue = Number(trimmed);
    if (!Number.isFinite(numericValue)) {
      return null;
    }

    parsedDate = new Date(trimmed.length > 10 ? numericValue : numericValue * 1000);
  } else {
    let normalizedValue = trimmed;

    if (/^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}/.test(normalizedValue)) {
      normalizedValue = normalizedValue.replace(" ", "T");
    }

    if (
      /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?$/.test(normalizedValue)
    ) {
      normalizedValue = `${normalizedValue}Z`;
    }

    parsedDate = new Date(normalizedValue);
  }

  if (Number.isNaN(parsedDate.getTime())) {
    return null;
  }

  return parsedDate;
}

export function formatDateTime(value: string | null | undefined): string {
  const parsedDate = parseTimestamp(value);
  if (!parsedDate) {
    return "Unknown time";
  }

  return DATE_TIME_FORMATTER.format(parsedDate);
}

export function formatRelativeTime(value: string | null | undefined): string {
  const parsedDate = parseTimestamp(value);
  if (!parsedDate) {
    return "Never";
  }

  const diffMs = Date.now() - parsedDate.getTime();
  const diffSecs = Math.floor(diffMs / 1000);
  const diffMins = Math.floor(diffSecs / 60);
  const diffHours = Math.floor(diffMins / 60);
  const diffDays = Math.floor(diffHours / 24);

  if (diffSecs < 60) return "Just now";
  if (diffMins < 60) return `${diffMins} minute${diffMins > 1 ? "s" : ""} ago`;
  if (diffHours < 24) return `${diffHours} hour${diffHours > 1 ? "s" : ""} ago`;
  if (diffDays < 7) return `${diffDays} day${diffDays > 1 ? "s" : ""} ago`;

  return formatDateTime(value);
}
