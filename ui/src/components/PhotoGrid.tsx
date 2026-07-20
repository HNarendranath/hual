import { PhotoThumbnail } from './PhotoThumbnail';
import { useVirtualGrid } from '../hooks/useVirtualGrid';
import { useState } from 'react';
import { Lightbox } from './Lightbox';
import { Photo } from '../lib/ipc';
import { PanelLeftOpen } from 'lucide-react';

const TILE_SIZE = 160;
const GAP = 8;

interface Props {
    photos: Photo[];
    loading: boolean;
    error: string | null;
    thumbcacheDir: string | null;
    sidebarOpen: boolean;
    onOpenSidebar: () => void;
}

export function PhotoGrid({ photos, loading, error, thumbcacheDir, sidebarOpen, onOpenSidebar }: Props) {
    const { containerRef, columns, startIndex, endIndex, totalHeight, offsetY } = useVirtualGrid({
        itemCount: photos.length,
        tileSize: TILE_SIZE,
        gap: GAP,
    });
    const [selected, setSelected] = useState<Photo | null>(null);

    const visiblePhotos = photos.slice(startIndex, endIndex);
    const showEmptyState = !loading && !error && photos.length === 0;

    return (
        <div className="photo-grid-container">
            <div className="content-header">
                {!sidebarOpen && (
                    <button className="icon-button" onClick={onOpenSidebar} aria-label="Open sidebar">
                        <PanelLeftOpen size={18} />
                    </button>
                )}
                {!loading && !error && photos.length > 0 && (
                    <span className="photo-count">{photos.length} photos</span>
                )}
            </div>
            <div ref={containerRef} className="photo-grid-viewport">
                {loading && <p>Loading...</p>}
                {error && <p>Error: {error}</p>}
                {showEmptyState && (
                    <div className="grid-empty-state">
                        <p>No photos yet — open the sidebar to import a library.</p>
                    </div>
                )}
                {!loading && !error && !showEmptyState && (
                    <div style={{ height: totalHeight, position: 'relative' }}>
                        <div
                            className="photo-grid"
                            style={{
                                gridTemplateColumns: `repeat(${columns}, ${TILE_SIZE}px)`,
                                gap: GAP,
                                transform: `translateY(${offsetY}px)`,
                                position: 'absolute',
                                top: 0,
                                left: 0,
                            }}
                        >
                            {visiblePhotos.map((photo) => (
                                <PhotoThumbnail
                                    key={photo.srcPath}
                                    photo={photo}
                                    thumbcacheDir={thumbcacheDir ?? ''}
                                    onClick={() => setSelected(photo)}
                                />
                            ))}
                        </div>
                    </div>
                )}
            </div>
            {selected && <Lightbox photo={selected} onClose={() => setSelected(null)} />}
        </div>
    );
}
