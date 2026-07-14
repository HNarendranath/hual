import { useState, useEffect } from 'react';
import { listPhotos, Photo } from '../lib/ipc';

export function usePhotos(dbPath: string) {
    const [photos, setPhotos] = useState<Photo[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        // eslint-disable-next-line react-hooks/set-state-in-effect
        setLoading(true);
        listPhotos(dbPath)
            .then(setPhotos)
            .catch((e) => setError(String(e)))
            .finally(() => setLoading(false));
    }, [dbPath]);

    return { photos, loading, error }
}