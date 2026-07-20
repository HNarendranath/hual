import { useEffect, useState } from 'react';
import { getFullThumbnail, Photo } from '../lib/ipc'
import { formatExposureTime, formatFocalLength } from '../lib/utils';

interface Props {
    photo: Photo;
    onClose: () => void;
}

export function Lightbox({ photo, onClose }: Props) {
    const [url, setUrl] = useState<string | null>(null);

    useEffect(() => {
        let objectUrl: string | null = null;
        let cancelled = false;

        getFullThumbnail(photo.srcPath).then((bytes) => {
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
    }, [photo.srcPath]);

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
                <img src={url} alt={photo.srcPath} className="lightbox-image" onClick={(e) => e.stopPropagation()} />
            ) : (
                <p>Loading...</p>
            )}
            <div className="lightbox-meta" onClick={(e) => e.stopPropagation()}>
                {photo.focalLength !== null && <span>{formatFocalLength(photo.focalLength)}</span>}

                {photo.iso !== null && <span>ISO {photo.iso}</span>}
                {photo.fStop !== null && <span>f/{photo.fStop}</span>}
                {photo.exposureTime !== null && <span>{formatExposureTime(photo.exposureTime)}</span>}
            </div>
        </div>
    );
}
