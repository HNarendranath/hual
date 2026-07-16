import { useEffect, useState } from "react";
import { getCachedThumbnail, Photo } from "../lib/ipc";

interface Props {
    photo: Photo;
    thumbcacheDir: string;
    onClick?: () => void;
}

export function PhotoThumbnail({ photo, thumbcacheDir, onClick}: Props) {
    const [url, setUrl] = useState<string | null>(null);

    useEffect(() => {
        let objectUrl: string | null = null;
        let cancelled = false;

        getCachedThumbnail(thumbcacheDir, photo.srcPath).then((bytes) => {
            if (cancelled || !bytes) return;
            const blob = new Blob([bytes], {type: 'image/webp'});
            objectUrl = URL.createObjectURL(blob);
            setUrl(objectUrl);
        })

        // cleanup
        return () => {
            cancelled = true;
            if (objectUrl) URL.revokeObjectURL(objectUrl);
        };
    }, [thumbcacheDir, photo.srcPath]);

    if (!url) {
        return <div className="thumbnail thumbnail-placeholder" />;
    }

    return <img src={url} alt={photo.srcPath} className="thumbnail" onClick={onClick} />;    
}