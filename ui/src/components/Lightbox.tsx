import { useEffect, useState } from 'react';
import { getFullThumbnail } from '../lib/ipc'

interface Props {
    src: string;
    onClose: () => void;
}

export function Lightbox({ src, onClose }: Props) {
    const [url, setUrl] = useState<string | null>(null);

    useEffect(() => {
        let objectUrl: string | null = null;
        let cancelled = false;

        getFullThumbnail(src).then((bytes) => {
            if (cancelled) return;
            const blob = new Blob([bytes], { type: 'image/jpeg' });
            objectUrl = URL.createObjectURL(blob);
            setUrl(objectUrl);
        });

        return () => {
            cancelled = true;
            if (objectUrl) {
                URL.revokeObjectURL(objectUrl);
            };
        };
    }, [src]);

    useEffect(() => {
        const onKey = (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
                onClose();
            };
        };
        window.addEventListener('keydown', onKey);
        return () => {
            window.removeEventListener('keydown', onKey);
        };
    }, [onClose]);

    return (
        <div className="lightbox-overlay" onClick={onClose}>
            {url ? (
                <img src={url} alt={src} className="lightbox-image" onClick={(e) => e.stopPropagation()} />
            ) : (
                <p>Loading...</p>
            )}
        </div>
    )
}
