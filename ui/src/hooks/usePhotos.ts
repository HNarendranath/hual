import { useState, useEffect } from 'react';
import { listPhotos, Photo, PhotoFilters } from '../lib/ipc';
import { useDebouncedValue } from './useDebouncedValue';

const FILTER_DEBOUNCE_DELAY_MS = 300;

export function usePhotos(dbPath: string, filters: PhotoFilters) {
    const [photos, setPhotos] = useState<Photo[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    const debouncedFilters = useDebouncedValue(filters, FILTER_DEBOUNCE_DELAY_MS);

    useEffect(() => {
        // eslint-disable-next-line react-hooks/set-state-in-effect
        setLoading(true);
        listPhotos(dbPath, debouncedFilters)
            .then(setPhotos)
            .catch((e) => setError(String(e)))
            .finally(() => setLoading(false));
    }, [dbPath, debouncedFilters]);

    return { photos, loading, error }
}