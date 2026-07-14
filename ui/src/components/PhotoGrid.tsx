import { usePhotos } from '../hooks/usePhotos';
import { PhotoThumbnail } from './PhotoThumbnail';

interface Props {
    dbPath: string;
    thumbcacheDir: string;
}

export function PhotoGrid({ dbPath, thumbcacheDir }: Props) {
    const { photos, loading, error } = usePhotos(dbPath);

    if (loading) return <p>Loading...</p>;
    if (error) return <p>Error: {error}</p>;

    return (
        <div className='photo-grid'>
            {photos.map((photo) => (
                <PhotoThumbnail key={photo.srcPath} photo={photo} thumbcacheDir={thumbcacheDir} />
            ))}
        </div>
    );
}