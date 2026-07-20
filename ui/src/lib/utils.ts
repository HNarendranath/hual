export function formatExposureTime(seconds: number | null): string {
    if (seconds === null || seconds <= 0) return '—';
    if (seconds >= 1) {
        return `${Number(seconds.toFixed(1))}s`;
    }
    return `1/${Math.round(1 / seconds)}`;
}

export function formatExposureInput(seconds: number | null): string {
    if (seconds === null) return '';
    if (seconds >= 1) return String(Number(seconds.toFixed(2)));
    return `1/${Math.round(1 / seconds)}`;
}

export function parseExposureInput(raw: string): number | null {
    const trimmed = raw.trim();
    if (trimmed === '') return null;

    const fraction = trimmed.match(/^(\d+(?:\.\d+)?)\s*\/\s*(\d+(?:\.\d+)?)$/);
    if (fraction) {
        const numerator = Number(fraction[1]);
        const denominator = Number(fraction[2]);
        return denominator === 0 ? null : numerator / denominator;
    }

    const value = Number(trimmed);
    return Number.isFinite(value) ? value : null;
}

export function formatFocalLength(mm: number | null): string {
    if (mm === null) return '—';
    return `${Number(mm.toFixed(1))}mm`;
}
