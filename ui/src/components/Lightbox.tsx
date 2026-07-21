import { useEffect, useState } from 'react';
import { getFullThumbnail, Photo } from '../lib/ipc'
import { formatExposureTime, formatFocalLength } from '../lib/utils';
import { ChevronLeft, ChevronRight } from 'lucide-react';

interface Props {
    photo: Photo;
    onClose: () => void;
    onPrev?: () => void;
    onNext?: () => void;
}

export function Lightbox({ photo, onClose, onPrev, onNext }: Props) {
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
            } else if (e.key === 'ArrowLeft' && onPrev) {
                onPrev();
            } else if (e.key === 'ArrowRight' && onNext) {
                onNext();
            };
        };
        window.addEventListener('keydown', onKey);
        return () => {
            window.removeEventListener('keydown', onKey);
        };
    }, [onClose, onPrev, onNext]);

    return (
        <div className="lightbox-overlay" onClick={onClose}>
            {url ? (
                <img src={url} alt={photo.srcPath} className="lightbox-image" onClick={(e) => e.stopPropagation()} />
            ) : (
                <p>Loading...</p>
            )}
            {onPrev && (
                <button
                    className="lightbox-nav lightbox-nav-prev"
                    onClick={(e) => { e.stopPropagation(); onPrev(); }}
                    aria-label="Previous photo"
                >
                    <ChevronLeft size={28} />
                </button>
            )}
            {onNext && (
                <button
                    className="lightbox-nav lightbox-nav-next"
                    onClick={(e) => { e.stopPropagation(); onNext(); }}
                    aria-label="Next photo"
                >
                    <ChevronRight size={28} />
                </button>
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
