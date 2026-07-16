import { usePhotos } from '../hooks/usePhotos';
import { PhotoThumbnail } from './PhotoThumbnail';
import { useVirtualGrid } from '../hooks/useVirtualGrid';
import { useState } from 'react';
import { Lightbox } from './Lightbox';
import { Photo } from '../lib/ipc';

const TILE_SIZE = 160;
const GAP = 8;

interface Props {
    dbPath: string;
    thumbcacheDir: string;
}

export function PhotoGrid({ dbPath, thumbcacheDir }: Props) {
    const { photos, loading, error } = usePhotos(dbPath);
    const { containerRef, columns, startIndex, endIndex, totalHeight, offsetY } = useVirtualGrid({
        itemCount: photos.length,
        tileSize: TILE_SIZE,
        gap: GAP,
    });
    const [selected, setSelected] = useState<Photo | null>(null);

    const visiblePhotos = photos.slice(startIndex, endIndex);

    return (
        <>
        <div ref={containerRef} className="photo-grid-viewport">
            {loading && <p>Loading...</p>}
            {error && <p>Error: {error}</p>}
            {!loading && !error && (
            <div style={{ height: totalHeight, position: 'relative' }}>
                <div
                    className="photo-grid"
                    style = {{
                        gridTemplateColumns: `repeat(${columns}, ${TILE_SIZE}PX)`,
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
                            thumbcacheDir={thumbcacheDir}
                            onClick={() => setSelected(photo)}
                        />
                    ))}
                </div>
            </div>
            )}
        </div>
        {selected && <Lightbox src={selected.srcPath} onClose={() => setSelected(null)} />}
        </>
    );
}