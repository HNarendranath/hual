import { useState, useEffect, useCallback } from 'react';
import { listPhotos, Photo, PhotoFilters } from '../lib/ipc';
import { useDebouncedValue } from './useDebouncedValue';

const FILTER_DEBOUNCE_DELAY_MS = 300;

export function usePhotos(dbPath: string | null, filters: PhotoFilters) {
    const [photos, setPhotos] = useState<Photo[]>([]);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const debouncedFilters = useDebouncedValue(filters, FILTER_DEBOUNCE_DELAY_MS);

    const fetchPhotos = useCallback(() => {
        if (!dbPath) {
            setPhotos([]);
            return;
        }
        setLoading(true);
        setError(null);
        listPhotos(dbPath, debouncedFilters)
            .then(setPhotos)
            .catch((e) => setError(String(e)))
            .finally(() => setLoading(false));
    }, [dbPath, debouncedFilters]);

    useEffect(() => {
        fetchPhotos();
    }, [fetchPhotos]);

    return { photos, loading, error, refetch: fetchPhotos };
}
